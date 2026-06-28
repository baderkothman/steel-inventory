use tauri::State;

use crate::{
    models::{InventoryTransactionRow, MovementFilters, ProductFilters, ProductPayload, ProductRow, StockAdjustmentPayload},
    services::{inventory_service, product_service},
    state::AppState,
    utils::errors::AppError,
};

#[tauri::command]
pub fn create_product(
    state: State<'_, AppState>,
    payload: ProductPayload,
) -> Result<ProductRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    product_service::create_product(&conn, user_id, payload)
}

#[tauri::command]
pub fn update_product(
    state: State<'_, AppState>,
    id: i64,
    payload: ProductPayload,
) -> Result<ProductRow, AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    product_service::update_product(&conn, user_id, id, payload)
}

#[tauri::command]
pub fn archive_product(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    product_service::archive_product(&conn, user_id, id)
}

#[tauri::command]
pub fn get_product(state: State<'_, AppState>, id: i64) -> Result<ProductRow, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    product_service::get_product(&conn, id)
}

#[tauri::command]
pub fn list_products(
    state: State<'_, AppState>,
    filters: ProductFilters,
) -> Result<Vec<ProductRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    product_service::list_products(&conn, filters)
}

#[tauri::command]
pub fn generate_product_sku(payload: ProductPayload) -> Result<String, AppError> {
    product_service::generate_sku(payload)
}

#[tauri::command]
pub fn get_product_stock(state: State<'_, AppState>, product_id: i64) -> Result<f64, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    product_service::product_stock(&conn, product_id)
}

#[tauri::command]
pub fn get_product_movement(
    state: State<'_, AppState>,
    product_id: i64,
    filters: MovementFilters,
) -> Result<Vec<InventoryTransactionRow>, AppError> {
    state.require_user_id()?;
    let conn = state.open_conn()?;
    product_service::product_movement(&conn, product_id, filters)
}

#[tauri::command]
pub fn adjust_stock(
    state: State<'_, AppState>,
    payload: StockAdjustmentPayload,
) -> Result<(), AppError> {
    let user_id = state.require_user_id()?;
    let conn = state.open_conn()?;
    inventory_service::adjust_stock(&conn, user_id, payload)
}
