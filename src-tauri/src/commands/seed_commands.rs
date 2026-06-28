use tauri::State;

use crate::{
    models::DemoSeedResult,
    services::seed_service,
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn seed_demo_data(state: State<'_, AppState>) -> Result<DemoSeedResult, AppError> {
    let user_id = state.require_user_id()?;
    let mut conn = state.open_conn()?;
    seed_service::seed_demo_data(&mut conn, user_id)
}
