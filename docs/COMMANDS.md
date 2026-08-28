# Command reference

The registered Tauri command surface is defined in `src-tauri/src/lib.rs`. The typed frontend facade is `src/lib/api.ts`.

## Calling convention

Use the domain facade instead of calling `invoke` from feature components. `src/lib/tauri.ts` converts top-level snake_case argument names to the camelCase names exposed by Tauri. Nested payload and filter objects must keep the snake_case field names declared by the Rust Serde models.

Every rejected call returns a structured `{ code, message }` error. `UNAUTHORIZED` also clears the frontend session.

## Authentication

- `has_admin`
- `setup_admin`
- `login_admin`
- `logout_admin`
- `change_admin_password`
- `get_current_admin`

Setup is allowed only when no active administrator exists. Business commands require the current session.

## Categories

- `create_category`
- `update_category`
- `archive_category`
- `restore_category`
- `delete_category`
- `list_categories`

Archive/restore changes lifecycle state. Permanent deletion is restricted by referenced children or products.

## Products and stock

- `create_product`
- `update_product`
- `archive_product`
- `restore_product`
- `delete_product`
- `get_product`
- `list_products`
- `generate_product_sku`
- `get_product_stock`
- `get_product_movement`
- `list_supplier_variants`
- `adjust_stock`
- `cancel_stock_adjustment`
- `restore_stock_adjustment`
- `permanently_delete_stock_adjustment`

Product filters support search, category, supplier, and active-only state. Supplier-variant filters support search, category, and in-stock-only state.

## Suppliers and customers

Suppliers:

- `create_supplier`, `update_supplier`, `archive_supplier`, `restore_supplier`, `delete_supplier`
- `get_supplier`, `list_suppliers`, `get_supplier_statement`
- `save_supplier_logo`, `get_supplier_logo`, `remove_supplier_logo`

Customers:

- `create_customer`, `update_customer`, `archive_customer`, `restore_customer`, `delete_customer`
- `get_customer`, `list_customers`, `get_customer_statement`

The two command groups share the same party service while retaining separate database tables and payment direction rules.

## Purchases

- `create_purchase_invoice`
- `cancel_purchase_invoice`
- `restore_purchase_invoice`
- `permanently_delete_purchase_invoice`
- `get_purchase_invoice`
- `list_purchase_invoices`
- `print_purchase_invoice`

## Purchase returns

- `get_purchase_return_context`
- `get_purchase_return`
- `create_purchase_return`
- `update_purchase_return`
- `cancel_purchase_return`
- `restore_purchase_return`
- `print_purchase_return`

Return mutation commands require an administrator session. Create payloads include an idempotency key; update payloads include the expected revision.

## Sales

- `create_sales_invoice`
- `cancel_sales_invoice`
- `restore_sales_invoice`
- `permanently_delete_sales_invoice`
- `get_sales_invoice`
- `list_sales_invoices`
- `print_sales_invoice`

## Quotations

- `create_quotation`
- `update_quotation`
- `get_quotation`
- `list_quotations`
- `change_quotation_status`
- `delete_quotation`
- `convert_quotation`
- `print_quotation`

Draft quotations snapshot customer/product labels and quoted prices without creating stock,
sales, payment, report, or accounting effects. Conversion accepts only an accepted,
unexpired, unconverted quotation and creates the sales invoice plus quotation link in one
transaction using the normal sales stock validation.

## Invoice and expense installments

- `list_invoice_payments`
- `record_invoice_payment`
- `list_expense_payments`
- `record_expense_payment`

Invoice-payment commands accept `kind` as `purchase` or `sales` and an invoice ID.

## Expenses

- `list_expense_categories`
- `create_expense`
- `update_expense`
- `delete_expense`
- `restore_expense`
- `permanently_delete_expense`
- `list_expenses`

The command named `delete_expense` performs lifecycle cancellation; permanent deletion is a separate command.

## General payments

- `create_payment`
- `delete_payment`
- `restore_payment`
- `permanently_delete_payment`
- `list_payments`

The service validates party type, payment direction, and any invoice link both in Rust and with SQLite triggers.

## Supplier settlements

- `create_settlement_payment`
- `delete_settlement_payment`
- `restore_settlement_payment`
- `permanently_delete_settlement_payment`
- `list_settlement_payments`
- `get_supplier_settlement_report`
- `get_supplier_settlement_summary`

The settlement-payment commands are exposed by the Rust backend, but the frontend has no `settlementApi` binding and no dedicated settlement-entry page.

## Dashboard and reports

- `get_dashboard_summary`
- `get_daily_sales_report`
- `get_profit_report`
- `get_monthly_profit_report`
- `get_stock_report`
- `get_stock_movement_report`
- `get_low_stock_report`
- `get_customer_debt_report`
- `get_supplier_debt_report`
- `get_expense_report`
- `get_purchase_report`
- `get_payment_report`
- `get_inventory_value_report`
- `get_best_selling_products_report`
- `get_stock_count_report`
- `get_cheapest_supplier_report`
- `get_supplier_settlement_report`
- `get_supplier_settlement_summary`

Most report commands accept `ReportFilters` with an optional date range, supplier, category, or payment status. Inventory-value and low-stock reports do not require date filters.

## Settings and data lifecycle

- `get_company_settings`
- `update_company_settings`
- `save_company_logo`, `get_company_logo`, `remove_company_logo`
- `clear_all_data`

`clear_all_data` requires the signed-in administrator's email/password and exact confirmation text.

## Backup

- `create_manual_backup`
- `restore_backup`
- `list_backups`

Restore creates an emergency backup before replacing the active database. The application must be restarted afterward.
