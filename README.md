# Steel Inventory Desktop System

Offline-first Windows desktop application for steel inventory, supplier purchases, sales invoices, expenses, customer/supplier payments, reports, invoice printing, and local backup/restore.

## Tech Stack

- Tauri 2 desktop shell
- React + TypeScript frontend
- MUI UI components
- Rust Tauri command backend
- SQLite via `rusqlite` with bundled SQLite
- Local app-data database with migrations

## Run in Development

```powershell
npm install
npm run tauri:dev
```

On first launch, create the single local admin account. Later launches require that admin login before business data is accessible.

After login, open the Dashboard and click `Seed demo data` to populate realistic sample rows across products, suppliers, customers, purchases, sales invoices, expenses, payments, reports, stock movement, and backup logs. The seed is idempotent and will not insert duplicates if demo products already exist.

## Build

```powershell
npm run tauri:build
```

Build outputs:

- `src-tauri/target/release/steel_inventory.exe`
- `src-tauri/target/release/bundle/msi/Steel Inventory_0.1.0_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/Steel Inventory_0.1.0_x64-setup.exe`

## Verification

```powershell
npm run build
cd src-tauri
cargo check
cargo test
```

The current implementation passes frontend build, Rust check, Rust tests, Tauri production build, and a release executable smoke launch.

## Database

The SQLite database is created in the local app-data directory for `SteelInventory`. Migrations run on startup and create the required tables, indexes, settings seed row, default expense categories, and suggested steel category tree.

Money values are stored as integer cents. Stock movements are recorded in `inventory_transactions`, with `stock_levels` maintained for fast current stock reads.

## Implementation Notes

- Invoice paid amounts are recorded as payment rows, while debt reports/statements use the statement-consistent formula: opening balance + active invoice totals - payments.
- Sales invoice items store cost, price, total cost, total price, and profit snapshots.
- Cancelled invoices reverse stock movement and remove linked invoice-created payments.
- The settings table includes `default_tax_rate` and `default_profit_method` because FR-018 requires them, although the SDS table omitted those columns.

## Supplier-Specific Features

Products can be tracked per supplier (the same specification from multiple companies coexists
as priced/stocked variants), compared by cheapest price, settled as daily/weekly supplier
payables based on actual completed sales, and printed as a physical stock count sheet. See
[SUPPLIER_FEATURES.md](SUPPLIER_FEATURES.md) for the full workflow and
[SUPPLIER_FEATURES_AUDIT.md](SUPPLIER_FEATURES_AUDIT.md) for the implementation audit
(what existed, what was missing, what changed). Database changes live in migration
`src-tauri/src/db/migrations/003_supplier_product_variants.sql`.
