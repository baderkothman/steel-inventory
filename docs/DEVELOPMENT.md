# Development

## Prerequisites

- Node.js and npm
- Rust toolchain compatible with Rust 1.80 or newer
- Tauri v2 platform prerequisites for the host operating system
- Xcode command-line tools when developing or packaging on macOS

Install locked frontend dependencies with:

```bash
npm ci
```

## Local commands

```bash
# Frontend only
npm run dev

# Full Tauri application with live frontend
npm run tauri:dev

# Type-check and build frontend
npm run build

# Rust compile checks and tests
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml

# Desktop bundles
npm run tauri:build
```

`npm run dev` runs in a browser but commands that depend on the Tauri runtime will not work there. Use `npm run tauri:dev` for end-to-end workflows.

## Repository layout

```text
src/
  app/                 layout and theme
  components/          shared UI, tables, print, updater, feedback
  features/            business pages and dialogs
  lib/                 API facade, invoke wrapper, constants, formatters
  types/               TypeScript payload and row types
src-tauri/
  capabilities/        Tauri window permissions
  icons/               platform bundle icons
  src/commands/        registered desktop command adapters
  src/db/              connection setup and migrations
  src/models/          Serde contracts
  src/services/        business rules, SQL, reports, and printing
  src/utils/           shared backend helpers
  src/tests.rs          backend integration/regression suite
.github/workflows/     release automation
docs/                   maintained documentation
```

## Adding or changing a workflow

1. Define or update the Rust payload/row model in `src-tauri/src/models/mod.rs`.
2. Put validation and transactional behavior in a service.
3. Add a thin authenticated command adapter.
4. Register the command in `src-tauri/src/lib.rs`.
5. Add or update the typed method in `src/lib/api.ts` and TypeScript types.
6. Implement the feature UI and invalidate affected query keys after mutations.
7. Add regression tests for success, validation, rollback, cancellation, and restoration paths.
8. Update the relevant maintained guide and `CHANGELOG.md` when user-visible behavior changes.

Do not invoke Tauri directly from pages. Keeping `src/lib/api.ts` as the frontend boundary makes argument conventions and response types auditable.

## Database changes

Create the next numbered SQL file under `src-tauri/src/db/migrations` and add it to `MIGRATIONS` in `migrations.rs`. A migration runs once in a transaction and is recorded only after success.

Rules:

- Never modify an already released migration.
- Prefer additive changes and explicit backfills.
- Preserve historical rows and source references.
- Add indexes for common filters and joins.
- Add database constraints/triggers for invariants that direct imports must also obey.
- Test both a fresh database and an upgrade path with representative legacy rows.

## Money and command arguments

Use integer cents in storage, Rust models, and TypeScript transport types. Convert user-entered decimals at the edge and format with the configured currency for display.

Tauri converts Rust command parameter names to camelCase. The shared `call` wrapper normalizes only the top-level argument keys. Keep nested object fields snake_case to match Serde models.

## Testing strategy

The backend suite in `src-tauri/src/tests.rs` uses in-memory SQLite databases and real migrations/services. It covers core money calculations, SKU generation, supplier variants, stock, settlements, payment integrity, lifecycle behavior, reset behavior, installment payments, purchase returns, and report shape.

Before handing off a change, run:

```bash
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Run `npm run tauri:build` for changes to Tauri configuration, capabilities, plugins, updater behavior, icons, or release packaging.

## Release process

The application version must match in:

- `package.json` and `package-lock.json`
- `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

Update `CHANGELOG.md`, commit the release, and create a semver tag such as `v1.0.12`. Pushing a `v*` tag runs `.github/workflows/release-desktop.yml`. The workflow verifies that the tag matches `tauri.conf.json`, builds a universal Apple Silicon/Intel macOS bundle, signs updater artifacts, and publishes the GitHub Release.

The repository secret `TAURI_SIGNING_PRIVATE_KEY` must contain the updater private key. The corresponding public key is committed in `tauri.conf.json`. Never commit the private key. The workflow currently uses ad-hoc macOS code signing; notarization is not configured.

The workflow may also be dispatched manually for an existing release tag. It does not currently publish Windows or Linux installers.

## Documentation policy

- `README.md` is the entry point, not a complete specification.
- `docs/` describes the current implementation.
- `CHANGELOG.md` records historical user-visible changes.
- Do not add one-off implementation plans, audit reports, or per-version release-note files at the repository root.
- Link code comments to stable domain concepts rather than duplicating entire guides.
