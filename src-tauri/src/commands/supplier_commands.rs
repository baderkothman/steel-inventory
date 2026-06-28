use tauri::State;

use crate::{
    models::{DateRangeFilters, PartyFilters, PartyPayload, PartyRow, StatementRow},
    services::party_service::{self, PartyKind},
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn create_supplier(state: State<'_, AppState>, payload: PartyPayload) -> Result<PartyRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::create_party(&conn, user_id, PartyKind::Supplier, payload)
}

#[tauri::command]
pub fn update_supplier(state: State<'_, AppState>, id: i64, payload: PartyPayload) -> Result<PartyRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::update_party(&conn, user_id, PartyKind::Supplier, id, payload)
}

#[tauri::command]
pub fn archive_supplier(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::archive_party(&conn, user_id, PartyKind::Supplier, id)
}

#[tauri::command]
pub fn get_supplier(state: State<'_, AppState>, id: i64) -> Result<PartyRow, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::get_party(&conn, PartyKind::Supplier, id)
}

#[tauri::command]
pub fn list_suppliers(state: State<'_, AppState>, filters: PartyFilters) -> Result<Vec<PartyRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::list_parties(&conn, PartyKind::Supplier, filters)
}

#[tauri::command]
pub fn get_supplier_statement(state: State<'_, AppState>, supplier_id: i64, filters: DateRangeFilters) -> Result<Vec<StatementRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::statement(&conn, PartyKind::Supplier, supplier_id, filters)
}
