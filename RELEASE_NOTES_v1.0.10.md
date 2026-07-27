# Steel Inventory v1.0.10

Released July 27, 2026.

## Highlights

- Fixed duplicate-SKU errors when creating a product with a blank, automatically generated SKU.
- Auto-generated SKUs now receive the next available numeric suffix when the preferred SKU is already retained by an active or archived product.
- Manually entered duplicate SKUs remain blocked to protect data integrity.
- Renamed the product action from **Delete** to **Archive** so the interface accurately communicates that product history is preserved.

## Demo data removal

- Removed the Dashboard demo-data button.
- Removed the demo-data frontend API, backend command, seeding service, and related documentation.
- Added database migration `008_remove_demo_data` to clean previously seeded demo records during upgrade.
- Demo-prefixed products later used on real, non-demo invoices are preserved to avoid changing real invoice history.

## Verification

- Frontend production build passed.
- All 25 Rust tests passed.
- Added regression coverage for auto-generated SKU uniqueness, archived-product replacement, manual duplicate rejection, and demo-data cleanup.
- Release metadata is aligned at version `1.0.10`.

## macOS installation

For a first installation, open the DMG and drag **Steel Inventory** into **Applications** before launching it. Do not run the app directly from the DMG because macOS mounts it read-only. Existing updater-enabled installations in Applications can install this release from inside the app.

The macOS build is ad-hoc signed. On first installation, macOS may require approval in **System Settings → Privacy & Security**.
