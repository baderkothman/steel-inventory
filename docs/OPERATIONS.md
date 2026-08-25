# Operations

## Supported release channel

The automated release channel produces a universal macOS DMG and updater archive for Apple Silicon and Intel Macs. The source contains general Tauri bundle configuration, but Windows and Linux installers are not built or verified by the committed workflow.

## First installation on macOS

1. Open the downloaded DMG.
2. Drag Steel Inventory into Applications.
3. Eject the DMG.
4. Launch the copy in Applications.

Do not run the app directly from the DMG. A DMG is read-only, so the application cannot replace itself during an update. Because current builds are ad-hoc signed, macOS may require approval in System Settings > Privacy & Security on first launch.

## Database location

The application uses the operating system's local application-data directory resolved for `com/local/SteelInventory`, with the filename `steel_inventory.db`.

On macOS this resolves beneath the user's Library/Application Support directory. Treat the entire database as sensitive business data. SQLite may also create `-wal` and `-shm` sidecars while the app is running; do not copy only the main file from a live database as an informal backup.

The application enables:

- Foreign-key enforcement
- WAL journaling
- A five-second busy timeout
- Bundled SQLite for consistent availability

## Backup behavior

At startup, the application checks whether a successful automatic backup exists for the current date. If not, it attempts one. A failed automatic backup does not block startup.

Manual backups are created from the Backup page. SQLite `VACUUM INTO` writes a consistent standalone database file named like:

```text
steel_inventory_backup_<timestamp>.db
```

The backup directory is the configured Settings path. If none is configured, the application uses `SteelInventoryBackups` under the user's Documents directory (or the home-directory fallback when Documents cannot be resolved).

Backup history in the app is a database log. Deleting or resetting that log does not necessarily delete external backup files, and moving an external file does not update the logged path.

## Restore and recovery

An in-app restore performs these steps:

1. Verify that the selected source path exists.
2. Create an emergency backup of the current active database.
3. Copy the selected database over the active database.
4. Open the restored database and record restore/audit entries.
5. Ask the operator to restart so all application state is reloaded.

Before restoring, preserve a second copy of both the source backup and current database. After restart, verify company settings, latest transactions, stock totals, and reports.

If the app cannot open after a restore, stop it completely and recover the emergency backup using a known-good build. Avoid editing production SQLite files manually unless you have first made a byte-for-byte copy.

## Automatic updates

The desktop updater reads the latest release metadata from the GitHub endpoint configured in `src-tauri/tauri.conf.json` and verifies artifacts with the embedded public key.

For a normal installed copy:

1. The app checks for an update after startup.
2. The user chooses Install and restart.
3. The signed updater artifact is downloaded and installed.
4. The process restarts into the new version.

The updater refuses to proceed from a read-only DMG or detected App Translocation path and explains how to move the app to Applications. Updating the application bundle does not move or replace the user database.

## Clear All Data

Clear All Data is an application reset, not a backup operation. It permanently removes business records and in-app backup history, preserves the administrator and settings, reseeds system defaults, and commits everything atomically.

Create and verify an external manual backup before using it. The action requires current administrator credentials and the phrase `CLEAR ALL DATA`.

## Operational checks

Before a release or after recovery:

- Launch and authenticate with an existing database.
- Confirm migrations complete without errors.
- Create and locate a manual backup.
- Open Dashboard, Products, Purchases, Sales, Payments, and Reports.
- Verify one known product's ledger quantity matches its displayed stock.
- Verify a known customer/supplier statement against its invoices and payments.
- Print or preview one invoice and one report.
- Confirm the updater can read release metadata from an installed Applications copy.

## Limitations

- Single local administrator; no per-user permissions or remote sign-in
- No cloud synchronization or shared multi-device database
- No background purge based on the retention setting
- Current automated distribution is macOS only
- Current macOS release is ad-hoc signed and not notarized
- Backups are local files and must be copied off-device for hardware-loss protection
