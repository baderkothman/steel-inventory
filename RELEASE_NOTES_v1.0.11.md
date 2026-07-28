# Steel Inventory v1.0.11

Released July 28, 2026.

## Purchase returns

- Added an end-to-end purchase-return workflow for damaged supplier goods.
- Purchases now expose **Create purchase return**, with partial and full return quantities, optional reasons and notes, printable return documents, and return history.
- Returnable quantities are validated against both the original purchase and currently available stock.
- Inventory, purchase net totals, supplier balances, tax, discount, shipping credits, and supplier statements update automatically in one database transaction.
- Duplicate create requests are idempotent, while edits retain superseded ledger revisions for audit history.
- Canceling and restoring a return reverses or reapplies its inventory and financial effects without duplicate active ledger entries.
- Active returns must be canceled before their original purchase can be canceled.

## Stock reports

- Stock and Stock Count reports now show product information in the order **Name → Thickness → Price**.
- The corrected order is shared by the on-screen table, print views, PDF, Excel, and CSV exports.
- The physical stock-count sheet now includes thickness and price in the same order.

## Data integrity and authorization

- Added migration `009_purchase_returns` with purchase-return headers, revisioned return items, inventory links, integrity triggers, and supporting indexes.
- Return mutations use immediate database transactions to serialize competing requests.
- Purchase-return commands require an authenticated administrator.
- Historical purchase, payment, inventory, and audit records are preserved.

## Verification

- Frontend production build passed.
- All 31 Rust tests passed.
- Added coverage for partial and full returns, excessive quantities, insufficient stock rollback, duplicate requests, edits, cancellation, restoration, inventory changes, supplier balances, tax allocation, and stock report ordering.
- Release metadata is aligned at version `1.0.11`.

## macOS installation

For a first installation, open the DMG and drag **Steel Inventory** into **Applications** before launching it. Do not run the app directly from the DMG because macOS mounts it read-only. Existing updater-enabled installations in Applications can install this release from inside the app.

The macOS build is ad-hoc signed. On first installation, macOS may require approval in **System Settings → Privacy & Security**.
