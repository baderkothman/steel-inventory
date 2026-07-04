# Supplier-Specific Tracking, Comparison & Settlement — User & Developer Guide

This document explains the supplier features added on top of the existing inventory /
sales / invoices / expenses system: supplier-specific product variants, cheapest-supplier
comparison, supplier credit settlement from actual sales, and the printable stock count sheet.

## 1. Concepts

- **Product variant = one supplier's version of a specification.** The same physical spec
  (e.g. "round pipe 2 inch 2 mm") bought from Company X and Company Y is stored as **two
  product rows**, each with its own supplier, cost/selling price, stock, and SKU. They share
  a supplier-independent `spec_key` so the system can group and compare them.
- **Unknown Supplier fallback.** Products created without a supplier are linked to an
  auto-created `Unknown Supplier` row (created by migration `003`). The supplier field is
  nullable; nothing breaks if it is missing. No supplier name is hard-coded in application
  logic — the fallback is looked up by name only at migration/seed time.
- **Supplier payable = cost of that supplier's goods that were actually sold.** When an item
  sells on a **completed** invoice, the sale carries the supplier through the product link,
  and the supplier's payable is the sold quantity × snapshot unit cost. Cancelled/returned
  sales are excluded automatically.

## 2. Database changes (migration `003_supplier_product_variants.sql`)

Additive and idempotent (applied once, recorded in `schema_migrations`). Existing data is
preserved.

- `products.supplier_id` → `suppliers(id)`, nullable.
- `products.spec_key` (NOT NULL, default '') — shared specification key:
  `TYPE|MATERIAL|SHAPE|FINISH|SIZE|THICKNESS`, uppercased.
- `products.location` — optional storage area.
- Backfill: creates `Unknown Supplier`, assigns existing products to it, fills `spec_key`.
- New table `supplier_settlement_payments` — period payments to a supplier
  (`amount_cents`, `status` ∈ unpaid/partial/paid, `payment_date`, `reference`, `notes`).
- Indexes on `products(supplier_id)`, `products(spec_key)`, and the settlement table.

`spec_key` is recomputed in Rust (`utils::sku::spec_key_from_product`) on every product
create/update; the Rust formatting mirrors SQLite's `CAST(thickness AS TEXT)` so legacy and
new rows for the same spec produce identical keys.

## 3. Backend API (Tauri commands)

- `list_products` — now accepts `supplier_id` filter; rows include `supplier_id`,
  `supplier_name`, `spec_key`, `location`.
- `list_supplier_variants { search, category_id, in_stock_only }` — products grouped by
  `spec_key`, cheapest selling price first within each group.
- `get_stock_count_report { category_id, supplier_id, payment_status="low"? }` — printable
  count rows (system qty + blank counted/difference).
- `get_cheapest_supplier_report { category_id }` — same-spec variants with a "cheapest" flag.
- `get_supplier_settlement_report { date_from, date_to, supplier_id }` — owed per supplier &
  product (completed sales only).
- `get_supplier_settlement_summary { date_from, date_to, supplier_id }` — owed vs settled vs
  remaining per supplier.
- `create_settlement_payment`, `list_settlement_payments`, `delete_settlement_payment`.

## 4. Using the features (shop owner)

### Add the same product from two suppliers
Products → **Add product**. Fill the spec, pick **Supplier** (X), set its prices/stock, save.
Repeat with the same spec but pick supplier Y and Y's prices. Both appear in the product list
with their **Supplier** column, and auto-generated SKUs are made unique per supplier.

### Find the cheapest supplier for a product
Reports → **Cheapest supplier comparison**. Optionally filter by category. Same-spec variants
are listed together, cheapest selling price first, with supplier, available quantity, cost,
selling price, and a "cheapest = Yes" marker.

### Sell from the correct supplier
Sales → **New invoice**. The product picker and the item rows show the **supplier name** so
you know whose stock you are selling. Selling a variant only reduces that supplier's stock.
The printed invoice shows the supplier next to each product name.

### Daily / weekly supplier settlement (what to pay each supplier)
Reports → **Supplier settlement (sold goods)** — set From/To (same day = daily, Mon–Sun =
weekly), optionally a supplier. Shows supplier, product, quantity sold, unit cost, amount
owed. **Supplier settlement summary** shows grand total owed, already settled, and remaining
per supplier. Record a payment with `create_settlement_payment` (amount, status, date,
reference, notes); the summary's settled/remaining update accordingly.

### Printable stock count sheet (physical vs system)
Reports → **Stock count (printable)**. Filter by category, supplier, and All/Low stock, then
**Print count sheet**. The sheet has SKU, Product, Supplier, Category, Location, Unit, System
Qty, and blank **Counted Qty** / **Difference** columns, plus Prepared by / Checked by lines
and the generation date. Print or Save as PDF from the preview.

## 5. Correctness guarantees

- Settlement totals count **completed** sales only; cancelling/returning a sale reverses its
  stock and removes it from payables.
- Stock decrements only from the sold variant's own `stock_levels` row.
- Existing sales, purchases, expenses, products, and stock flows are unchanged.

## 6. Tests

`src-tauri/src/tests.rs` covers: two suppliers sharing one `spec_key`; selling one variant
not touching the other's stock; settlement owed to the correct supplier with cancelled sales
excluded; partial settlement remaining balance; Unknown-Supplier fallback; and the seed
producing two variants under one `spec_key`. Run with `cargo test` in `src-tauri`.
