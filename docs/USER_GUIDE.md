# User guide

This guide describes the workflows available in the current desktop interface. Backend-only capabilities are documented separately in [COMMANDS.md](COMMANDS.md).

## First launch

On a new database, Steel Inventory asks for a full name, email address, and password or PIN. The credential must contain at least four characters and is stored as an Argon2 password hash.

The application supports one active local administrator. A successful sign-in creates an in-memory session that expires after eight hours. The UI also rechecks the session every minute and returns to sign-in after expiration.

Before entering transactions, open Settings and configure:

- Company name and contact details used on printed documents
- Three-letter default currency, such as `SAR` or `USD`
- Sales, purchase invoice, and quotation prefixes
- Company logo used on quotation printouts
- Default tax value and profit method
- Whether sales may take stock below zero
- Backup directory

## Navigation

The sidebar contains Dashboard, Products, Categories, Suppliers, Customers, Purchases, Sales Invoices, Quotations, Expenses, Payments, Reports, Settings, and Backup. The collapse preference is stored in local browser storage and does not affect business data.

## Recommended setup order

1. Configure company settings.
2. Review the seeded steel category tree and add any missing categories.
3. Add suppliers and customers, including opening balances when required.
4. Add products with their supplier, steel specification, location, prices, and opening stock.
5. Start entering purchases, sales, expenses, and payments.
6. Create a manual backup after the initial setup.

## Products and categories

Categories may have parent/child relationships. A category that is referenced by products or children cannot be permanently removed until those references are resolved.

Each product records:

- SKU, name, category, supplier, and storage location
- Type, material, shape, finish, and display size
- Width, height, diameter, thickness, and length where applicable
- Unit, cost price, selling price, wholesale price, and minimum quantity
- Current quantity derived from inventory movements

Leaving the SKU blank asks the backend to generate one from the steel attributes. If the preferred value already exists, a numeric suffix is added. A manually supplied duplicate is rejected.

The same physical specification from two suppliers is stored as two product rows. Their supplier-independent `spec_key` lets comparison reports group them, while stock and pricing remain independent for each supplier variant.

Use Adjust stock for opening, incoming, outgoing, damaged, or corrective movements. A movement is written to the inventory ledger; the displayed quantity is then recalculated. Product Archive preserves referenced history and hides the product from active workflows.

## Suppliers and customers

Supplier and customer profiles store contact details, tax number, opening balance, notes, status, and a calculated balance.

Supplier profiles also support a PNG, JPEG, or WebP company logo up to 2 MB. Purchasing
invoices and purchase returns place the supplier logo and identity near the top of A4 printouts;
documents use a readable initials fallback when no valid logo is available.

Statements combine the opening balance, active invoices, and active payments in date order. Archived parties can still contribute to debt totals until their balances are settled.

Customer payments are money in. Supplier payments are money out. When a payment is linked to an invoice, the backend verifies the party, direction, invoice ownership, and invoice lifecycle before accepting it.

## Purchases

A purchase invoice requires a supplier, date, and one or more positive-quantity items. It can also contain discount, tax, shipping, an initial paid amount, and notes.

Saving a purchase performs one transaction that:

- Creates the invoice and item snapshots
- Adds stock movements
- Updates product cost-price history
- Records the initial invoice payment when supplied
- Recomputes paid, remaining, and payment status
- Writes audit history

Additional installments are recorded from the invoice's Payment history action.

### Purchase returns

Use Create purchase return on an active purchase invoice. Select partial or full quantities, enter a date, and optionally add a reason and notes.

The system limits each line by both the quantity purchased and the stock currently available. A successful return reduces stock, creates a supplier credit, recalculates the purchase invoice's returned and net totals, and updates the supplier statement.

Existing returns can be printed, edited, cancelled, or restored from the return history. Editing creates a new ledger revision and supersedes the old revision. Duplicate requests are protected by an idempotency key. An invoice with an active return cannot be cancelled first; cancel its returns before cancelling the purchase.

