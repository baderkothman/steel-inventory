use serde_json::Value;
use tauri::State;

use crate::{
    models::{DashboardSummary, ReportFilters},
    services::report_service,
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn get_dashboard_summary(state: State<'_, AppState>, date: Option<String>) -> Result<DashboardSummary, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::dashboard_summary(&conn, date)
}

#[tauri::command]
pub fn get_daily_sales_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::daily_sales_report(&conn, filters)
}

#[tauri::command]
pub fn get_profit_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::profit_report(&conn, filters)
}

#[tauri::command]
pub fn get_monthly_profit_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::monthly_profit_report(&conn, filters)
}

#[tauri::command]
pub fn get_stock_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::stock_report(&conn, filters)
}

#[tauri::command]
pub fn get_stock_movement_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::stock_movement_report(&conn, filters)
}

#[tauri::command]
pub fn get_low_stock_report(state: State<'_, AppState>) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::low_stock_report(&conn)
}

#[tauri::command]
pub fn get_customer_debt_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::customer_debt_report(&conn, filters)
}

#[tauri::command]
pub fn get_supplier_debt_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::supplier_debt_report(&conn, filters)
}

#[tauri::command]
pub fn get_expense_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::expense_report(&conn, filters)
}

#[tauri::command]
pub fn get_purchase_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::purchase_report(&conn, filters)
}

#[tauri::command]
pub fn get_payment_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::payment_report(&conn, filters)
}

#[tauri::command]
pub fn get_inventory_value_report(state: State<'_, AppState>) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::inventory_value_report(&conn)
}

#[tauri::command]
pub fn get_best_selling_products_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::best_selling_products_report(&conn, filters)
}

#[tauri::command]
pub fn get_stock_count_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::stock_count_report(&conn, filters)
}

#[tauri::command]
pub fn get_cheapest_supplier_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::cheapest_supplier_report(&conn, filters)
}

#[tauri::command]
pub fn get_supplier_settlement_report(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::supplier_settlement_report(&conn, filters)
}

#[tauri::command]
pub fn get_supplier_settlement_summary(state: State<'_, AppState>, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    report_service::supplier_settlement_summary(&conn, filters)
}
