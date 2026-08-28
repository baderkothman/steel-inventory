use rusqlite::params;
use tauri::State;

use crate::{
    models::{
        ClearAllDataPayload, ClearAllDataResult, CompanySettings, CompanySettingsPayload,
        LogoUploadPayload,
    },
    services::{data_lifecycle_service, logo_service, settings_service},
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn save_company_logo(
    state: State<'_, AppState>,
    payload: LogoUploadPayload,
) -> Result<CompanySettings, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    let settings = settings_service::get_company_settings(&conn)?;
    let path = logo_service::save_logo(state.db_path(), "company", "logo", &payload)?;
    conn.execute(
        "UPDATE company_settings SET logo_path = ?1, updated_at = ?2 WHERE id = 1",
        params![path, crate::utils::dates::now_iso()],
    )?;
    crate::utils::audit::insert_audit_log(
        &conn,
        user_id,
        "update_logo",
        "company_settings",
        1,
        settings.logo_path.map(serde_json::Value::String),
        Some(serde_json::Value::String(path)),
    )?;
    settings_service::get_company_settings(&conn)
}

#[tauri::command]
pub fn get_company_logo(state: State<'_, AppState>) -> Result<Option<String>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    let settings = settings_service::get_company_settings(&conn)?;
    Ok(logo_service::logo_data_uri(
        state.db_path(),
        settings.logo_path.as_deref(),
    ))
}

#[tauri::command]
pub fn remove_company_logo(state: State<'_, AppState>) -> Result<CompanySettings, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    let settings = settings_service::get_company_settings(&conn)?;
    logo_service::remove_logo(state.db_path(), settings.logo_path.as_deref())?;
    conn.execute(
        "UPDATE company_settings SET logo_path = NULL, updated_at = ?1 WHERE id = 1",
        [crate::utils::dates::now_iso()],
    )?;
    crate::utils::audit::insert_audit_log(
        &conn,
        user_id,
        "remove_logo",
        "company_settings",
        1,
        None,
        None,
    )?;
    settings_service::get_company_settings(&conn)
}

#[tauri::command]
pub fn get_company_settings(state: State<'_, AppState>) -> Result<CompanySettings, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    settings_service::get_company_settings(&conn)
}

#[tauri::command]
pub fn update_company_settings(
    state: State<'_, AppState>,
    payload: CompanySettingsPayload,
) -> Result<CompanySettings, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    settings_service::update_company_settings(&conn, user_id, payload)
}

#[tauri::command]
pub fn clear_all_data(
    state: State<'_, AppState>,
    payload: ClearAllDataPayload,
) -> Result<ClearAllDataResult, AppError> {
    let user = state.require_user()?;
    if user.role != "admin" {
        return Err(AppError::unauthorized());
    }
    let conn = state.open_conn()?;
    data_lifecycle_service::clear_all_data(&conn, user.id, payload)
}
