use rusqlite::{params, Connection};

use crate::{
    models::{Category, CategoryPayload},
    utils::{
        audit::insert_audit_log,
        dates::now_iso,
        errors::AppError,
        validation::required,
    },
};

pub fn list_categories(conn: &Connection) -> Result<Vec<Category>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, parent_id, description, is_active, created_at, updated_at
         FROM categories
         ORDER BY parent_id IS NOT NULL, parent_id, name",
    )?;
    let rows = stmt
        .query_map([], map_category)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn create_category(
    conn: &Connection,
    user_id: i64,
    payload: CategoryPayload,
) -> Result<Category, AppError> {
    required(&payload.name, "Category name")?;
    let now = now_iso();
    conn.execute(
        "INSERT INTO categories (name, parent_id, description, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, 1, ?4, ?4)",
        params![payload.name.trim(), payload.parent_id, payload.description, now],
    )?;
    let id = conn.last_insert_rowid();
    let category = get_category(conn, id)?;
    insert_audit_log(
        conn,
        user_id,
        "create",
        "categories",
        id,
        None,
        Some(serde_json::to_value(&category).unwrap_or_default()),
    )?;
    Ok(category)
}

pub fn update_category(
    conn: &Connection,
    user_id: i64,
    id: i64,
    payload: CategoryPayload,
) -> Result<Category, AppError> {
    required(&payload.name, "Category name")?;
    if payload.parent_id == Some(id) {
        return Err(AppError::validation("A category cannot be its own parent."));
    }
    ensure_category_exists(conn, id)?;
    let now = now_iso();
    conn.execute(
        "UPDATE categories
         SET name = ?1, parent_id = ?2, description = ?3, updated_at = ?4
         WHERE id = ?5",
        params![payload.name.trim(), payload.parent_id, payload.description, now, id],
    )?;
    let category = get_category(conn, id)?;
    insert_audit_log(
        conn,
        user_id,
        "update",
        "categories",
        id,
        None,
        Some(serde_json::to_value(&category).unwrap_or_default()),
    )?;
    Ok(category)
}

pub fn archive_category(conn: &Connection, user_id: i64, id: i64) -> Result<(), AppError> {
    ensure_category_exists(conn, id)?;
    conn.execute(
        "UPDATE categories SET is_active = 0, updated_at = ?1 WHERE id = ?2",
        params![now_iso(), id],
    )?;
    insert_audit_log(conn, user_id, "archive", "categories", id, None, None)?;
    Ok(())
}

pub fn get_category(conn: &Connection, id: i64) -> Result<Category, AppError> {
    conn.query_row(
        "SELECT id, name, parent_id, description, is_active, created_at, updated_at
         FROM categories WHERE id = ?1",
        [id],
        map_category,
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => AppError::not_found("Category not found."),
        other => other.into(),
    })
}

fn ensure_category_exists(conn: &Connection, id: i64) -> Result<(), AppError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories WHERE id = ?1", [id], |row| {
        row.get(0)
    })?;
    if count == 0 {
        Err(AppError::not_found("Category not found."))
    } else {
        Ok(())
    }
}

fn map_category(row: &rusqlite::Row<'_>) -> rusqlite::Result<Category> {
    Ok(Category {
        id: row.get(0)?,
        name: row.get(1)?,
        parent_id: row.get(2)?,
        description: row.get(3)?,
        is_active: row.get::<_, i64>(4)? == 1,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}
