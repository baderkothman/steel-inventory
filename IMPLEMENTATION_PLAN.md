# Steel Inventory Desktop System - Implementation Plan and Current Status

**Source of truth:** `SRS.md` v1.0 and `SDS.md` v1.0  
**Current implementation status:** Implemented and build-verified  
**Last updated:** 2026-06-28  
**Project path:** `D:\steel-inventory`

This document has been updated from the original pre-implementation roadmap to reflect what has been implemented so far. The system now contains the Tauri desktop shell, React/MUI frontend, Rust command backend, SQLite schema and migrations, core workflows, reports, backup/restore, tests, and Windows build artifacts.

---

## 1. Current Outcome

The project is now a runnable offline-first Windows desktop application for one admin user.

Implemented stack:

| Layer              | Implemented technology                                    |
| ------------------ | --------------------------------------------------------- |
| Desktop shell      | Tauri 2                                                   |
| Frontend           | React + TypeScript                                        |
| UI library         | MUI                                                       |
| Backend logic      | Rust Tauri commands                                       |
| Database           | SQLite                                                    |
| Database access    | `rusqlite` with bundled SQLite                            |
| Reports / invoices | HTML print templates and browser print / Save as PDF flow |
| Backup             | Local SQLite backup files                                 |
| Build output       | Windows executable, MSI installer, NSIS installer         |

Build artifacts produced:

- `src-tauri/target/release/steel_inventory.exe`
- `src-tauri/target/release/bundle/msi/Steel Inventory_0.1.0_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/Steel Inventory_0.1.0_x64-setup.exe`

---

## 2. Implemented Modules

| Module               | Status | Implemented scope                                                                                                                 |
| -------------------- | -----: | --------------------------------------------------------------------------------------------------------------------------------- |
| Auth                 |   Done | First-run admin setup, login, logout, password change, Argon2 hashing, session guard                                              |
| Dashboard            |   Done | Daily sales, profit, expenses, net profit, debts, low-stock products, stock value, recent invoices                                |
| Categories           |   Done | List, add, edit, archive, parent category support, seeded steel category tree                                                     |
| Products             |   Done | Add, edit, archive, search/filter, SKU generation, price history insertion, current stock, movement drawer                        |
| Inventory            |   Done | Stock levels, inventory transactions, opening stock, adjustment in/out, damaged stock                                             |
| Suppliers            |   Done | Add, edit, archive, balance, statement                                                                                            |
| Customers            |   Done | Add, edit, archive, balance, statement, walk-in sales support                                                                     |
| Purchases / Stock In |   Done | Purchase invoice builder, supplier selection, items, totals, paid amount, stock increase, supplier payment, cancel, print         |
| Sales Invoices       |   Done | Sales invoice builder, customer/walk-in selection, stock check, item snapshots, profit calculation, stock decrease, cancel, print |
| Expenses             |   Done | Seeded expense categories, add, edit, delete, list                                                                                |
| Payments             |   Done | Customer money-in, supplier money-out, optional invoice link, delete with linked invoice cache reversal                           |
| Reports              |   Done | Required report categories, dashboard queries, CSV export, print                                                                  |
| Settings             |   Done | Company data, currency, invoice prefixes, negative-stock toggle, backup path, default tax value, profit method                    |
| Backup / Restore     |   Done | Manual backup, automatic daily backup, restore with emergency backup, backup log                                                  |
| Audit Log            |   Done | Audit records for major create/update/archive/delete/cancel/login/backup actions                                                  |
| Demo Data Seed       |   Done | Dashboard action inserts realistic sample rows across all main pages without duplicate inserts                                    |

---

## 3. Implemented Project Structure

The repository now follows the SDS feature-based structure.

```text
steel-inventory/
  package.json
  README.md
  IMPLEMENTATION_REPORT.md
  src/
    main.tsx
    App.tsx
    app/
      layout/
      theme.ts
    components/
      feedback/
      print/
      MoneyText.tsx
      PageHeader.tsx
    features/
      auth/
      backup/
      categories/
      dashboard/
      expenses/
      invoices/
      parties/
      payments/
      products/
      reports/
      settings/
    lib/
      api.ts
      constants.ts
      formatters.ts
      tauri.ts
      validators.ts
    types/
  src-tauri/
    Cargo.toml
    tauri.conf.json
    icons/icon.ico
    src/
      lib.rs
      main.rs
      commands/
      db/
        connection.rs
        migrations.rs
        migrations/
      models/
      services/
      utils/
      tests.rs
```

Notable implementation choices:

- Frontend database access is not implemented and is intentionally blocked by architecture.
- All database operations go through Tauri commands.
- Write-heavy workflows run inside Rust services and SQLite transactions.
- React Query is used for frontend server-state caching and invalidation.
- A shared `src/lib/api.ts` wrapper centralizes all Tauri `invoke` calls.

---

## 4. Database Implementation Status

Implemented migrations:

- `001_initial_schema.sql`
- `002_seed_data.sql`

Implemented tables:

