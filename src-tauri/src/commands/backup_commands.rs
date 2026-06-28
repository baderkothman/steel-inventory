use tauri::State;

use crate::{
    models::BackupRow,
    services::backup_service,
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn create_manual_backup(state: State<'_, AppState>) -> Result<BackupRow, AppError> {
    let user_id = state.require_user_id()?;
    backup_service::create_manual_backup(state.db_path(), user_id)
}

#[tauri::command]
pub fn restore_backup(state: State<'_, AppState>, path: String) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    backup_service::restore_backup(state.db_path(), user_id, path)
}

#[tauri::command]
pub fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    backup_service::list_backups(&conn)
}
