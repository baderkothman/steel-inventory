use rusqlite::params;
use tauri::State;

use crate::{
    models::{
        DateRangeFilters, LogoUploadPayload, PartyFilters, PartyPayload, PartyRow, StatementRow,
    },
    services::{
        logo_service,
        party_service::{self, PartyKind},
    },
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn save_supplier_logo(
    state: State<'_, AppState>,
    id: i64,
    payload: LogoUploadPayload,
) -> Result<PartyRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    let supplier = party_service::get_party(&conn, PartyKind::Supplier, id)?;
    let path = logo_service::save_logo(state.db_path(), "suppliers", &id.to_string(), &payload)?;
    conn.execute(
        "UPDATE suppliers SET logo_path = ?1, updated_at = ?2 WHERE id = ?3",
        params![path, crate::utils::dates::now_iso(), id],
    )?;
    crate::utils::audit::insert_audit_log(
        &conn,
        user_id,
        "update_logo",
        "suppliers",
        id,
        supplier.logo_path.map(serde_json::Value::String),
        Some(serde_json::Value::String(path)),
    )?;
    party_service::get_party(&conn, PartyKind::Supplier, id)
}

#[tauri::command]
pub fn get_supplier_logo(state: State<'_, AppState>, id: i64) -> Result<Option<String>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    let supplier = party_service::get_party(&conn, PartyKind::Supplier, id)?;
    Ok(logo_service::logo_data_uri(
        state.db_path(),
        supplier.logo_path.as_deref(),
    ))
}

#[tauri::command]
pub fn remove_supplier_logo(state: State<'_, AppState>, id: i64) -> Result<PartyRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    let supplier = party_service::get_party(&conn, PartyKind::Supplier, id)?;
    logo_service::remove_logo(state.db_path(), supplier.logo_path.as_deref())?;
    conn.execute(
        "UPDATE suppliers SET logo_path = NULL, updated_at = ?1 WHERE id = ?2",
        params![crate::utils::dates::now_iso(), id],
    )?;
    crate::utils::audit::insert_audit_log(
        &conn,
        user_id,
        "remove_logo",
        "suppliers",
        id,
        None,
        None,
    )?;
    party_service::get_party(&conn, PartyKind::Supplier, id)
}

#[tauri::command]
pub fn create_supplier(
    state: State<'_, AppState>,
    payload: PartyPayload,
) -> Result<PartyRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::create_party(&conn, user_id, PartyKind::Supplier, payload)
}

#[tauri::command]
pub fn update_supplier(
    state: State<'_, AppState>,
    id: i64,
    payload: PartyPayload,
) -> Result<PartyRow, AppError> {
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
pub fn restore_supplier(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::restore_party(&conn, user_id, PartyKind::Supplier, id)
}

#[tauri::command]
pub fn delete_supplier(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::delete_party(&conn, user_id, PartyKind::Supplier, id)
}

#[tauri::command]
pub fn get_supplier(state: State<'_, AppState>, id: i64) -> Result<PartyRow, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::get_party(&conn, PartyKind::Supplier, id)
}

#[tauri::command]
pub fn list_suppliers(
    state: State<'_, AppState>,
    filters: PartyFilters,
) -> Result<Vec<PartyRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::list_parties(&conn, PartyKind::Supplier, filters)
}

#[tauri::command]
pub fn get_supplier_statement(
    state: State<'_, AppState>,
    supplier_id: i64,
    filters: DateRangeFilters,
) -> Result<Vec<StatementRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    party_service::statement(&conn, PartyKind::Supplier, supplier_id, filters)
}
