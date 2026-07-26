use rusqlite::{params, Connection};

use crate::{
    models::{
        ExpenseCategory, ExpenseFilters, ExpensePayload, ExpenseRow, InstallmentPaymentPayload,
        InstallmentPaymentRow,
    },
    utils::{
        audit::insert_audit_log,
        dates::{now_iso, validate_date},
        errors::AppError,
        money::payment_status,
        validation::{non_negative_i64, positive_i64, required},
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

pub fn list_expenses(
    conn: &Connection,
    filters: ExpenseFilters,
) -> Result<Vec<ExpenseRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT e.id, e.expense_category_id, ec.name, e.title, e.amount_cents,
                e.paid_cents, e.remaining_cents, e.payment_status, e.currency,
                e.expense_date, e.payment_method, e.notes, e.status,
                e.created_at, e.updated_at, e.deleted_at
         FROM expenses e
         JOIN expense_categories ec ON ec.id = e.expense_category_id
         WHERE (?1 IS NULL OR date(e.expense_date) >= date(?1))
           AND (?2 IS NULL OR date(e.expense_date) <= date(?2))
           AND (?3 IS NULL OR e.expense_category_id = ?3)
           AND (?4 = 0 OR e.status = 'active')
         ORDER BY e.expense_date DESC, e.id DESC",
    )?;
    let rows = stmt
        .query_map(
            params![
                filters.date_from,
                filters.date_to,
                filters.expense_category_id,
                if filters.active_only.unwrap_or(false) {
                    1
                } else {
                    0
                }
            ],
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
    if payload.paid_cents > payload.amount_cents {
        return Err(AppError::validation(
            "Amount paid now cannot exceed the expense total.",
        ));
    }
    let now = now_iso();
    let remaining = payload.amount_cents - payload.paid_cents;
    let status = payment_status(payload.amount_cents, payload.paid_cents);
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO expenses
         (expense_category_id, title, amount_cents, currency, expense_date, payment_method,
          notes, created_by, created_at, updated_at, paid_cents, remaining_cents, payment_status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, ?10, ?11, ?12)",
        params![
            payload.expense_category_id,
            payload.title.trim(),
            payload.amount_cents,
            payload.currency.trim().to_uppercase(),
            payload.expense_date,
            payload.payment_method,
            payload.notes,
            user_id,
            now,
            payload.paid_cents,
            remaining,
            status
        ],
    )?;
    let id = tx.last_insert_rowid();
    if payload.paid_cents > 0 {
        tx.execute(
            "INSERT INTO expense_payments
             (expense_id, amount_cents, currency, payment_method, payment_date,
              notes, created_by, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                payload.paid_cents,
                payload.currency.trim().to_uppercase(),
                payload.payment_method,
                payload.expense_date,
                Some("Payment recorded with expense"),
                user_id,
                now
            ],
        )?;
    }
    insert_audit_log(
        &tx,
        user_id,
        "create",
        "expenses",
        id,
        None,
        Some(serde_json::json!({
            "id": id,
            "amount_cents": payload.amount_cents,
            "paid_cents": payload.paid_cents
        })),
    )?;
    tx.commit()?;
    get_expense(conn, id)
}