- `users`
- `company_settings`
- `categories`
- `products`
- `product_prices`
- `suppliers`
- `customers`
- `purchase_invoices`
- `purchase_invoice_items`
- `sales_invoices`
- `sales_invoice_items`
- `inventory_transactions`
- `stock_levels`
- `expense_categories`
- `expenses`
- `payments`
- `audit_logs`
- `backups`
- `schema_migrations`

Implemented database rules:

- Primary keys use `INTEGER PRIMARY KEY AUTOINCREMENT`.
- Dates are stored as text.
- Money is stored as integer cents.
- Stock quantities are stored as `REAL`.
- Foreign keys are enabled on every SQLite connection.
- WAL mode and busy timeout are configured.
- Required SDS indexes are implemented.
- Master records use archive/status fields instead of destructive deletes.
- Expenses and payments can be deleted where required by the SRS.

Seed data:

- One `company_settings` row.
- Default expense categories: Rent, Electricity, Fuel, Delivery, Salary, Maintenance, Tools, Packaging, Other.
- Suggested steel category tree from the SRS.

Additional schema fields added to satisfy FR-018:

- `company_settings.default_tax_rate`
- `company_settings.default_profit_method`

---

## 5. Backend Implementation Status

Implemented Rust layers:

- `commands/`: Tauri command API exposed to the frontend.
- `services/`: business logic, validation, transactions, reporting, backup.
- `db/`: database connection and migration runner.
- `models/`: DTOs and response types.
- `utils/`: errors, dates, money, SKU generation, validation, audit helpers.
- `state.rs`: app state, DB path, local session handling.

Implemented command groups:

- Auth commands
- Category commands
- Product commands
- Supplier commands
- Customer commands
- Purchase commands
- Sales commands
- Expense commands
- Payment commands
- Report commands
- Settings commands
- Backup commands

Implemented backend guarantees:

- Admin credential stored as Argon2 hash.
- Commands that expose business data require an active session.
- Purchase invoice save is transactional.
- Sales invoice save is transactional.
- Stock updates and inventory transactions occur in the same transaction as invoice save/cancel.
- Sales invoice item cost/price/profit snapshots are stored permanently.
- Structured errors return `{ code, message }`.
- Manual and automatic backups use the SQLite database file and log results.

---

## 6. Frontend Implementation Status

Implemented frontend foundation:

- MUI theme and CSS baseline.
- Auth provider and route guard.
- Sidebar navigation matching the SDS modules.
- Topbar with admin/logout.
- Shared loading, empty, error, confirmation, print, money, and page-header components.
- Typed frontend data models.
- Typed Tauri command wrappers.

Implemented screens:

- First-run setup / login
- Dashboard
- Categories
- Products
- Suppliers
- Customers
- Purchases / Stock In
- Sales Invoices
- Expenses
- Payments
- Reports
- Settings
- Backup

Implemented UX behavior:

- Loading states.
- Empty states.
- Error messages from structured backend errors.
- Confirmation prompts for sensitive actions.
- Print preview dialog for invoices.
- CSV export for reports.
- Query invalidation after mutations that affect dashboard, stock, debt, invoices, payments, or reports.
- Dashboard seed action for populating demo rows across all implemented modules.

---

## 7. Business Workflow Status

| Workflow                | Status | Current behavior                                                                                       |
| ----------------------- | -----: | ------------------------------------------------------------------------------------------------------ |
| First run / login       |   Done | First launch creates admin; later launches require login                                               |
| Product setup           |   Done | Product can be created with generated or manual SKU, prices, initial stock, and minimum stock          |
| Purchase / stock-in     |   Done | Saves invoice/items, increases stock, records inventory transactions, records payment if paid          |
| Sales / stock-out       |   Done | Checks stock unless negative stock enabled, saves snapshots, decreases stock, records payment if paid  |
| Manual stock adjustment |   Done | Supports opening stock, adjustment in/out, damaged stock                                               |
| Invoice cancel          |   Done | Reverses stock, marks invoice cancelled, removes invoice-created payment                               |
| Expense                 |   Done | Validates and records operating expenses; supports edit/delete                                         |
| Payment                 |   Done | Records customer/supplier payments; linked payment updates cached invoice paid/remaining/status        |
| Statements              |   Done | Customer/supplier running balance statements                                                           |
| Reports                 |   Done | Required operational, profit, debt, stock, expense, payment, inventory value, and best-selling reports |
| Invoice print / PDF     |   Done | HTML print dialog; PDF through system/browser print Save as PDF                                        |
| Backup                  |   Done | Manual and automatic daily backup                                                                      |
| Restore                 |   Done | Emergency backup before restore, then database file replacement                                        |

---

## 8. Conflict Resolutions Applied

The original plan listed unresolved conflicts. These are now resolved in the implementation as follows.

