# Steel Inventory

Steel Inventory is an offline-first desktop application for running a small steel inventory business. It manages supplier-specific products, purchases, purchase returns, sales, expenses, payments, stock movement, statements, reports, printing, and local backups from one SQLite database.

Current application version: `1.2.0`.

## What it does

- Tracks steel dimensions, SKUs, locations, prices, minimum stock, and supplier-specific variants.
- Posts purchases and sales to an inventory ledger and maintains a current-stock cache.
- Supports partial invoice and expense payments, customer/supplier statements, and debt reports.
- Handles partial or full purchase returns with revision history and reversible accounting effects.
- Produces operational, profit, stock, debt, supplier-settlement, and inventory-value reports.
- Prints invoices, purchase returns, dashboards, tables, and physical stock-count sheets.
- Stores all business data locally, creates automatic/manual backups, and restores SQLite backups.
- Uses a single local administrator account with an eight-hour in-memory session.

## Technology

- Tauri 2 and Rust for the desktop shell and business layer
- React 19, TypeScript, Material UI, and TanStack Query for the frontend
- SQLite through bundled `rusqlite`
- Vite for frontend development and builds
- GitHub Actions for universal macOS releases and signed updater artifacts

## Quick start

Install the Tauri v2 system prerequisites, Node.js, npm, and Rust 1.80 or newer. Then run:

```bash
npm ci
npm run tauri:dev
```

The first launch asks you to create the one local administrator. Later launches require that account to sign in.

Useful checks:

```bash
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Build a desktop bundle with:

```bash
npm run tauri:build
```

## Documentation

- [User guide](docs/USER_GUIDE.md) — first-run setup and day-to-day workflows
- [Architecture](docs/ARCHITECTURE.md) — layers, request flow, security, and design rules
- [Data model](docs/DATA_MODEL.md) — tables, migrations, accounting, inventory, and lifecycle rules
- [Command reference](docs/COMMANDS.md) — the frontend API and registered Tauri command surface
- [Development](docs/DEVELOPMENT.md) — repository layout, local workflow, testing, and releases
- [Operations](docs/OPERATIONS.md) — installation, updates, database location, backup, and recovery
- [Changelog](CHANGELOG.md) — consolidated release history

## Core invariants

- Monetary values cross the backend boundary and persist as integer cents.
- Business mutations that affect more than one ledger are transactional.
- `inventory_transactions` is the stock history; `stock_levels` is a recalculated read cache.
- Completed sales snapshot cost and profit, so later product-price changes do not rewrite history.
- Cancelled operational records remain available for audit but are excluded from live calculations.
- Database migrations are append-only, ordered, and recorded in `schema_migrations`.

## Distribution status

The committed release workflow publishes a universal macOS application for Apple Silicon and Intel Macs. Windows and Linux source compatibility is not the same as a tested release channel; the current workflow does not publish installers for them.
