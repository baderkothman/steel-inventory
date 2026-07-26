use tauri::State;

use crate::{
    models::{InstallmentPaymentPayload, InstallmentPaymentRow},
    services::invoice_payment_service,
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn list_invoice_payments(
    state: State<'_, AppState>,
    kind: String,
    invoice_id: i64,
) -> Result<Vec<InstallmentPaymentRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    invoice_payment_service::list_invoice_payments(&conn, &kind, invoice_id)
}

#[tauri::command]
pub fn record_invoice_payment(
    state: State<'_, AppState>,
    kind: String,
    invoice_id: i64,
    payload: InstallmentPaymentPayload,
) -> Result<InstallmentPaymentRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    invoice_payment_service::record_invoice_payment(&conn, user_id, &kind, invoice_id, payload)
}
