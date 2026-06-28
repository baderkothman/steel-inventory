use tauri::State;

use crate::{
    models::{InvoiceDetail, InvoiceFilters, InvoiceListRow, InvoiceSaveResult, SalesInvoicePayload},
    services::sales_service,
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn create_sales_invoice(state: State<'_, AppState>, payload: SalesInvoicePayload) -> Result<InvoiceSaveResult, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    sales_service::create_sales_invoice(&conn, user_id, payload)
}

#[tauri::command]
pub fn cancel_sales_invoice(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    sales_service::cancel_sales_invoice(&conn, user_id, id)
}

#[tauri::command]
pub fn get_sales_invoice(state: State<'_, AppState>, id: i64) -> Result<InvoiceDetail, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    sales_service::get_sales_invoice(&conn, id)
}

#[tauri::command]
pub fn list_sales_invoices(state: State<'_, AppState>, filters: InvoiceFilters) -> Result<Vec<InvoiceListRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    sales_service::list_sales_invoices(&conn, filters)
}

#[tauri::command]
pub fn print_sales_invoice(state: State<'_, AppState>, id: i64) -> Result<String, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    sales_service::sales_invoice_html(&conn, id)
}
