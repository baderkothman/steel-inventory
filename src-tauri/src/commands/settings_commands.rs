use tauri::State;

use crate::{
    models::{
        ClearAllDataPayload, ClearAllDataResult, CompanySettings, CompanySettingsPayload,
    },
    services::{data_lifecycle_service, settings_service},
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn get_company_settings(state: State<'_, AppState>) -> Result<CompanySettings, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    settings_service::get_company_settings(&conn)
}

#[tauri::command]
pub fn update_company_settings(state: State<'_, AppState>, payload: CompanySettingsPayload) -> Result<CompanySettings, AppError> {
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
