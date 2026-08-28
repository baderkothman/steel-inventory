use tauri::State;

use crate::{
    models::{
        InvoiceSaveResult, QuotationConversionPayload, QuotationDetail, QuotationFilters,
        QuotationListRow, QuotationPayload, QuotationStatusPayload,
    },
    services::quotation_service,
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn create_quotation(
    state: State<'_, AppState>,
    payload: QuotationPayload,
) -> Result<QuotationDetail, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    quotation_service::create_quotation(&conn, user_id, payload)
}

#[tauri::command]
pub fn update_quotation(
    state: State<'_, AppState>,
    id: i64,
    payload: QuotationPayload,
) -> Result<QuotationDetail, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    quotation_service::update_quotation(&conn, user_id, id, payload)
}

#[tauri::command]
pub fn get_quotation(state: State<'_, AppState>, id: i64) -> Result<QuotationDetail, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    quotation_service::get_quotation(&conn, id)
}

#[tauri::command]
pub fn list_quotations(
    state: State<'_, AppState>,
    filters: QuotationFilters,
) -> Result<Vec<QuotationListRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    quotation_service::list_quotations(&conn, filters)
}

#[tauri::command]
pub fn change_quotation_status(
    state: State<'_, AppState>,
    id: i64,
    payload: QuotationStatusPayload,
) -> Result<QuotationDetail, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    quotation_service::change_quotation_status(&conn, user_id, id, payload)
}

#[tauri::command]
pub fn delete_quotation(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    quotation_service::delete_draft_quotation(&conn, user_id, id)
}

#[tauri::command]
pub fn convert_quotation(
    state: State<'_, AppState>,
    id: i64,
    payload: QuotationConversionPayload,
) -> Result<InvoiceSaveResult, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    quotation_service::convert_quotation(&conn, user_id, id, payload)
}

#[tauri::command]
pub fn print_quotation(state: State<'_, AppState>, id: i64) -> Result<String, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    quotation_service::quotation_html(&conn, state.db_path(), id)
}
