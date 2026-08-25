# Data model

The SQLite schema is created through ordered, append-only migrations in `src-tauri/src/db/migrations`. Applied names are recorded in `schema_migrations`.

## Domain tables

### Identity and settings

- `users` — the local administrator and Argon2 password hash
- `company_settings` — company identity, currency, numbering, stock policy, backup path, tax/profit defaults, and retention setting
- `audit_logs` — actor, action, table, record, and before/after JSON
- `backups` — backup path, type, status, notes, and timestamp

### Catalog and inventory

- `categories` — hierarchical product categories
- `products` — steel identity/specification, supplier, `spec_key`, location, unit, and lifecycle state
- `product_prices` — effective-dated cost/selling/wholesale snapshots and optional purchase source
- `stock_levels` — one cached current/minimum quantity row per product
- `inventory_transactions` — append-oriented stock movement ledger with source references and lifecycle state

### Parties and settlement

- `suppliers` and `customers` — contact details, opening balance, notes, and lifecycle state
- `payments` — customer money-in or supplier money-out, optionally linked to an invoice
- `supplier_settlement_payments` — dated payments against a supplier sold-goods settlement period

### Purchases and returns

- `purchase_invoices` and `purchase_invoice_items` — purchase header and immutable line snapshots
- `purchase_returns` — return header, financial allocation, idempotency key, revision, and lifecycle
- `purchase_return_items` — revisioned return lines tied to original purchase lines

### Sales

- `sales_invoices` and `sales_invoice_items` — customer or walk-in sale, payment state, and cost/price/profit snapshots
- `walk_in_sales_payments` — installments for sales without a customer party

### Expenses

- `expense_categories` — seeded expense classification
- `expenses` — expense amount, paid/remaining totals, and lifecycle
- `expense_payments` — installment rows for an expense

## Money, quantities, and dates

All persisted money uses signed SQLite integers representing the smallest currency unit. API fields use a `_cents` suffix. Formatting into the configured currency happens at the UI/print boundary.

Quantities and dimensions use SQLite `REAL` and Rust `f64`. Services validate that transactional quantities are positive and enforce stock policy before committing.

Dates and timestamps are stored as ISO-style text. Reporting filters use inclusive date ranges.

## Product variants and `spec_key`

One product row represents one supplier's stocked and priced variant. Equivalent variants share a normalized `spec_key` built from product type, material, shape, finish, size, and thickness. This supports cheapest-supplier comparison without merging stock or prices.

Products created without an explicit supplier use the seeded Unknown Supplier fallback. Selling one variant only affects that product's stock row and attributes the supplier cost through the product link.

## Inventory rules

`inventory_transactions` stores quantities in and out. Active current stock is:

```text
sum(quantity_in - quantity_out) for active movements of the product
```

`stock_levels.current_quantity` caches that value. Services update or recalculate the cache inside the same transaction as the source mutation. Migration `005_transaction_lifecycle` also rebuilt the cache from the active ledger to repair legacy drift.

Reference fields link movements to products, purchases, sales, manual adjustments, and purchase-return records. Purchase-return item IDs have a partial unique index so only one movement can represent a specific active return-item revision.

## Invoice and payment rules

Invoice headers cache subtotal, adjustments, total, paid, remaining, and payment status. Payment rows remain the detailed source for installments.

The database validates polymorphic payments with triggers:

- Customer payments must be money in; supplier payments must be money out.
- The party row must exist.
- A referenced sales invoice must belong to that customer and be completed.
- A referenced purchase invoice must belong to that supplier and be active.
- A reference ID cannot exist without a reference type.

Walk-in sales and expenses use dedicated installment tables because they do not fit the customer/supplier payment relationship.

## Purchase-return rules

A purchase return belongs to one active purchase and its supplier. Return lines must match an original invoice line's product and snapshot cost.

The service calculates proportional discount, tax, and shipping credits, reduces stock, updates the invoice's returned/net totals, and changes supplier balance in one transaction. The idempotency key makes repeated create requests safe.

Editing a return increments `current_revision`, writes new item rows, marks old item rows `superseded`, cancels old inventory effects, and creates replacement effects. Cancelling and restoring toggle the active accounting/inventory effect without duplicating ledger rows.

## Migrations

1. `001_initial_schema` — base identity, catalog, parties, invoices, inventory, expenses, payments, audit, and backups
2. `002_seed_data` — settings, steel category tree, expense categories, and starter system data
3. `003_supplier_product_variants` — supplier link, `spec_key`, location, and settlement payments
4. `004_accounting_integrity` — payment cleanup, reconciliation, and integrity triggers
5. `005_transaction_lifecycle` — cancellation state, source-linked price snapshots, and stock-cache repair
6. `006_recently_deleted` — `deleted_at` fields and retention setting
7. `007_installment_payments` — expense and walk-in sale installment ledgers
8. `008_remove_demo_data` — safe retirement of previously seeded demo records
9. `009_purchase_returns` — revisioned purchase returns and inventory links

Never edit a migration that may already have run on a user database. Add the next numbered migration and register it in `src-tauri/src/db/migrations.rs`.

## Reset behavior

Clear All Data deletes business tables in dependency order, resets their SQLite sequences, and reseeds default categories, expense categories, and Unknown Supplier. It preserves `users`, `company_settings`, and `schema_migrations`, then records a new audit event.
