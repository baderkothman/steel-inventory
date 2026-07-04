use tauri::State;

use crate::{
    models::{SettlementFilters, SettlementPaymentPayload, SettlementPaymentRow},
    services::settlement_service,
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn create_settlement_payment(
    state: State<'_, AppState>,
    payload: SettlementPaymentPayload,
) -> Result<SettlementPaymentRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    settlement_service::create_settlement_payment(&conn, user_id, payload)
}

#[tauri::command]
pub fn delete_settlement_payment(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    settlement_service::delete_settlement_payment(&conn, user_id, id)
}

#[tauri::command]
pub fn list_settlement_payments(
    state: State<'_, AppState>,
    filters: SettlementFilters,
) -> Result<Vec<SettlementPaymentRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    settlement_service::list_settlement_payments(&conn, filters)
}
