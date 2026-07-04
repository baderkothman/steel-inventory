use rusqlite::{params, Connection};

use crate::{
    models::{
        InvoiceDetail, InvoiceFilters, InvoiceItemRow, InvoiceListRow, InvoiceSaveResult,
        SalesInvoicePayload,
    },
    services::{
        inventory_service::{current_stock, insert_inventory_transaction, update_stock},
        product_service::latest_price,
        purchase_service::{escape, invoice_html, money, next_invoice_number},
        settings_service::get_company_settings,
    },
    utils::{
        audit::insert_audit_log,
        dates::{now_iso, validate_date},
        errors::AppError,
        money::{checked_total, payment_status},
        validation::{non_negative_i64, positive_f64},
    },
};

pub fn create_sales_invoice(
    conn: &Connection,
    user_id: i64,
    payload: SalesInvoicePayload,
) -> Result<InvoiceSaveResult, AppError> {
    validate_sales_payload(&payload)?;
    let settings = get_company_settings(conn)?;
    let invoice_number = match payload.invoice_number.as_ref().map(|v| v.trim()).filter(|v| !v.is_empty()) {
        Some(value) => value.to_uppercase(),
        None => next_invoice_number(conn, "sales_invoices", &settings.invoice_prefix_sales)?,
    };
    ensure_unique_sales_number(conn, &invoice_number)?;

    let subtotal = payload
        .items
        .iter()
        .map(|item| (item.quantity * item.unit_price_cents as f64).round() as i64)
        .sum::<i64>();
    let total = checked_total(
        subtotal,
        payload.discount_cents,
        payload.tax_cents,
        payload.delivery_cents,
    )?;
    if payload.paid_cents > total {
        return Err(AppError::validation("Paid amount cannot exceed invoice total."));
    }
    let remaining = total - payload.paid_cents;
    let status = payment_status(total, payload.paid_cents);
    let now = now_iso();

    if !settings.allow_negative_stock {
        for item in payload.items.iter() {
            let stock = current_stock(conn, item.product_id)?;
            if stock + 0.000001 < item.quantity {
                return Err(AppError::insufficient_stock(format!(
                    "Product {} has only {} available.",
                    item.product_id, stock
                )));
            }
        }
    }

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO sales_invoices
         (customer_id, invoice_number, invoice_date, subtotal_cents, discount_cents, tax_cents,
          delivery_cents, total_cents, paid_cents, remaining_cents, payment_status, sales_status,
          notes, created_by, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'completed', ?12, ?13, ?14, ?14)",
        params![
            payload.customer_id,
            invoice_number,
            payload.invoice_date,
            subtotal,
            payload.discount_cents,
            payload.tax_cents,
            payload.delivery_cents,
            total,
            payload.paid_cents,
            remaining,
            status,
            payload.notes,
            user_id,
            now
        ],
    )?;
    let invoice_id = tx.last_insert_rowid();

    for item in payload.items.iter() {
        let (unit_cost, _, _) = latest_price(&tx, item.product_id)?;
        let total_price = (item.quantity * item.unit_price_cents as f64).round() as i64;
        let total_cost = (item.quantity * unit_cost as f64).round() as i64;
        let profit = total_price - total_cost;
        tx.execute(
            "INSERT INTO sales_invoice_items
             (sales_invoice_id, product_id, quantity, unit_cost_cents, unit_price_cents,
              total_cost_cents, total_price_cents, profit_cents, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                invoice_id,
                item.product_id,
                item.quantity,
                unit_cost,
                item.unit_price_cents,
                total_cost,
                total_price,
                profit,
                now
            ],
        )?;
        update_stock(&tx, item.product_id, -item.quantity, settings.allow_negative_stock)?;
        insert_inventory_transaction(
            &tx,
            item.product_id,
            "sale",
            "sales_invoice",
            Some(invoice_id),
            0.0,
            item.quantity,
            Some(unit_cost),
            Some(format!("Sales invoice {invoice_number}")),
            user_id,
        )?;
    }

    if payload.paid_cents > 0 {
        if let Some(customer_id) = payload.customer_id {
            tx.execute(
                "INSERT INTO payments
                 (party_type, party_id, payment_direction, amount_cents, currency, payment_method,
                  payment_date, reference_type, reference_id, notes, created_by, created_at)
                 VALUES ('customer', ?1, 'in', ?2, ?3, 'cash', ?4, 'sales_invoice', ?5, ?6, ?7, ?8)",
                params![
                    customer_id,
                    payload.paid_cents,
                    settings.default_currency,
                    payload.invoice_date,
                    invoice_id,
                    Some(format!("Payment recorded with sales invoice {invoice_number}")),
                    user_id,
                    now
                ],
            )?;
        }
    }

    insert_audit_log(
        &tx,
        user_id,
        "create",
        "sales_invoices",
        invoice_id,
        None,
        Some(serde_json::json!({"id": invoice_id, "invoice_number": invoice_number})),
    )?;
    tx.commit()?;

    Ok(InvoiceSaveResult {
        id: invoice_id,
        invoice_number,
    })
}

