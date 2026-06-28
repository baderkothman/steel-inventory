use tauri::State;

use crate::{
    models::{PaymentFilters, PaymentPayload, PaymentRow},
    services::payment_service,
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn create_payment(state: State<'_, AppState>, payload: PaymentPayload) -> Result<PaymentRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    payment_service::create_payment(&conn, user_id, payload)
}

#[tauri::command]
pub fn delete_payment(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    payment_service::delete_payment(&conn, user_id, id)
}

#[tauri::command]
pub fn list_payments(state: State<'_, AppState>, filters: PaymentFilters) -> Result<Vec<PaymentRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    payment_service::list_payments(&conn, filters)
}
