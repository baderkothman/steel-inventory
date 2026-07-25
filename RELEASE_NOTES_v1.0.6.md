# Steel Inventory v1.0.6

Released July 26, 2026.

## Highlights

- Redesigned Stock Remaining for faster in-store scanning.
- Grouped stock by category and product type.
- Added product name, size, thickness, configured-currency selling price, and remaining quantity.
- Matched the on-screen, print-preview, and printed report layouts.
- Removed SKU from Stock Remaining and physical stock-count views.

## Accounting integrity

- Corrected customer and supplier outstanding-balance calculations.
- Kept archived-party balances in debt totals until they are fully settled.
- Prevented a party credit from incorrectly reducing another party's debt.
- Reconciled linked invoice totals when payments are added or deleted.
- Rejected payments linked to the wrong customer, supplier, or invoice.
- Added database guards against orphaned and mismatched payment records.

On first launch, database migration `004_accounting_integrity` removes legacy invalid payment rows and reconciles active invoice payment summaries. Valid general payments and walk-in sales are preserved.

## Other improvements

- Monetary report values and CSV exports now use the application's configured currency.
- Inventory value and low-stock reports retain their existing operational data after the Stock Remaining redesign.
- Added regression coverage for stock report fields, archived balances, party credits, linked-payment deletion, mismatched invoice payments, and orphan-payment protection.

## Verification

- Frontend production build passed.
- All 17 Rust tests passed.
- Release metadata is aligned at version `1.0.6`.

## macOS installation

Download the DMG for a first installation. Existing updater-enabled installations can install this release from inside the app.

The macOS build is ad-hoc signed. On first installation, macOS may require approval in **System Settings → Privacy & Security**.