pub fn update_expense(
    conn: &Connection,
    user_id: i64,
    id: i64,
    payload: ExpensePayload,
) -> Result<ExpenseRow, AppError> {
    validate_expense_payload(&payload)?;
    ensure_active_expense_exists(conn, id)?;
    let paid_cents: i64 = conn.query_row(
        "SELECT paid_cents FROM expenses WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    if payload.amount_cents < paid_cents {
        return Err(AppError::validation(
            "Expense total cannot be lower than the amount already paid.",
        ));
    }
    let remaining = payload.amount_cents - paid_cents;
    let status = payment_status(payload.amount_cents, paid_cents);
    conn.execute(
        "UPDATE expenses
         SET expense_category_id = ?1, title = ?2, amount_cents = ?3, currency = ?4,
             expense_date = ?5, payment_method = ?6, notes = ?7, updated_at = ?8,
             remaining_cents = ?9, payment_status = ?10
         WHERE id = ?11",
        params![
            payload.expense_category_id,
            payload.title.trim(),
            payload.amount_cents,
            payload.currency.trim().to_uppercase(),
            payload.expense_date,
            payload.payment_method,
            payload.notes,
            now_iso(),
            remaining,
            status,
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
    let expense = get_expense(conn, id)?;
    if expense.status == "cancelled" {
        return Ok(());
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE expenses
         SET status = 'cancelled', updated_at = ?1, deleted_at = ?1
         WHERE id = ?2",
        params![now_iso(), id],
    )?;
    tx.execute(
        "UPDATE expense_payments
         SET status = 'cancelled', cancelled_by_expense = 1, deleted_at = ?1
         WHERE expense_id = ?2 AND status = 'active'",
        params![now_iso(), id],
    )?;
    insert_audit_log(&tx, user_id, "cancel", "expenses", id, None, None)?;
    tx.commit()?;
    Ok(())
}

pub fn restore_expense(conn: &Connection, user_id: i64, id: i64) -> Result<(), AppError> {
    let expense = get_expense(conn, id)?;
    if expense.status != "cancelled" {
        return Ok(());
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE expenses
         SET status = 'active', updated_at = ?1, deleted_at = NULL
         WHERE id = ?2",
        params![now_iso(), id],
    )?;
    tx.execute(
        "UPDATE expense_payments
         SET status = 'active', cancelled_by_expense = 0, deleted_at = NULL
         WHERE expense_id = ?1 AND cancelled_by_expense = 1",
        [id],
    )?;
    insert_audit_log(&tx, user_id, "restore", "expenses", id, None, None)?;
    tx.commit()?;
    Ok(())
}

pub fn permanently_delete_expense(
    conn: &Connection,
    user_id: i64,
    id: i64,
) -> Result<(), AppError> {
    let expense = get_expense(conn, id)?;
    if expense.status != "cancelled" {
        return Err(AppError::validation(
            "Only a cancelled expense can be permanently deleted.",
        ));
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM expenses WHERE id = ?1", [id])?;
    insert_audit_log(&tx, user_id, "delete", "expenses", id, None, None)?;
    tx.commit()?;
    Ok(())
}

fn get_expense(conn: &Connection, id: i64) -> Result<ExpenseRow, AppError> {
    conn.query_row(
        "SELECT e.id, e.expense_category_id, ec.name, e.title, e.amount_cents,
                e.paid_cents, e.remaining_cents, e.payment_status, e.currency,
                e.expense_date, e.payment_method, e.notes, e.status,
                e.created_at, e.updated_at, e.deleted_at
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

fn ensure_active_expense_exists(conn: &Connection, id: i64) -> Result<(), AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM expenses WHERE id = ?1 AND status = 'active'",
        [id],
        |row| row.get(0),
    )?;
    if count == 0 {
        Err(AppError::validation("Cancelled expenses cannot be edited."))
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
    non_negative_i64(payload.paid_cents, "Paid amount")?;
    Ok(())
}

pub fn list_expense_payments(
    conn: &Connection,
    expense_id: i64,
) -> Result<Vec<InstallmentPaymentRow>, AppError> {
    get_expense(conn, expense_id)?;
    let mut stmt = conn.prepare(
        "SELECT id, amount_cents, currency, payment_method, payment_date,
                notes, status, created_at
         FROM expense_payments
         WHERE expense_id = ?1 AND status = 'active'
         ORDER BY payment_date DESC, id DESC",
    )?;
    let rows = stmt
        .query_map([expense_id], map_installment_payment)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn record_expense_payment(
    conn: &Connection,
    user_id: i64,
    expense_id: i64,
    payload: InstallmentPaymentPayload,
) -> Result<InstallmentPaymentRow, AppError> {
    positive_i64(payload.amount_cents, "Payment amount")?;
    required(&payload.payment_method, "Payment method")?;
    validate_date(&payload.payment_date, "Payment date")?;
    let expense = get_expense(conn, expense_id)?;
    if expense.status != "active" {
        return Err(AppError::validation(
            "Payments can only be added to an active expense.",
        ));
    }
    if payload.amount_cents > expense.remaining_cents {
        return Err(AppError::validation(
            "Payment amount cannot exceed the remaining expense balance.",
        ));
    }
    let now = now_iso();
    let paid = expense.paid_cents + payload.amount_cents;
    let remaining = expense.amount_cents - paid;
    let status = payment_status(expense.amount_cents, paid);
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO expense_payments
         (expense_id, amount_cents, currency, payment_method, payment_date,
          notes, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            expense_id,
            payload.amount_cents,
            expense.currency,
            payload.payment_method,
            payload.payment_date,
            payload.notes,
            user_id,
            now
        ],
    )?;
    let payment_id = tx.last_insert_rowid();
    tx.execute(
        "UPDATE expenses
         SET paid_cents = ?1, remaining_cents = ?2, payment_status = ?3,
             payment_method = ?4, updated_at = ?5
         WHERE id = ?6",
        params![
            paid,
            remaining,
            status,
            payload.payment_method,
            now,
            expense_id
        ],
    )?;
    insert_audit_log(
        &tx,
        user_id,
        "record_payment",
        "expenses",
        expense_id,
        None,
        Some(serde_json::json!({
            "payment_id": payment_id,
            "amount_cents": payload.amount_cents
        })),
    )?;
    tx.commit()?;
    get_expense_payment(conn, payment_id)
}

fn get_expense_payment(conn: &Connection, id: i64) -> Result<InstallmentPaymentRow, AppError> {
    conn.query_row(
        "SELECT id, amount_cents, currency, payment_method, payment_date,
                notes, status, created_at
         FROM expense_payments
         WHERE id = ?1",
        [id],
        map_installment_payment,
    )
    .map_err(Into::into)
}

fn map_installment_payment(row: &rusqlite::Row<'_>) -> rusqlite::Result<InstallmentPaymentRow> {
    Ok(InstallmentPaymentRow {
        id: row.get(0)?,
        amount_cents: row.get(1)?,
        currency: row.get(2)?,
        payment_method: row.get(3)?,
        payment_date: row.get(4)?,
        notes: row.get(5)?,
        status: row.get(6)?,
        created_at: row.get(7)?,
    })
}

fn map_expense(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExpenseRow> {
    Ok(ExpenseRow {
        id: row.get(0)?,
        expense_category_id: row.get(1)?,
        category_name: row.get(2)?,
        title: row.get(3)?,
        amount_cents: row.get(4)?,
        paid_cents: row.get(5)?,
        remaining_cents: row.get(6)?,
        payment_status: row.get(7)?,
        currency: row.get(8)?,
        expense_date: row.get(9)?,
        payment_method: row.get(10)?,
        notes: row.get(11)?,
        status: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
        deleted_at: row.get(15)?,
    })
}
