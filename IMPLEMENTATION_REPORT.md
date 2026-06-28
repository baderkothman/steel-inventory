# Implementation Report

## Built Scope

Implemented the full offline-first Tauri desktop project described by `SRS.md`, `SDS.md`, and `IMPLEMENTATION_PLAN.md`.

Major implemented areas:

- First-run admin setup, local login, logout, password change, Argon2 password hashing
- SQLite schema, migrations, indexes, seed data, settings row, category tree, default expense categories
- Products, categories, suppliers, customers, purchases, sales, expenses, payments, reports, settings, backup/restore
- Purchase stock-in workflow with invoice items, supplier payment, stock increase, inventory transaction, audit log
- Sales stock-out workflow with stock check, snapshots, profit calculation, customer payment, stock decrease, audit log
- Manual stock adjustments and movement history
- Customer and supplier balances/statements
- Dashboard summary, all required report categories, CSV export, report printing
- Invoice HTML print/PDF flow through the print dialog
- Manual backup, automatic daily backup, restore with emergency backup
- Windows executable, MSI installer, and NSIS installer build
- Explicit Dashboard action for seeding realistic demo data across all main pages

## Source-of-Truth Decisions

- Debt calculations follow FR-014/FR-015 statement behavior: invoices are debits by full total, payments are credits. This avoids double-counting paid-at-invoice amounts.
- Purchase invoice numbers remain globally unique, matching SDS `idx_purchase_invoice_number`.
- Expense delete is a hard delete as explicitly allowed by FR-012.
- Linked payment delete reverses cached invoice paid/remaining/status values in the same transaction.
- `default_tax_rate` and `default_profit_method` were added to `company_settings` to satisfy FR-018.

## Important Files

- Frontend entry: `src/main.tsx`, `src/App.tsx`
- Layout: `src/app/layout/`
- API wrapper: `src/lib/api.ts`
- Feature pages: `src/features/`
- Rust entrypoint: `src-tauri/src/lib.rs`
- Tauri commands: `src-tauri/src/commands/`
- Rust services: `src-tauri/src/services/`
- Migrations: `src-tauri/src/db/migrations/`
- Tests: `src-tauri/src/tests.rs`

## Verification Run

Passed:

- `npm run build`
- `cargo check`
- `cargo test`
- `npm run tauri:build`
- Release executable smoke launch
- `npm run build`, `cargo check`, and `cargo test` after adding demo-data seeding

Installer artifacts:

- `src-tauri/target/release/bundle/msi/Steel Inventory_0.1.0_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/Steel Inventory_0.1.0_x64-setup.exe`

## Remaining Issues

No known compile, test, or build blockers remain.

Manual business QA should still be performed with real client sample data before delivery: create admin, add product/supplier/customer, create purchase, create sale, record expense/payment, inspect dashboard/reports, print invoice, backup, restore.