pub fn cancel_sales_invoice(conn: &Connection, user_id: i64, id: i64) -> Result<(), AppError> {
    let invoice = get_sales_invoice(conn, id)?;
    if invoice.invoice.status == "cancelled" {
        return Ok(());
    }
    let now = now_iso();
    let tx = conn.unchecked_transaction()?;
    for item in invoice.items {
        update_stock(&tx, item.product_id, item.quantity, true)?;
        insert_inventory_transaction(
            &tx,
            item.product_id,
            "customer_return",
            "sales_invoice",
            Some(id),
            item.quantity,
            0.0,
            Some(item.unit_cost_cents),
            Some(format!("Cancelled sales invoice {}", invoice.invoice.invoice_number)),
            user_id,
        )?;
    }
    tx.execute(
        "UPDATE sales_invoices
         SET sales_status = 'cancelled', payment_status = 'unpaid', paid_cents = 0,
             remaining_cents = total_cents, updated_at = ?1
         WHERE id = ?2",
        params![now, id],
    )?;
    tx.execute(
        "DELETE FROM payments WHERE reference_type = 'sales_invoice' AND reference_id = ?1",
        [id],
    )?;
    insert_audit_log(&tx, user_id, "cancel", "sales_invoices", id, None, None)?;
    tx.commit()?;
    Ok(())
}

