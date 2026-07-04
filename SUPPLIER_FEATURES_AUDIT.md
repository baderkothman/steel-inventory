# Supplier-Specific Features — Implementation Audit

Date: 2026-06-30
Scope: Supplier-specific product tracking, cheapest-supplier comparison, supplier credit settlement, printable stock count report.

## 1. What already exists

**Architecture**

- Tauri 2 desktop app. Backend: Rust + `rusqlite` (SQLite), layered as `commands → services → db`. Money stored as integer `*_cents`. Frontend: React 19 + MUI 7 + TanStack Query, Tauri `invoke` via `src/lib/api.ts`.
- Versioned, idempotent migrations in `src-tauri/src/db/migrations.rs` (each `(version, sql)` applied once and recorded in `schema_migrations`). Safe to append a `003_*` migration.

**Suppliers**

- `suppliers` table exists (party). Managed through `party_service` / `supplier_commands` / `supplierApi`. Supplier payables today are derived only from **purchase invoice** balances (`party_balance`), not from sales of their goods.

**Products / stock / pricing**

- `products` table holds the steel specification (type, material, shape, finish, size, dims, unit). **No supplier link.**
- `product_prices` = price history per product (cost/selling/wholesale). Latest row wins.
- `stock_levels` = one current-quantity row per product. `inventory_transactions` = full movement ledger.
- Sales (`sales_service`) snapshot `unit_cost` from latest price into `sales_invoice_items`, decrement `stock_levels`, and write a `sale` inventory transaction. Cancel reverses stock and deletes linked payments. `sales_status` ∈ draft/completed/cancelled/returned (created as `completed`).

**Reports**

- Generic `ReportsPage` renders any backend report as columns/rows with CSV export + `window.print()`. A `stock` report and `low_stock` report already exist (no physical-count or per-supplier columns). `supplier_debt` report exists but reflects purchase-invoice balances, not sold-goods payables.

## 2. What is missing (the client's 4 requests)

1. **Supplier-specific product tracking** — products have no supplier; the same spec from two companies cannot coexist as priced/stocked variants, and no supplier label appears in product list / POS / invoice / reports.
2. **Cheapest supplier comparison** — no way to group by shared specification and compare supplier variants by price.
3. **Supplier credit settlement from actual sales** — no supplier payable accrual on sale, no supplier-aware daily/weekly settlement report, no settlement payment status/notes per period.
4. **Printable stock count report** — existing stock report lacks supplier, counted-qty/difference columns, prepared-by/checked-by, and print-friendly layout/filters.

## 3. Files / tables to change

**Database (new migration `003_supplier_product_variants.sql`)**

- `products`: add `supplier_id INTEGER NULL REFERENCES suppliers(id)`, `spec_key TEXT`, `location TEXT NULL`.
- Backfill: create an **"Unknown Supplier"** fallback and assign existing products to it; populate `spec_key`. (Nullable supplier kept for safety.)
- New table `supplier_settlement_payments` (period payments against a supplier: amount, date, status reference, notes).
- Indexes on `products(supplier_id)`, `products(spec_key)`.

**Backend (Rust)**

- `models/mod.rs`: add `supplier_id` / `supplier_name` / `spec_key` / `location` to `ProductRow` + `ProductPayload`; add settlement payload/row structs; add `supplier_id` to `ProductFilters`.
- `services/product_service.rs`: persist + return supplier/spec_key/location; compute `spec_key`; filter by supplier; new `cheapest_variants` query grouped by spec_key.
- `utils/sku.rs`: helper to build `spec_key` from attributes.
- `services/report_service.rs`: `supplier_settlement_report` (payables from completed sales joined to product → supplier), extend `stock_report` with supplier/location/counted/difference scaffolding, supplier filter.
- New `services/settlement_service.rs` + `commands/settlement_commands.rs`: record/list settlement payments.
- `report_commands.rs` / `product_commands.rs` / `lib.rs`: register new commands.

**Frontend (TS/React)**

- `types/product.ts`, `types/report.ts`, `lib/api.ts`, `lib/constants.ts` (report options + supplier-product report keys).
- `ProductsPage`: supplier select in form + supplier column + supplier filter.
- `InvoicePages`: show supplier label in the product picker and item rows (POS clarity).
- `ReportsPage`: supplier filter dropdown; new report keys (cheapest comparison, supplier settlement, stock count); print-friendly stock-count rendering.
- New `SettlementPage` (or reuse Reports) for recording supplier settlement payments.

## 4. Guarantees / safety

- Migration is additive and idempotent; existing data preserved via Unknown-Supplier backfill; `supplier_id` nullable.
- Sales/purchase/stock/profit flows unchanged in behavior; sales just carry supplier through the existing `product_id` join.
- Settlement payables count **completed sales only** (cancelled/returned excluded), matching existing `sales_status = 'completed'` filters.
- No hard-coded supplier names (fallback row created in DB, looked up by name only at migration time).
