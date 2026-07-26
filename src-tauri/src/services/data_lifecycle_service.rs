use rusqlite::Connection;

use crate::{
    models::{ClearAllDataPayload, ClearAllDataResult},
    services::auth_service::verify_admin_credentials,
    utils::{audit::insert_audit_log, errors::AppError, validation::required},
};

const DEFAULT_DATA_SQL: &str = r#"
INSERT INTO expense_categories (name, description, is_active, created_at, updated_at) VALUES
('Rent', 'Office, warehouse, or shop rent', 1, datetime('now'), datetime('now')),
('Electricity', 'Electricity and utilities', 1, datetime('now'), datetime('now')),
('Fuel', 'Fuel and vehicle expenses', 1, datetime('now'), datetime('now')),
('Delivery', 'Delivery and transportation expenses', 1, datetime('now'), datetime('now')),
('Salary', 'Staff salary payments', 1, datetime('now'), datetime('now')),
('Maintenance', 'General maintenance expenses', 1, datetime('now'), datetime('now')),
('Tools', 'Tools and equipment expenses', 1, datetime('now'), datetime('now')),
('Packaging', 'Packaging material expenses', 1, datetime('now'), datetime('now')),
('Other', 'Other business expenses', 1, datetime('now'), datetime('now'));

INSERT INTO categories (id, name, parent_id, description, is_active, created_at, updated_at) VALUES
(1, 'Steel Products', NULL, 'Steel inventory root category', 1, datetime('now'), datetime('now')),
(2, 'Pipes', 1, NULL, 1, datetime('now'), datetime('now')),
(3, 'Galvanized Pipes', 2, NULL, 1, datetime('now'), datetime('now')),
(4, 'Square Pipes', 3, NULL, 1, datetime('now'), datetime('now')),
(5, 'Rectangular Pipes', 3, NULL, 1, datetime('now'), datetime('now')),
(6, 'Round Pipes', 3, NULL, 1, datetime('now'), datetime('now')),
(7, 'Black Pipes', 2, NULL, 1, datetime('now'), datetime('now')),
(8, 'Sheets / Plates', 1, NULL, 1, datetime('now'), datetime('now')),
(9, 'Galvanized Sheets', 8, NULL, 1, datetime('now'), datetime('now')),
(10, 'Black Sheets', 8, NULL, 1, datetime('now'), datetime('now')),
(11, 'Stainless Steel Sheets', 8, NULL, 1, datetime('now'), datetime('now')),
(12, 'Bars', 1, NULL, 1, datetime('now'), datetime('now')),
(13, 'Flat Bars', 12, NULL, 1, datetime('now'), datetime('now')),
(14, 'Round Bars', 12, NULL, 1, datetime('now'), datetime('now')),
(15, 'Square Bars', 12, NULL, 1, datetime('now'), datetime('now')),
(16, 'Angles / Channels / Beams', 1, NULL, 1, datetime('now'), datetime('now')),
(17, 'Angle Bar', 16, NULL, 1, datetime('now'), datetime('now')),
(18, 'U Channel', 16, NULL, 1, datetime('now'), datetime('now')),
(19, 'I Beam', 16, NULL, 1, datetime('now'), datetime('now')),
(20, 'H Beam', 16, NULL, 1, datetime('now'), datetime('now')),
(21, 'Rebar', 1, NULL, 1, datetime('now'), datetime('now')),
(22, 'Accessories', 1, NULL, 1, datetime('now'), datetime('now')),
(23, 'Equipment', 1, NULL, 1, datetime('now'), datetime('now'));

INSERT INTO suppliers
(name, company_name, opening_balance_cents, notes, is_active, created_at, updated_at)
VALUES
('Unknown Supplier', NULL, 0, 'System fallback supplier.', 1, datetime('now'), datetime('now'));
"#;

const BUSINESS_TABLES: &[&str] = &[
    "supplier_settlement_payments",
    "expense_payments",
    "walk_in_sales_payments",
    "payments",
    "expenses",
    "sales_invoice_items",
    "purchase_invoice_items",
    "inventory_transactions",
    "stock_levels",
    "product_prices",
    "sales_invoices",
    "purchase_invoices",
    "products",
    "customers",
    "suppliers",
    "categories",
    "expense_categories",
    "backups",
    "audit_logs",
];

pub fn clear_all_data(
    conn: &Connection,
    user_id: i64,
    payload: ClearAllDataPayload,
) -> Result<ClearAllDataResult, AppError> {
    required(&payload.confirmation, "Final confirmation")?;
    if payload.confirmation != "CLEAR ALL DATA" {
        return Err(AppError::validation(
            "Type CLEAR ALL DATA exactly to confirm the reset.",
        ));
    }
    verify_admin_credentials(conn, user_id, &payload.admin_email, &payload.admin_password)?;

    let mut deleted_records = 0_i64;
    for table in BUSINESS_TABLES {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        deleted_records += conn.query_row(&sql, [], |row| row.get::<_, i64>(0))?;
    }

    let tx = conn.unchecked_transaction()?;
    tx.execute_batch(
        "DELETE FROM supplier_settlement_payments;
         DELETE FROM expense_payments;
         DELETE FROM walk_in_sales_payments;
         DELETE FROM payments;
         DELETE FROM expenses;
         DELETE FROM sales_invoice_items;
         DELETE FROM purchase_invoice_items;
         DELETE FROM inventory_transactions;
         DELETE FROM stock_levels;
         DELETE FROM product_prices;
         DELETE FROM sales_invoices;
         DELETE FROM purchase_invoices;
         DELETE FROM products;
         DELETE FROM customers;
         DELETE FROM suppliers;
         UPDATE categories SET parent_id = NULL;
         DELETE FROM categories;
         DELETE FROM expense_categories;
         DELETE FROM backups;
         DELETE FROM audit_logs;
         DELETE FROM sqlite_sequence
         WHERE name IN (
             'supplier_settlement_payments', 'expense_payments',
             'walk_in_sales_payments', 'payments', 'expenses',
             'sales_invoice_items', 'purchase_invoice_items', 'inventory_transactions',
             'stock_levels', 'product_prices', 'sales_invoices', 'purchase_invoices',
             'products', 'customers', 'suppliers', 'categories', 'expense_categories',
             'backups', 'audit_logs'
         );",
    )?;
    tx.execute_batch(DEFAULT_DATA_SQL)?;
    insert_audit_log(
        &tx,
        user_id,
        "clear_all_data",
        "system",
        0,
        None,
        Some(serde_json::json!({
            "deleted_records": deleted_records,
            "preserved": ["users", "company_settings", "schema_migrations"]
        })),
    )?;
    tx.commit()?;

    Ok(ClearAllDataResult {
        deleted_records,
        message:
            "All business data was cleared. Administrator and company settings were preserved."
                .to_string(),
    })
}