| Conflict / ambiguity                  | Implemented resolution                                                                                                                                                                                   |
| ------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Debt double-counting risk             | Statements and debt reports use opening balance + active invoice totals - payments. Invoice paid amounts are also recorded as payment rows. `remaining_cents` is kept as a cached invoice display value. |
| Purchase invoice number uniqueness    | Kept global unique purchase invoice numbers to match SDS `idx_purchase_invoice_number`.                                                                                                                  |
| Payment deletion with linked invoices | Deleting a linked payment reverses the invoice cached paid, remaining, and payment status values in the same transaction.                                                                                |
| Expense deletion                      | Implemented hard delete because FR-012 explicitly allows delete if entered by mistake.                                                                                                                   |
| Tax handling                          | Optional invoice tax values are supported as cents; default tax setting exists.                                                                                                                          |
| Discounts                             | Implemented as amount-based cents values.                                                                                                                                                                |
| Returns workflow                      | No dedicated returns screen was added because it is not a v1 FR. Cancel and manual adjustments cover reversal/adjustment needs.                                                                          |
| Draft sales                           | Saved sales are treated as `completed`; draft UI is not surfaced because no draft workflow is specified.                                                                                                 |
| PIN vs password                       | Single credential field accepts password or PIN and stores one Argon2 hash.                                                                                                                              |
| Programmatic PDF                      | Implemented browser/system print flow; PDF export is through Save as PDF in the print dialog.                                                                                                            |

---

## 9. Milestone Status

| Milestone                    |      Status | Result                                                                                        |
| ---------------------------- | ----------: | --------------------------------------------------------------------------------------------- |
| M1 - Setup                   |        Done | Tauri, React, TypeScript, MUI, SQLite connection, migrations, layout, routing, auth           |
| M2 - Master Data             |        Done | Categories, products, suppliers, customers, settings                                          |
| M3 - Inventory and Purchases |        Done | Stock levels, transactions, adjustments, purchase invoices, cancel, print                     |
| M4 - Sales                   |        Done | Sales invoices, stock check, snapshots, profit, cancel, print                                 |
| M5 - Expenses and Payments   |        Done | Expense categories, expenses, customer/supplier payments, statements                          |
| M6 - Reports and Dashboard   |        Done | Dashboard and required report commands/page                                                   |
| M7 - Backup and Final QA     | Mostly done | Backup/restore, installer build, smoke launch done; manual client-data QA remains recommended |

---

## 10. Verification Completed

Commands run successfully:

```powershell
npm install
npm run build
cd src-tauri
cargo check
cargo test
cd ..
npm run tauri:build
```

Additional verification:

- Release executable launched and stayed running during smoke test.
- MSI installer was produced.
- NSIS setup executable was produced.

Rust tests currently cover:

- Money total and payment status calculations.
- SKU generation for steel product attributes.
- Migration and seed data creation.
- Purchase-to-sale workflow with stock decrease and profit snapshot.

---

## 11. Run and Build Instructions

Development:

```powershell
npm install
npm run tauri:dev
```

Frontend build only:

```powershell
npm run build
```

Rust check and tests:

```powershell
cd src-tauri
cargo check
cargo test
```

Production desktop build:

```powershell
npm run tauri:build
```

---

## 12. Remaining Recommended Work

No known compile, test, build, or launch blockers remain.

Recommended before delivery:

1. Run manual QA with realistic client sample data.
2. Verify first-run setup on a clean Windows user profile.
3. Add sample products, supplier, customer, purchase invoice, sales invoice, expense, and payment.
4. Compare dashboard and reports against hand-calculated values.
5. Print sales and purchase invoices and verify physical/PDF layout.
6. Create a manual backup, restore it, and confirm the app restarts with expected data.
7. Confirm the generated MSI/NSIS installer works on the target Windows machine.

Optional future hardening:

- Add frontend component tests.
- Add Tauri/WebDriver end-to-end smoke tests.
- Add more integration tests for invoice cancellation, linked payment deletion, backup creation, and restore.
- Improve report filters for product/category/supplier/customer selectors in the UI.

---

## 13. Requirement Coverage Summary

Functional requirements FR-001 through FR-020 are implemented in the current codebase.

Non-functional requirements status:

| Requirement           |                                                                                     Status |
| --------------------- | -----------------------------------------------------------------------------------------: |
| Offline-first         |                                                                                Implemented |
| Performance           | Indexed SQLite queries and local execution implemented; manual timing QA still recommended |
| Reliability           |                                       Transactional invoice writes and backups implemented |
| Security              |        Local login, hashed credential, no frontend DB access, guarded commands implemented |
| Usability             |                           Sidebar, tables, dialogs, loading/empty/error states implemented |
| Maintainability       |    TypeScript, feature folders, Rust services, migrations, centralized helpers implemented |
| Windows compatibility |                                           Tauri Windows executable and installers produced |

Out-of-scope version 1 items remain excluded:

- Multi-user permissions
- Cloud sync
- Web dashboard
- Mobile app
- Barcode scanner integration
- Online customer ordering
- Multiple branches
- Multi-device synchronization
- Full accounting ledger
- Tax authority integration
- Supplier invoice OCR

---

## 14. Current Delivery State

The implementation is ready for manual business QA and client installation testing.

See also:

- `README.md`
- `IMPLEMENTATION_REPORT.md`
