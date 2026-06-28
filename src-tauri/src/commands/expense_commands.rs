use tauri::State;

use crate::{
    models::{ExpenseCategory, ExpenseFilters, ExpensePayload, ExpenseRow},
    services::expense_service,
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn list_expense_categories(state: State<'_, AppState>) -> Result<Vec<ExpenseCategory>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    expense_service::list_expense_categories(&conn)
}

#[tauri::command]
pub fn create_expense(state: State<'_, AppState>, payload: ExpensePayload) -> Result<ExpenseRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    expense_service::create_expense(&conn, user_id, payload)
}

#[tauri::command]
pub fn update_expense(state: State<'_, AppState>, id: i64, payload: ExpensePayload) -> Result<ExpenseRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    expense_service::update_expense(&conn, user_id, id, payload)
}

#[tauri::command]
pub fn delete_expense(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    expense_service::delete_expense(&conn, user_id, id)
}

#[tauri::command]
pub fn list_expenses(state: State<'_, AppState>, filters: ExpenseFilters) -> Result<Vec<ExpenseRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    expense_service::list_expenses(&conn, filters)
}