pub fn list_sales_invoices(
    conn: &Connection,
    filters: InvoiceFilters,
) -> Result<Vec<InvoiceListRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT si.id, si.invoice_number, si.invoice_date, COALESCE(c.name, 'Walk-in Customer'),
                si.subtotal_cents, si.discount_cents, si.tax_cents, si.delivery_cents,
                si.total_cents, si.paid_cents, si.remaining_cents, si.payment_status,
                si.sales_status, si.notes, si.created_at
         FROM sales_invoices si
         LEFT JOIN customers c ON c.id = si.customer_id
         WHERE (?1 IS NULL OR date(si.invoice_date) >= date(?1))
           AND (?2 IS NULL OR date(si.invoice_date) <= date(?2))
           AND (?3 IS NULL OR si.customer_id = ?3)
           AND (?4 IS NULL OR si.payment_status = ?4)
         ORDER BY si.invoice_date DESC, si.id DESC",
    )?;
    let rows = stmt
        .query_map(
            params![filters.date_from, filters.date_to, filters.party_id, filters.payment_status],
            map_invoice_row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_sales_invoice(conn: &Connection, id: i64) -> Result<InvoiceDetail, AppError> {
    let invoice = conn
        .query_row(
            "SELECT si.id, si.invoice_number, si.invoice_date, COALESCE(c.name, 'Walk-in Customer'),
                    si.subtotal_cents, si.discount_cents, si.tax_cents, si.delivery_cents,
                    si.total_cents, si.paid_cents, si.remaining_cents, si.payment_status,
                    si.sales_status, si.notes, si.created_at
             FROM sales_invoices si
             LEFT JOIN customers c ON c.id = si.customer_id
             WHERE si.id = ?1",
            [id],
            map_invoice_row,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => AppError::not_found("Sales invoice not found."),
            other => other.into(),
        })?;
    let mut stmt = conn.prepare(
        "SELECT sii.id, sii.product_id, p.sku,
                COALESCE(s.name, 'Unknown Supplier') || ' — ' || p.name, sii.quantity,
                sii.unit_cost_cents, sii.unit_price_cents, sii.total_price_cents, sii.profit_cents
         FROM sales_invoice_items sii
         JOIN products p ON p.id = sii.product_id
         LEFT JOIN suppliers s ON s.id = p.supplier_id
         WHERE sii.sales_invoice_id = ?1
         ORDER BY sii.id",
    )?;
    let items = stmt
        .query_map([id], map_item_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(InvoiceDetail { invoice, items })
}

pub fn sales_invoice_html(conn: &Connection, id: i64) -> Result<String, AppError> {
    let settings = get_company_settings(conn)?;
    let detail = get_sales_invoice(conn, id)?;
    let rows = detail
        .items
        .iter()
        .map(|item| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{:.3}</td><td>{}</td><td>{}</td></tr>",
                escape(&item.sku),
                escape(&item.product_name),
                item.quantity,
                money(item.unit_price_cents),
                money(item.row_total_cents)
            )
        })
        .collect::<String>();
    Ok(invoice_html(
        "Sales Invoice",
        &settings.company_name,
        settings.phone.as_deref().unwrap_or(""),
        settings.address.as_deref().unwrap_or(""),
        &detail.invoice.invoice_number,
        &detail.invoice.invoice_date,
        &detail.invoice.party_name,
        &rows,
        detail.invoice.subtotal_cents,
        detail.invoice.discount_cents,
        detail.invoice.tax_cents,
        detail.invoice.extra_cents,
        "Delivery",
        detail.invoice.total_cents,
        detail.invoice.paid_cents,
        detail.invoice.remaining_cents,
        detail.invoice.notes.as_deref().unwrap_or(""),
    ))
}

fn validate_sales_payload(payload: &SalesInvoicePayload) -> Result<(), AppError> {
    validate_date(&payload.invoice_date, "Invoice date")?;
    if payload.items.is_empty() {
        return Err(AppError::validation("At least one invoice item is required."));
    }
    non_negative_i64(payload.discount_cents, "Discount")?;
    non_negative_i64(payload.tax_cents, "Tax")?;
    non_negative_i64(payload.delivery_cents, "Delivery")?;
    non_negative_i64(payload.paid_cents, "Paid amount")?;
    for item in payload.items.iter() {
        positive_f64(item.quantity, "Quantity")?;
        non_negative_i64(item.unit_price_cents, "Unit price")?;
    }
    Ok(())
}

fn ensure_unique_sales_number(conn: &Connection, invoice_number: &str) -> Result<(), AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sales_invoices WHERE invoice_number = ?1",
        [invoice_number],
        |row| row.get(0),
    )?;
    if count > 0 {
        Err(AppError::duplicate_invoice_number())
    } else {
        Ok(())
    }
}

fn map_invoice_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<InvoiceListRow> {
    Ok(InvoiceListRow {
        id: row.get(0)?,
        invoice_number: row.get(1)?,
        invoice_date: row.get(2)?,
        party_name: row.get(3)?,
        subtotal_cents: row.get(4)?,
        discount_cents: row.get(5)?,
        tax_cents: row.get(6)?,
        extra_cents: row.get(7)?,
        total_cents: row.get(8)?,
        paid_cents: row.get(9)?,
        remaining_cents: row.get(10)?,
        payment_status: row.get(11)?,
        status: row.get(12)?,
        notes: row.get(13)?,
        created_at: row.get(14)?,
    })
}

fn map_item_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<InvoiceItemRow> {
    Ok(InvoiceItemRow {
        id: row.get(0)?,
        product_id: row.get(1)?,
        sku: row.get(2)?,
        product_name: row.get(3)?,
        quantity: row.get(4)?,
        unit_cost_cents: row.get(5)?,
        unit_price_cents: row.get(6)?,
        row_total_cents: row.get(7)?,
        profit_cents: row.get(8)?,
    })
}
