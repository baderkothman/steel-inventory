# Steel Inventory Desktop System

Offline-first macOS desktop application for steel inventory, supplier purchases, sales invoices, expenses, customer/supplier payments, reports, invoice printing, local backup/restore, and in-app updates.

## Tech Stack

- Tauri 2 desktop shell
- React + TypeScript frontend
- MUI UI components
- Rust Tauri command backend
- SQLite via `rusqlite` with bundled SQLite
- Local app-data database with migrations

## Run in Development

```bash
npm install
npm run tauri:dev
```

On first launch, create the single local admin account. Later launches require that admin login before business data is accessible.

After login, open the Dashboard and click `Seed demo data` to populate realistic sample rows across products, suppliers, customers, purchases, sales invoices, expenses, payments, reports, stock movement, and backup logs. The seed is idempotent and will not insert duplicates if demo products already exist.

## Build

```bash
npm run tauri:build
```

Build outputs:

- `src-tauri/target/release/bundle/macos/Steel Inventory.app`
- `src-tauri/target/release/bundle/dmg/Steel Inventory_<version>_<architecture>.dmg`

## Verification

```bash
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

## macOS Releases and Automatic Updates

The GitHub Actions workflow publishes one universal macOS build that runs on Apple Silicon and Intel Macs. It intentionally does not build or publish Windows assets.

The updater private key is stored locally at `~/.tauri/steel-inventory.key`. Keep an encrypted backup outside this repository. Never commit or share it. Its public key is embedded in `src-tauri/tauri.conf.json`.

Configure the required GitHub repository secret once:

```bash
gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/steel-inventory.key
```

To publish version `1.0.6`, make sure `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` all contain `1.0.6`, then commit and push the code:

```bash
git add .github/workflows/release-desktop.yml README.md RELEASE_NOTES_v1.0.6.md package.json package-lock.json \
  src src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/capabilities \
  src-tauri/gen/schemas src-tauri/src src-tauri/tauri.conf.json
git commit -m "release: v1.0.6"
git push origin master
git tag v1.0.6
git push origin v1.0.6
```

Pushing the tag runs `.github/workflows/release-desktop.yml`. The workflow creates a public GitHub Release containing the universal DMG, signed updater archive, signature, and `latest.json`.
Use `RELEASE_NOTES_v1.0.6.md` as the GitHub Release description after the workflow finishes.

Builds are ad-hoc signed because no Apple Developer identity is currently configured. Users may need to approve the first installation in macOS Privacy & Security. For frictionless public distribution, configure a paid Apple Developer ID certificate and notarization credentials.

The first updater-enabled version must be downloaded and installed once. Every later version is detected on app startup and can be installed with **Install and restart**, without downloading the repository or another DMG manually.
