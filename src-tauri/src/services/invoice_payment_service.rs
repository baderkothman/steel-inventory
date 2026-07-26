use rusqlite::{params, Connection};

use crate::{
    models::{InstallmentPaymentPayload, InstallmentPaymentRow, PaymentPayload},
    services::{
        payment_service, purchase_service::get_purchase_invoice, sales_service::get_sales_invoice,
        settings_service::get_company_settings,
    },
    utils::{
        audit::insert_audit_log,
        dates::{now_iso, validate_date},
        errors::AppError,
        money::payment_status,
        validation::{positive_i64, required},
    },
};

pub fn list_invoice_payments(
    conn: &Connection,
    kind: &str,
    invoice_id: i64,
) -> Result<Vec<InstallmentPaymentRow>, AppError> {
    let reference_type = reference_type(kind)?;
    if kind == "sales"
        && get_sales_invoice(conn, invoice_id)?
            .invoice
            .party_id
            .is_none()
    {
        let mut stmt = conn.prepare(
            "SELECT id, amount_cents, currency, payment_method, payment_date,
                    notes, status, created_at
             FROM walk_in_sales_payments
             WHERE sales_invoice_id = ?1 AND status = 'active'
             ORDER BY payment_date DESC, id DESC",
        )?;
        return Ok(stmt
            .query_map([invoice_id], map_installment)?
            .collect::<Result<Vec<_>, _>>()?);
    }
    if kind == "purchase" {
        get_purchase_invoice(conn, invoice_id)?;
    } else {
        get_sales_invoice(conn, invoice_id)?;
    }
    let mut stmt = conn.prepare(
        "SELECT id, amount_cents, currency, payment_method, payment_date,
                notes, status, created_at
         FROM payments
         WHERE reference_type = ?1 AND reference_id = ?2 AND status = 'active'
         ORDER BY payment_date DESC, id DESC",
    )?;
    let payments = stmt
        .query_map(params![reference_type, invoice_id], map_installment)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(payments)
}

pub fn record_invoice_payment(
    conn: &Connection,
    user_id: i64,
    kind: &str,
    invoice_id: i64,
    payload: InstallmentPaymentPayload,
) -> Result<InstallmentPaymentRow, AppError> {
    validate_payload(&payload)?;
    let reference_type = reference_type(kind)?;
    let invoice = if kind == "purchase" {
        get_purchase_invoice(conn, invoice_id)?.invoice
    } else {
        get_sales_invoice(conn, invoice_id)?.invoice
    };
    if invoice.status == "cancelled" {
        return Err(AppError::validation(
            "Payments can only be added to an active invoice.",
        ));
    }
    if payload.amount_cents > invoice.remaining_cents {
        return Err(AppError::validation(
            "Payment amount cannot exceed the remaining invoice balance.",
        ));
    }

    if let Some(party_id) = invoice.party_id {
        let payment = payment_service::create_payment(
            conn,
            user_id,
            PaymentPayload {
                party_type: if kind == "purchase" {
                    "supplier".to_string()
                } else {
                    "customer".to_string()
                },
                party_id,
                amount_cents: payload.amount_cents,
                currency: String::new(),
                payment_method: payload.payment_method,
                payment_date: payload.payment_date,
                reference_type: Some(reference_type.to_string()),
                reference_id: Some(invoice_id),
                notes: payload.notes,
            },
        )?;
        return Ok(InstallmentPaymentRow {
            id: payment.id,
            amount_cents: payment.amount_cents,
            currency: payment.currency,
            payment_method: payment.payment_method,
            payment_date: payment.payment_date,
            notes: payment.notes,
            status: payment.status,
            created_at: payment.created_at,
        });
    }

    if kind != "sales" {
        return Err(AppError::validation("The invoice party is missing."));
    }
    let settings = get_company_settings(conn)?;
    let now = now_iso();
    let paid = invoice.paid_cents + payload.amount_cents;
    let remaining = invoice.total_cents - paid;
    let status = payment_status(invoice.total_cents, paid);
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO walk_in_sales_payments
         (sales_invoice_id, amount_cents, currency, payment_method, payment_date,
          notes, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            invoice_id,
            payload.amount_cents,
            settings.default_currency,
            payload.payment_method,
            payload.payment_date,
            payload.notes,
            user_id,
            now
        ],
    )?;
    let payment_id = tx.last_insert_rowid();
    tx.execute(
        "UPDATE sales_invoices
         SET paid_cents = ?1, remaining_cents = ?2, payment_status = ?3, updated_at = ?4
         WHERE id = ?5",
        params![paid, remaining, status, now, invoice_id],
    )?;
    insert_audit_log(
        &tx,
        user_id,
        "record_payment",
        "sales_invoices",
        invoice_id,
        None,
        Some(serde_json::json!({
            "payment_id": payment_id,
            "amount_cents": payload.amount_cents,
            "walk_in": true
        })),
    )?;
    tx.commit()?;
    get_walk_in_payment(conn, payment_id)
}

fn get_walk_in_payment(conn: &Connection, id: i64) -> Result<InstallmentPaymentRow, AppError> {
    conn.query_row(
        "SELECT id, amount_cents, currency, payment_method, payment_date,
                notes, status, created_at
         FROM walk_in_sales_payments
         WHERE id = ?1",
        [id],
        map_installment,
    )
    .map_err(Into::into)
}

fn validate_payload(payload: &InstallmentPaymentPayload) -> Result<(), AppError> {
    positive_i64(payload.amount_cents, "Payment amount")?;
    required(&payload.payment_method, "Payment method")?;
    validate_date(&payload.payment_date, "Payment date")?;
    Ok(())
}

fn reference_type(kind: &str) -> Result<&'static str, AppError> {
    match kind {
        "purchase" => Ok("purchase_invoice"),
        "sales" => Ok("sales_invoice"),
        _ => Err(AppError::validation(
            "Invoice type must be purchase or sales.",
        )),
    }
}

fn map_installment(row: &rusqlite::Row<'_>) -> rusqlite::Result<InstallmentPaymentRow> {
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
