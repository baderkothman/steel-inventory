mod commands;
mod db;
mod models;
mod services;
mod state;
mod utils;

#[cfg(test)]
mod tests;

use commands::{
    auth_commands::*,
    backup_commands::*,
    category_commands::*,
    customer_commands::*,
    expense_commands::*,
    payment_commands::*,
    product_commands::*,
    purchase_commands::*,
    report_commands::*,
    sales_commands::*,
    seed_commands::*,
    settings_commands::*,
    supplier_commands::*,
};
use state::AppState;

pub fn run() {
    let app_state = AppState::initialize().expect("failed to initialize application state");
    let _ = services::backup_service::create_automatic_backup_if_due(app_state.db_path());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            has_admin,
            setup_admin,
            login_admin,
            logout_admin,
            change_admin_password,
            get_current_admin,
            create_category,
            update_category,
            archive_category,
            list_categories,
            create_product,
            update_product,
            archive_product,
            get_product,
            list_products,
            generate_product_sku,
            get_product_stock,
            get_product_movement,
            adjust_stock,
            create_supplier,
            update_supplier,
            archive_supplier,
            get_supplier,
            list_suppliers,
            get_supplier_statement,
            create_customer,
            update_customer,
            archive_customer,
            get_customer,
            list_customers,
            get_customer_statement,
            create_purchase_invoice,
            cancel_purchase_invoice,
            get_purchase_invoice,
            list_purchase_invoices,
            print_purchase_invoice,
            create_sales_invoice,
            cancel_sales_invoice,
            get_sales_invoice,
            list_sales_invoices,
            print_sales_invoice,
            seed_demo_data,
            list_expense_categories,
            create_expense,
            update_expense,
            delete_expense,
            list_expenses,
            create_payment,
            delete_payment,
            list_payments,
            get_dashboard_summary,
            get_daily_sales_report,
            get_profit_report,
            get_monthly_profit_report,
            get_stock_report,
            get_stock_movement_report,
            get_low_stock_report,
            get_customer_debt_report,
            get_supplier_debt_report,
            get_expense_report,
            get_purchase_report,
            get_payment_report,
            get_inventory_value_report,
            get_best_selling_products_report,
            get_company_settings,
            update_company_settings,
            create_manual_backup,
            restore_backup,
            list_backups
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
