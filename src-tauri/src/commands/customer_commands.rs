use tauri::State;

use crate::{
    models::{DateRangeFilters, PartyFilters, PartyPayload, PartyRow, StatementRow},
    services::party_service::{self, PartyKind},
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn create_customer(state: State<'_, AppState>, payload: PartyPayload) -> Result<PartyRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::create_party(&conn, user_id, PartyKind::Customer, payload)
}

#[tauri::command]
pub fn update_customer(state: State<'_, AppState>, id: i64, payload: PartyPayload) -> Result<PartyRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::update_party(&conn, user_id, PartyKind::Customer, id, payload)
}

#[tauri::command]
pub fn archive_customer(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::archive_party(&conn, user_id, PartyKind::Customer, id)
}

#[tauri::command]
pub fn get_customer(state: State<'_, AppState>, id: i64) -> Result<PartyRow, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::get_party(&conn, PartyKind::Customer, id)
}

#[tauri::command]
pub fn list_customers(state: State<'_, AppState>, filters: PartyFilters) -> Result<Vec<PartyRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::list_parties(&conn, PartyKind::Customer, filters)
}

#[tauri::command]
pub fn get_customer_statement(state: State<'_, AppState>, customer_id: i64, filters: DateRangeFilters) -> Result<Vec<StatementRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::statement(&conn, PartyKind::Customer, customer_id, filters)
}
