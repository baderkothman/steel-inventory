use tauri::State;

use crate::{
    models::{Category, CategoryPayload},
    services::category_service,
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn create_category(
    state: State<'_, AppState>,
    payload: CategoryPayload,
) -> Result<Category, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    category_service::create_category(&conn, user_id, payload)
}

#[tauri::command]
pub fn update_category(
    state: State<'_, AppState>,
    id: i64,
    payload: CategoryPayload,
) -> Result<Category, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    category_service::update_category(&conn, user_id, id, payload)
}

#[tauri::command]
pub fn archive_category(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    category_service::archive_category(&conn, user_id, id)
}

#[tauri::command]
pub fn list_categories(state: State<'_, AppState>) -> Result<Vec<Category>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    category_service::list_categories(&conn)
}