## Sales invoices

A sale may be linked to a customer or recorded as a walk-in sale. The product picker identifies the supplier variant so the correct stock row is reduced.

Saving a completed sale snapshots item cost and selling price, reduces stock, records initial payment, calculates profit, and updates the customer balance when applicable. Additional installments are available from Payment history.

If negative stock is disabled, a sale or restoration that exceeds current stock is rejected. Cancelling a sale removes its live stock, payment, balance, dashboard, and report effects while retaining history.

## Quotations

Use Quotations when a customer wants prices before placing an order. Select or quickly create a
customer, search products by name or SKU, enter quantities and quoted unit prices, then set the
valid-until date, discount, tax/VAT, and notes.

Saving creates a Draft only. It does not reserve or reduce stock, create a sale or payment, change
customer balances, or appear in revenue and sales reports. Mark the quote Sent, then Accepted or
Rejected as the customer responds. Expired quotes are marked automatically after their valid-until
date.

An accepted, unexpired quote can be converted to a sales invoice. Conversion preserves the quoted
prices and customer/item snapshots, validates current customer/product availability and stock, and
then uses the normal sales flow. The quote is marked Converted and linked to the resulting invoice;
the same quote cannot be converted twice.

The A4 quotation printout includes the company logo/contact details, customer details, itemized
pricing, totals, notes, validity date, status, and a clear statement that it is not an invoice,
receipt, completed sale, or stock reservation.

## Expenses and payments

Expenses are grouped by seeded categories such as Rent, Electricity, Fuel, Delivery, Salary, Maintenance, Tools, Packaging, and Other. An expense can be partly paid at creation and receive later installments from Payment history.

The Payments page records general customer receipts and supplier disbursements. Payments may optionally reference an invoice. Cancelling a payment recalculates the linked invoice and party balance.

## Reports

The Reports page provides:

- Daily sales, daily profit, and monthly profit
- Stock remaining, printable stock count, stock movement, and low stock
- Cheapest supplier comparison
- Supplier sold-goods settlement detail and summary
- Purchases, supplier debt, customer debt, expenses, and payments
- Inventory value and best-selling products

Date, supplier, category, and payment-status filters appear when they apply. Tables can be sorted and exported. The physical stock-count report prints system quantity beside blank Counted Quantity and Difference fields.

Supplier settlement reports calculate the cost of a supplier's product variants sold on completed sales. The backend contains settlement-payment commands, but the current UI exposes the reports rather than a dedicated settlement-entry screen.

## Printing and exports

Invoices, quotations, and purchase returns are rendered as printable HTML using company settings and the configured currency. The print preview waits for logos and fonts before opening the system dialog. Table exports support CSV, Excel-compatible output, and PDF. Use the operating system print dialog to print or save preview content as PDF.

## Backup and restore

The application attempts one automatic backup per day when it starts. Use Backup to create an additional manual recovery point.

To restore:

1. Open Backup and choose a SQLite `.db` or `.sqlite` file.
2. Confirm the restore.
3. The application creates an emergency backup of the current database.
4. Restart the application after the restore completes.

Restoring replaces the active database. Verify the selected file and keep an independent copy before proceeding. See [OPERATIONS.md](OPERATIONS.md) for paths and recovery details.

## Clear all data

Settings contains a destructive Clear All Data action for the administrator. It requires the current administrator email/password and the exact phrase `CLEAR ALL DATA`.

The reset removes operational data and backup history in one transaction, then recreates default categories, expense categories, and the Unknown Supplier. It preserves users, company settings, and migration history. It cannot be undone unless an external database backup is restored.

## Updates

Supported macOS builds check the configured GitHub endpoint for updates. Install the app in Applications; an application launched from a DMG or a read-only/translocated path cannot replace itself. The updater explains this condition before download and offers Install and restart when the installation is writable.
