use tauri::State;

use crate::{
    models::{CompanySettings, CompanySettingsPayload},
    services::settings_service,
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
