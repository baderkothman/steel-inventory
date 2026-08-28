# Changelog

This file consolidates the release notes that were previously stored as separate root-level documents. Only releases with preserved historical notes are listed.

## 1.2.0 — 2026-08-28

### Added

- Dedicated customer quotations with Draft, Sent, Accepted, Rejected, Expired, and Converted states.
- Snapshot customer details, products, SKUs, quantities, prices, discounts, tax/VAT, totals, notes, terms, and validity dates.
- Transactional conversion of accepted quotations into normal sales invoices with current stock validation and duplicate-conversion protection.
- Professional A4 quotation printouts with company branding and an explicit quotation-only disclaimer.
- Validated company and supplier logo upload, preview, replacement, removal, and local application-data storage.

### Changed

- Purchase invoices and purchase returns now place the supplier logo, company name, contact details, reference, and date near the top of the printed document.
- Print preview now waits for images and fonts before opening the system print dialog.
- Quotation values remain isolated from inventory, customer balances, payments, revenue, sales reports, and accounting until conversion.

### Verified

- Added coverage for quotation stock isolation, backend-calculated totals, price snapshots, conversion rollback, duplicate conversion, archived products, A4 printing, logo fallback, embedded supplier branding, and aspect-ratio preservation.
- Frontend production build, Rust formatting/checks, and all 34 backend tests pass.

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
