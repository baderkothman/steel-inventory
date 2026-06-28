use rusqlite::{params, Connection};

use crate::{
    models::{PaymentFilters, PaymentPayload, PaymentRow},
    services::settings_service::get_company_settings,
    utils::{
        audit::insert_audit_log,
        dates::{now_iso, validate_date},
        errors::AppError,
        money::payment_status,
        validation::{positive_i64, required},
    },
};

pub fn list_payments(conn: &Connection, filters: PaymentFilters) -> Result<Vec<PaymentRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.party_type, p.party_id,
                CASE WHEN p.party_type = 'customer' THEN c.name ELSE s.name END AS party_name,
                p.payment_direction, p.amount_cents, p.currency, p.payment_method,
                p.payment_date, p.reference_type, p.reference_id, p.notes, p.created_at
         FROM payments p
         LEFT JOIN customers c ON p.party_type = 'customer' AND c.id = p.party_id
         LEFT JOIN suppliers s ON p.party_type = 'supplier' AND s.id = p.party_id
         WHERE (?1 IS NULL OR date(p.payment_date) >= date(?1))
           AND (?2 IS NULL OR date(p.payment_date) <= date(?2))
           AND (?3 IS NULL OR p.party_type = ?3)
           AND (?4 IS NULL OR p.party_id = ?4)
         ORDER BY p.payment_date DESC, p.id DESC",
    )?;
    let rows = stmt
        .query_map(
            params![
                filters.date_from,
                filters.date_to,
                filters.party_type,
                filters.party_id
            ],
            map_payment,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn create_payment(
    conn: &Connection,
    user_id: i64,
    payload: PaymentPayload,
) -> Result<PaymentRow, AppError> {
    validate_payment_payload(&payload)?;
    let direction = direction_for_party(&payload.party_type)?;
    let settings = get_company_settings(conn)?;
    let currency = if payload.currency.trim().is_empty() {
        settings.default_currency
    } else {
        payload.currency.trim().to_uppercase()
    };
    let now = now_iso();

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO payments
         (party_type, party_id, payment_direction, amount_cents, currency, payment_method,
          payment_date, reference_type, reference_id, notes, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            payload.party_type,
            payload.party_id,
            direction,
            payload.amount_cents,
            currency,
            payload.payment_method,
            payload.payment_date,
            payload.reference_type,
            payload.reference_id,
            payload.notes,
            user_id,
            now
        ],
    )?;
    let id = tx.last_insert_rowid();
    sync_linked_invoice_payment(&tx, &payload.party_type, payload.reference_type.as_deref(), payload.reference_id, payload.amount_cents)?;
    insert_audit_log(
        &tx,
        user_id,
        "create",
        "payments",
        id,
        None,
        Some(serde_json::json!({"id": id, "party_type": payload.party_type, "amount_cents": payload.amount_cents})),
    )?;
    tx.commit()?;
    get_payment(conn, id)
}

pub fn delete_payment(conn: &Connection, user_id: i64, id: i64) -> Result<(), AppError> {
    let payment = get_payment(conn, id)?;
    let tx = conn.unchecked_transaction()?;
    sync_linked_invoice_payment(
        &tx,
        &payment.party_type,
        payment.reference_type.as_deref(),
        payment.reference_id,
        -payment.amount_cents,
    )?;
    tx.execute("DELETE FROM payments WHERE id = ?1", [id])?;
    insert_audit_log(&tx, user_id, "delete", "payments", id, None, None)?;
    tx.commit()?;
    Ok(())
}

fn get_payment(conn: &Connection, id: i64) -> Result<PaymentRow, AppError> {
    conn.query_row(
        "SELECT p.id, p.party_type, p.party_id,
                CASE WHEN p.party_type = 'customer' THEN c.name ELSE s.name END AS party_name,
                p.payment_direction, p.amount_cents, p.currency, p.payment_method,
                p.payment_date, p.reference_type, p.reference_id, p.notes, p.created_at
         FROM payments p
         LEFT JOIN customers c ON p.party_type = 'customer' AND c.id = p.party_id
         LEFT JOIN suppliers s ON p.party_type = 'supplier' AND s.id = p.party_id
         WHERE p.id = ?1",
        [id],
        map_payment,
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => AppError::not_found("Payment not found."),
        other => other.into(),
    })
}

fn sync_linked_invoice_payment(
    conn: &Connection,
    party_type: &str,
    reference_type: Option<&str>,
    reference_id: Option<i64>,
    amount_delta: i64,
) -> Result<(), AppError> {
    let Some(reference_type) = reference_type else {
        return Ok(());
    };
    let Some(reference_id) = reference_id else {
        return Ok(());
    };

    let (table, expected_party) = match reference_type {
        "sales_invoice" => ("sales_invoices", "customer"),
        "purchase_invoice" => ("purchase_invoices", "supplier"),
        _ => return Err(AppError::validation("Invalid payment invoice reference.")),
    };
    if party_type != expected_party {
        return Err(AppError::validation("Payment party type does not match invoice reference."));
    }

    let sql = format!("SELECT total_cents, paid_cents FROM {table} WHERE id = ?1");
    let (total, paid): (i64, i64) = conn
        .query_row(&sql, [reference_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => AppError::not_found("Linked invoice not found."),
            other => other.into(),
        })?;
    let next_paid = paid + amount_delta;
    if next_paid < 0 || next_paid > total {
        return Err(AppError::validation("Payment amount does not fit the linked invoice balance."));
    }
    let remaining = total - next_paid;
    let status = payment_status(total, next_paid);
    let status_column = if table == "sales_invoices" { "payment_status" } else { "payment_status" };
    let sql = format!(
        "UPDATE {table}
         SET paid_cents = ?1, remaining_cents = ?2, {status_column} = ?3, updated_at = ?4
         WHERE id = ?5"
    );
    conn.execute(&sql, params![next_paid, remaining, status, now_iso(), reference_id])?;
    Ok(())
}

fn validate_payment_payload(payload: &PaymentPayload) -> Result<(), AppError> {
    required(&payload.party_type, "Party type")?;
    required(&payload.payment_method, "Payment method")?;
    validate_date(&payload.payment_date, "Payment date")?;
    positive_i64(payload.amount_cents, "Amount")?;
    direction_for_party(&payload.party_type)?;
    if let Some(reference_type) = payload.reference_type.as_deref() {
        if !["sales_invoice", "purchase_invoice"].contains(&reference_type) {
            return Err(AppError::validation("Invalid invoice reference type."));
        }
        if payload.reference_id.is_none() {
            return Err(AppError::validation("Reference ID is required when reference type is provided."));
        }
    }
    Ok(())
}

fn direction_for_party(party_type: &str) -> Result<&'static str, AppError> {
    match party_type {
        "customer" => Ok("in"),
        "supplier" => Ok("out"),
        _ => Err(AppError::validation("Party type must be customer or supplier.")),
    }
}

fn map_payment(row: &rusqlite::Row<'_>) -> rusqlite::Result<PaymentRow> {
    Ok(PaymentRow {
        id: row.get(0)?,
        party_type: row.get(1)?,
        party_id: row.get(2)?,
        party_name: row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "Unknown".to_string()),
        payment_direction: row.get(4)?,
        amount_cents: row.get(5)?,
        currency: row.get(6)?,
        payment_method: row.get(7)?,
        payment_date: row.get(8)?,
        reference_type: row.get(9)?,
        reference_id: row.get(10)?,
        notes: row.get(11)?,
        created_at: row.get(12)?,
    })
}
