use tauri::State;

use crate::{
    models::{InvoiceDetail, InvoiceFilters, InvoiceListRow, InvoiceSaveResult, PurchaseInvoicePayload},
    services::purchase_service,
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn create_purchase_invoice(state: State<'_, AppState>, payload: PurchaseInvoicePayload) -> Result<InvoiceSaveResult, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    purchase_service::create_purchase_invoice(&conn, user_id, payload)
}

#[tauri::command]
pub fn cancel_purchase_invoice(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    purchase_service::cancel_purchase_invoice(&conn, user_id, id)
}

#[tauri::command]
pub fn get_purchase_invoice(state: State<'_, AppState>, id: i64) -> Result<InvoiceDetail, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    purchase_service::get_purchase_invoice(&conn, id)
}

#[tauri::command]
pub fn list_purchase_invoices(state: State<'_, AppState>, filters: InvoiceFilters) -> Result<Vec<InvoiceListRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    purchase_service::list_purchase_invoices(&conn, filters)
}

#[tauri::command]
pub fn print_purchase_invoice(state: State<'_, AppState>, id: i64) -> Result<String, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    purchase_service::purchase_invoice_html(&conn, id)
}
