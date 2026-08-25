# Changelog

This file consolidates the release notes that were previously stored as separate root-level documents. Only releases with preserved historical notes are listed.

## 1.0.11 — 2026-07-28

### Added

- End-to-end partial and full purchase returns for damaged supplier goods.
- Return reasons, notes, printable documents, history, editing, cancellation, and restoration.
- Revisioned return-item history and idempotency protection for duplicate create requests.
- Migration `009_purchase_returns` with return tables, inventory links, triggers, and indexes.

### Changed

- Purchase returns now update inventory, purchase net totals, supplier balances, statements, tax, discount, and shipping allocation in one transaction.
- Active returns must be cancelled before their source purchase can be cancelled.
- Stock and Stock Count report columns use Name, Thickness, then Price across the UI, print, PDF, Excel, and CSV output.

### Verified

- Added backend coverage for quantities, rollback, idempotency, editing, lifecycle changes, supplier balances, tax allocation, and report ordering.

## 1.0.10 — 2026-07-27

### Fixed

- Blank auto-generated SKUs now receive an available numeric suffix instead of failing on retained active or archived SKUs.
- Manually entered duplicate SKUs remain rejected.

### Changed

- Renamed the product action from Delete to Archive to reflect retained history.
- Removed the demo-data UI, API, command, seeding service, and obsolete documentation.
- Migration `008_remove_demo_data` removes old demo records while preserving demo-prefixed products used by real invoices.

## 1.0.6 — 2026-07-26

### Changed

- Redesigned Stock Remaining for category/product-type grouping and faster scanning.
- Unified product name, size, thickness, configured-currency price, and remaining quantity across screen and print layouts.
- Removed SKU from Stock Remaining and physical stock-count views.

### Fixed

- Corrected customer and supplier outstanding-balance calculations.
- Kept archived-party debt visible until settled and stopped one party's credit from offsetting another party's debt.
- Reconciled invoice summaries when linked payments are added or removed.
- Added database guards against orphaned or mismatched party/invoice payments.
- Migration `004_accounting_integrity` removes invalid legacy payments and reconciles active invoices.
