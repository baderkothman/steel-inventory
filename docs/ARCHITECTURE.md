# Architecture

Steel Inventory is a local desktop application with a React presentation layer and a Rust/SQLite business layer inside Tauri. There is no application server and no cloud synchronization.

## Request flow

```text
React page or dialog
  -> src/lib/api.ts domain method
  -> src/lib/tauri.ts argument normalization
  -> registered Tauri command
  -> Rust service and validation
  -> rusqlite transaction / query
  -> serialized model or structured AppError
  -> TanStack Query cache and UI
```

The frontend never opens SQLite directly. Tauri commands are thin authenticated adapters; business rules belong in services so tests can exercise them without a desktop runtime.

## Frontend

`src/main.tsx` composes the Material UI theme, TanStack Query, hash router, global error boundary, and authentication provider. `src/App.tsx` gates all business routes behind the local administrator session.

The frontend is organized by responsibility:

- `src/app` — layout, navigation, and theme
- `src/features` — page-level business workflows
- `src/components` — tables, feedback, print, confirmation, money, and updater UI
- `src/lib/api.ts` — typed domain facade over Tauri commands
- `src/lib/tauri.ts` — command invocation and structured-error normalization
- `src/lib` — validation, formatting, constants, and exports
- `src/types` — TypeScript transport/domain shapes

TanStack Query caches reads for 20 seconds, retries once, and does not refetch merely because the window regains focus. Mutations explicitly invalidate related query keys.

## Tauri boundary

All callable commands are registered in `src-tauri/src/lib.rs`. Command modules in `src-tauri/src/commands` obtain `AppState`, require a current user where appropriate, open a connection, and delegate to `src-tauri/src/services`.

Tauri exposes Rust command parameter names to JavaScript as camelCase. The frontend domain facade uses database-style names and `normalizeCommandArgs` converts only top-level keys. Nested payload/filter fields stay snake_case because Serde deserializes them into Rust models.

Errors cross the boundary as `{ code, message }`. An `UNAUTHORIZED` response dispatches a browser event that clears the frontend session.

## Backend

The Rust crate is split into:

- `commands` — Tauri adapters and authorization checks
- `services` — transactions, SQL, validation, accounting, stock, reports, and print HTML
- `models` — Serde input/output contracts
- `db` — connection settings and ordered migrations
- `utils` — errors, money, dates, SKU/spec-key generation, validation, and audit helpers
- `state` — database path and the in-memory administrator session

Each operation opens a SQLite connection with foreign keys enabled, WAL journal mode, and a five-second busy timeout. Multi-row mutations use transactions; purchase-return writes use immediate transactions to serialize competing stock changes.

## State and persistence

`AppState` contains only the database path and an optional session guarded by a mutex. The session is not persisted and expires after eight hours.

The database path is resolved with `directories::ProjectDirs` for the `com/local/SteelInventory` identity. Migrations run synchronously during application initialization. After initialization, startup attempts an automatic backup if no successful automatic backup exists for the current day.

## Accounting and inventory design

Money is stored in integer cents to avoid floating-point rounding. Physical quantities use real numbers because steel may be measured in pieces, metres, kilograms, or sheets.

The inventory ledger is authoritative. Every opening stock, purchase, sale, return, and manual adjustment writes `inventory_transactions`. `stock_levels` is a fast current-quantity cache and can be recalculated from active ledger rows.

Sales item rows snapshot cost and price. Historical profit therefore remains stable when a product's current prices change. Purchase item rows snapshot unit cost, and purchase-created price records retain a link to the source invoice for cancellation/restoration.

Balances and payment summaries are derived from active source rows. SQLite triggers protect the polymorphic `payments.party_id` relationship and reject invalid direction, party, or invoice combinations that ordinary foreign keys cannot express.

## Lifecycle strategy

Operational deletion usually means cancellation or archiving first:

- Categories, products, suppliers, and customers use active/archive state plus `deleted_at`.
- Purchases, sales, payments, expenses, inventory adjustments, and settlements retain cancelled rows.
- Cancelled rows are excluded from live stock, balances, dashboard totals, and reports.
- Permanent-delete services reject records whose business history must be retained.
- Purchase-return items are revisioned; edits supersede old rows instead of overwriting history.

The configured retention period is stored in settings, but retention-based background purging is not currently implemented. Permanent deletion is an explicit service action.

## Security boundary

- One local administrator role
- Argon2 password hashes with random salts
- Eight-hour in-memory session
- Authentication required by business commands
- Administrator check on purchase-return mutations and destructive reset
- Audit rows for important mutations and authentication events

This is local application security, not a multi-user authorization system. Anyone with operating-system access to the user's application-data directory may be able to copy or replace the SQLite database. Protect the OS account and backups accordingly.

## Desktop and update integration

Tauri plugins provide file dialogs, process restart, and updates. The committed capability file grants the main window only the required core, dialog, process, and updater permissions.

The updater reads a public key and GitHub `latest.json` endpoint from `src-tauri/tauri.conf.json`. Releases create updater archives and signatures. The frontend blocks self-update when macOS reports a read-only DMG or translocated installation.
