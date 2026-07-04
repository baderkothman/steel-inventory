use rusqlite::{params, Connection};

use crate::{
    models::{SettlementFilters, SettlementPaymentPayload, SettlementPaymentRow},
    services::settings_service::get_company_settings,
    utils::{
        audit::insert_audit_log,
        dates::{now_iso, validate_date},
        errors::AppError,
        validation::non_negative_i64,
    },
};

/// Records a payment made to a supplier for a settlement period (driven by sales of
/// that supplier's goods). Validates the supplier, dates, amount, and status.
pub fn create_settlement_payment(
    conn: &Connection,
    user_id: i64,
    payload: SettlementPaymentPayload,
) -> Result<SettlementPaymentRow, AppError> {
    validate_date(&payload.period_start, "Period start")?;
    validate_date(&payload.period_end, "Period end")?;
    validate_date(&payload.payment_date, "Payment date")?;
    if payload.period_end < payload.period_start {
        return Err(AppError::validation("Period end must be on or after period start."));
    }
    non_negative_i64(payload.amount_cents, "Amount")?;
    if payload.amount_cents <= 0 {
        return Err(AppError::validation("Settlement amount must be greater than zero."));
    }
    let status = payload.status.trim().to_lowercase();
    if !["unpaid", "partial", "paid"].contains(&status.as_str()) {
        return Err(AppError::validation("Settlement status must be unpaid, partial, or paid."));
    }
    ensure_supplier_exists(conn, payload.supplier_id)?;

    let settings = get_company_settings(conn)?;
    let now = now_iso();
    conn.execute(
        "INSERT INTO supplier_settlement_payments
         (supplier_id, period_start, period_end, amount_cents, currency, status,
          payment_date, reference, notes, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            payload.supplier_id,
            payload.period_start,
            payload.period_end,
            payload.amount_cents,
            settings.default_currency,
            status,
            payload.payment_date,
            clean_optional(payload.reference.as_deref()),
            clean_optional(payload.notes.as_deref()),
            user_id,
            now
        ],
    )?;
    let id = conn.last_insert_rowid();
    insert_audit_log(
        conn,
        user_id,
        "create",
        "supplier_settlement_payments",
        id,
        None,
        Some(serde_json::json!({"id": id, "supplier_id": payload.supplier_id, "amount_cents": payload.amount_cents})),
    )?;
    get_settlement_payment(conn, id)
}

pub fn delete_settlement_payment(conn: &Connection, user_id: i64, id: i64) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM supplier_settlement_payments WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(AppError::not_found("Settlement payment not found."));
    }
    insert_audit_log(conn, user_id, "delete", "supplier_settlement_payments", id, None, None)?;
    Ok(())
}

pub fn list_settlement_payments(
    conn: &Connection,
    filters: SettlementFilters,
) -> Result<Vec<SettlementPaymentRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT sp.id, sp.supplier_id, COALESCE(s.name, 'Unknown Supplier'),
                sp.period_start, sp.period_end, sp.amount_cents, sp.currency, sp.status,
                sp.payment_date, sp.reference, sp.notes, sp.created_at
         FROM supplier_settlement_payments sp
         LEFT JOIN suppliers s ON s.id = sp.supplier_id
         WHERE (?1 IS NULL OR date(sp.payment_date) >= date(?1))
           AND (?2 IS NULL OR date(sp.payment_date) <= date(?2))
           AND (?3 IS NULL OR sp.supplier_id = ?3)
         ORDER BY sp.payment_date DESC, sp.id DESC",
    )?;
    let rows = stmt
        .query_map(
            params![filters.date_from, filters.date_to, filters.supplier_id],
            map_settlement_row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_settlement_payment(conn: &Connection, id: i64) -> Result<SettlementPaymentRow, AppError> {
    conn.query_row(
        "SELECT sp.id, sp.supplier_id, COALESCE(s.name, 'Unknown Supplier'),
                sp.period_start, sp.period_end, sp.amount_cents, sp.currency, sp.status,
                sp.payment_date, sp.reference, sp.notes, sp.created_at
         FROM supplier_settlement_payments sp
         LEFT JOIN suppliers s ON s.id = sp.supplier_id
         WHERE sp.id = ?1",
        [id],
        map_settlement_row,
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => AppError::not_found("Settlement payment not found."),
        other => other.into(),
    })
}

fn ensure_supplier_exists(conn: &Connection, supplier_id: i64) -> Result<(), AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM suppliers WHERE id = ?1",
        [supplier_id],
        |row| row.get(0),
    )?;
    if count == 0 {
        Err(AppError::not_found("Supplier not found."))
    } else {
        Ok(())
    }
}

fn clean_optional(value: Option<&str>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn map_settlement_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SettlementPaymentRow> {
    Ok(SettlementPaymentRow {
        id: row.get(0)?,
        supplier_id: row.get(1)?,
        supplier_name: row.get(2)?,
        period_start: row.get(3)?,
        period_end: row.get(4)?,
        amount_cents: row.get(5)?,
        currency: row.get(6)?,
        status: row.get(7)?,
        payment_date: row.get(8)?,
        reference: row.get(9)?,
        notes: row.get(10)?,
        created_at: row.get(11)?,
    })
}
