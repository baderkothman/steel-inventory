use rusqlite::{params, Connection};

use crate::{
    models::{ExpenseCategory, ExpenseFilters, ExpensePayload, ExpenseRow},
    utils::{
        audit::insert_audit_log,
        dates::{now_iso, validate_date},
        errors::AppError,
        validation::{positive_i64, required},
    },
};

pub fn list_expense_categories(conn: &Connection) -> Result<Vec<ExpenseCategory>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, is_active
         FROM expense_categories
         WHERE is_active = 1
         ORDER BY name",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ExpenseCategory {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                is_active: row.get::<_, i64>(3)? == 1,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn list_expenses(conn: &Connection, filters: ExpenseFilters) -> Result<Vec<ExpenseRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT e.id, e.expense_category_id, ec.name, e.title, e.amount_cents,
                e.currency, e.expense_date, e.payment_method, e.notes, e.created_at, e.updated_at
         FROM expenses e
         JOIN expense_categories ec ON ec.id = e.expense_category_id
         WHERE (?1 IS NULL OR date(e.expense_date) >= date(?1))
           AND (?2 IS NULL OR date(e.expense_date) <= date(?2))
           AND (?3 IS NULL OR e.expense_category_id = ?3)
         ORDER BY e.expense_date DESC, e.id DESC",
    )?;
    let rows = stmt
        .query_map(
            params![filters.date_from, filters.date_to, filters.expense_category_id],
            map_expense,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn create_expense(
    conn: &Connection,
    user_id: i64,
    payload: ExpensePayload,
) -> Result<ExpenseRow, AppError> {
    validate_expense_payload(&payload)?;
    let now = now_iso();
    conn.execute(
        "INSERT INTO expenses
         (expense_category_id, title, amount_cents, currency, expense_date, payment_method,
          notes, created_by, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![
            payload.expense_category_id,
            payload.title.trim(),
            payload.amount_cents,
            payload.currency.trim().to_uppercase(),
            payload.expense_date,
            payload.payment_method,
            payload.notes,
            user_id,
            now
        ],
    )?;
    let id = conn.last_insert_rowid();
    let row = get_expense(conn, id)?;
    insert_audit_log(
        conn,
        user_id,
        "create",
        "expenses",
        id,
        None,
        Some(serde_json::to_value(&row).unwrap_or_default()),
    )?;
    Ok(row)
}

pub fn update_expense(
    conn: &Connection,
    user_id: i64,
    id: i64,
    payload: ExpensePayload,
) -> Result<ExpenseRow, AppError> {
    validate_expense_payload(&payload)?;
    ensure_expense_exists(conn, id)?;
    conn.execute(
        "UPDATE expenses
         SET expense_category_id = ?1, title = ?2, amount_cents = ?3, currency = ?4,
             expense_date = ?5, payment_method = ?6, notes = ?7, updated_at = ?8
         WHERE id = ?9",
        params![
            payload.expense_category_id,
            payload.title.trim(),
            payload.amount_cents,
            payload.currency.trim().to_uppercase(),
            payload.expense_date,
            payload.payment_method,
            payload.notes,
            now_iso(),
            id
        ],
    )?;
    let row = get_expense(conn, id)?;
    insert_audit_log(
        conn,
        user_id,
        "update",
        "expenses",
        id,
        None,
        Some(serde_json::to_value(&row).unwrap_or_default()),
    )?;
    Ok(row)
}

pub fn delete_expense(conn: &Connection, user_id: i64, id: i64) -> Result<(), AppError> {
    ensure_expense_exists(conn, id)?;
    conn.execute("DELETE FROM expenses WHERE id = ?1", [id])?;
    insert_audit_log(conn, user_id, "delete", "expenses", id, None, None)?;
    Ok(())
}

fn get_expense(conn: &Connection, id: i64) -> Result<ExpenseRow, AppError> {
    conn.query_row(
        "SELECT e.id, e.expense_category_id, ec.name, e.title, e.amount_cents,
                e.currency, e.expense_date, e.payment_method, e.notes, e.created_at, e.updated_at
         FROM expenses e
         JOIN expense_categories ec ON ec.id = e.expense_category_id
         WHERE e.id = ?1",
        [id],
        map_expense,
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => AppError::not_found("Expense not found."),
        other => other.into(),
    })
}

fn ensure_expense_exists(conn: &Connection, id: i64) -> Result<(), AppError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM expenses WHERE id = ?1", [id], |row| row.get(0))?;
    if count == 0 {
        Err(AppError::not_found("Expense not found."))
    } else {
        Ok(())
    }
}

fn validate_expense_payload(payload: &ExpensePayload) -> Result<(), AppError> {
    required(&payload.title, "Expense title")?;
    required(&payload.currency, "Currency")?;
    required(&payload.payment_method, "Payment method")?;
    validate_date(&payload.expense_date, "Expense date")?;
    positive_i64(payload.amount_cents, "Amount")?;
    Ok(())
}

fn map_expense(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExpenseRow> {
    Ok(ExpenseRow {
        id: row.get(0)?,
        expense_category_id: row.get(1)?,
        category_name: row.get(2)?,
        title: row.get(3)?,
        amount_cents: row.get(4)?,
        currency: row.get(5)?,
        expense_date: row.get(6)?,
        payment_method: row.get(7)?,
        notes: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}
