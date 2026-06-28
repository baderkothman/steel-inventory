use tauri::State;

use crate::{
    models::{AdminUser, ChangePasswordPayload, LoginPayload, SetupAdminPayload},
    services::auth_service,
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn has_admin(state: State<'_, AppState>) -> Result<bool, AppError> {
    let conn = state.open_conn()?;
    auth_service::has_admin(&conn)
}

#[tauri::command]
pub fn setup_admin(
    state: State<'_, AppState>,
    payload: SetupAdminPayload,
) -> Result<AdminUser, AppError> {
    let conn = state.open_conn()?;
    let user = auth_service::setup_admin(&conn, payload)?;
    state.set_session(user.clone())?;
    Ok(user)
}

#[tauri::command]
pub fn login_admin(
    state: State<'_, AppState>,
    payload: LoginPayload,
) -> Result<AdminUser, AppError> {
    let conn = state.open_conn()?;
    let user = auth_service::login_admin(&conn, payload)?;
    state.set_session(user.clone())?;
    Ok(user)
}

#[tauri::command]
pub fn logout_admin(state: State<'_, AppState>) -> Result<(), AppError> {
    state.clear_session()
}

#[tauri::command]
pub fn change_admin_password(
    state: State<'_, AppState>,
    payload: ChangePasswordPayload,
) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    auth_service::change_password(&conn, user_id, payload)
}

#[tauri::command]
pub fn get_current_admin(state: State<'_, AppState>) -> Result<Option<AdminUser>, AppError> {
    state.current_user()
}
