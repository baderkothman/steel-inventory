use tauri::State;

use crate::{
    models::{
        PurchaseReturnContext, PurchaseReturnDetail, PurchaseReturnPayload,
        PurchaseReturnUpdatePayload,
    },
    services::purchase_return_service,
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn get_purchase_return_context(
    state: State<'_, AppState>,
    purchase_invoice_id: i64,
) -> Result<PurchaseReturnContext, AppError> {
    state.require_admin_id()?;
    let conn = state.open_conn()?;
    purchase_return_service::get_purchase_return_context(&conn, purchase_invoice_id)
}

#[tauri::command]
pub fn get_purchase_return(
    state: State<'_, AppState>,
    id: i64,
) -> Result<PurchaseReturnDetail, AppError> {
    state.require_admin_id()?;
    let conn = state.open_conn()?;
    purchase_return_service::get_purchase_return(&conn, id)
}

#[tauri::command]
pub fn create_purchase_return(
    state: State<'_, AppState>,
    payload: PurchaseReturnPayload,
) -> Result<PurchaseReturnDetail, AppError> {
    let user_id = state.require_admin_id()?;
    let conn = state.open_conn()?;
    purchase_return_service::create_purchase_return(&conn, user_id, payload)
}

#[tauri::command]
pub fn update_purchase_return(
    state: State<'_, AppState>,
    id: i64,
    payload: PurchaseReturnUpdatePayload,
) -> Result<PurchaseReturnDetail, AppError> {
    let user_id = state.require_admin_id()?;
    let conn = state.open_conn()?;
    purchase_return_service::update_purchase_return(&conn, user_id, id, payload)
}

#[tauri::command]
pub fn cancel_purchase_return(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_admin_id()?;
    let conn = state.open_conn()?;
    purchase_return_service::cancel_purchase_return(&conn, user_id, id)
}

#[tauri::command]
pub fn restore_purchase_return(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_admin_id()?;
    let conn = state.open_conn()?;
    purchase_return_service::restore_purchase_return(&conn, user_id, id)
}

#[tauri::command]
pub fn print_purchase_return(state: State<'_, AppState>, id: i64) -> Result<String, AppError> {
    state.require_admin_id()?;
    let conn = state.open_conn()?;
    purchase_return_service::purchase_return_html(&conn, state.db_path(), id)
}
