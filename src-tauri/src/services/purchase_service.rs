use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    models::{
        InvoiceDetail, InvoiceFilters, InvoiceItemRow, InvoiceListRow, InvoiceSaveResult,
        PurchaseInvoicePayload,
    },
    services::{
        inventory_service::{insert_inventory_transaction, update_stock},
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

pub fn create_purchase_invoice(
    conn: &Connection,
    user_id: i64,
    payload: PurchaseInvoicePayload,
) -> Result<InvoiceSaveResult, AppError> {
    validate_purchase_payload(&payload)?;
    let settings = get_company_settings(conn)?;
    let invoice_number = match payload.invoice_number.as_ref().map(|v| v.trim()).filter(|v| !v.is_empty()) {
        Some(value) => value.to_uppercase(),
        None => next_invoice_number(conn, "purchase_invoices", &settings.invoice_prefix_purchase)?,
    };
    ensure_unique_purchase_number(conn, &invoice_number)?;

    let subtotal = payload
        .items
        .iter()
        .map(|item| (item.quantity * item.unit_cost_cents as f64).round() as i64)
        .sum::<i64>();
    let total = checked_total(
        subtotal,
        payload.discount_cents,
        payload.tax_cents,
        payload.shipping_cents,
    )?;
    if payload.paid_cents > total {
        return Err(AppError::validation("Paid amount cannot exceed invoice total."));
    }
    let remaining = total - payload.paid_cents;
    let status = payment_status(total, payload.paid_cents);
    let now = now_iso();

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO purchase_invoices
         (supplier_id, invoice_number, invoice_date, subtotal_cents, discount_cents, tax_cents,
          shipping_cents, total_cents, paid_cents, remaining_cents, payment_status, status,
          notes, created_by, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'active', ?12, ?13, ?14, ?14)",
        params![
            payload.supplier_id,
            invoice_number,
            payload.invoice_date,
            subtotal,
            payload.discount_cents,
            payload.tax_cents,
            payload.shipping_cents,
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
        let row_total = (item.quantity * item.unit_cost_cents as f64).round() as i64;
        tx.execute(
            "INSERT INTO purchase_invoice_items
             (purchase_invoice_id, product_id, quantity, unit_cost_cents, total_cost_cents, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![invoice_id, item.product_id, item.quantity, item.unit_cost_cents, row_total, now],
        )?;
        update_stock(&tx, item.product_id, item.quantity, true)?;
        insert_inventory_transaction(
            &tx,
            item.product_id,
            "purchase",
            "purchase_invoice",
            Some(invoice_id),
            item.quantity,
            0.0,
            Some(item.unit_cost_cents),
            Some(format!("Purchase invoice {invoice_number}")),
            user_id,
        )?;
        update_latest_cost_price(&tx, item.product_id, item.unit_cost_cents, &settings.default_currency, &now)?;
    }

    if payload.paid_cents > 0 {
        tx.execute(
            "INSERT INTO payments
             (party_type, party_id, payment_direction, amount_cents, currency, payment_method,
              payment_date, reference_type, reference_id, notes, created_by, created_at)
             VALUES ('supplier', ?1, 'out', ?2, ?3, 'cash', ?4, 'purchase_invoice', ?5, ?6, ?7, ?8)",
            params![
                payload.supplier_id,
                payload.paid_cents,
                settings.default_currency,
                payload.invoice_date,
                invoice_id,
                Some(format!("Payment recorded with purchase invoice {invoice_number}")),
                user_id,
                now
            ],
        )?;
    }

    insert_audit_log(
        &tx,
        user_id,
        "create",
        "purchase_invoices",
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

pub fn cancel_purchase_invoice(conn: &Connection, user_id: i64, id: i64) -> Result<(), AppError> {
    let invoice = get_purchase_invoice(conn, id)?;
    if invoice.invoice.status == "cancelled" {
        return Ok(());
    }
    let settings = get_company_settings(conn)?;
    let now = now_iso();
    let tx = conn.unchecked_transaction()?;
    for item in invoice.items {
        update_stock(&tx, item.product_id, -item.quantity, settings.allow_negative_stock)?;
        insert_inventory_transaction(
            &tx,
            item.product_id,
            "supplier_return",
            "purchase_invoice",
            Some(id),
            0.0,
            item.quantity,
            Some(item.unit_cost_cents),
            Some(format!("Cancelled purchase invoice {}", invoice.invoice.invoice_number)),
            user_id,
        )?;
    }
    tx.execute(
        "UPDATE purchase_invoices
         SET status = 'cancelled', payment_status = 'unpaid', paid_cents = 0,
             remaining_cents = total_cents, updated_at = ?1
         WHERE id = ?2",
        params![now, id],
    )?;
    tx.execute(
        "DELETE FROM payments WHERE reference_type = 'purchase_invoice' AND reference_id = ?1",
        [id],
    )?;
    insert_audit_log(&tx, user_id, "cancel", "purchase_invoices", id, None, None)?;
    tx.commit()?;
    Ok(())
}

pub fn list_purchase_invoices(
    conn: &Connection,
    filters: InvoiceFilters,
) -> Result<Vec<InvoiceListRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT pi.id, pi.invoice_number, pi.invoice_date, s.name,
                pi.subtotal_cents, pi.discount_cents, pi.tax_cents, pi.shipping_cents,
                pi.total_cents, pi.paid_cents, pi.remaining_cents, pi.payment_status,
                pi.status, pi.notes, pi.created_at
         FROM purchase_invoices pi
         JOIN suppliers s ON s.id = pi.supplier_id
         WHERE (?1 IS NULL OR date(pi.invoice_date) >= date(?1))
           AND (?2 IS NULL OR date(pi.invoice_date) <= date(?2))
           AND (?3 IS NULL OR pi.supplier_id = ?3)
           AND (?4 IS NULL OR pi.payment_status = ?4)
         ORDER BY pi.invoice_date DESC, pi.id DESC",
    )?;
    let rows = stmt
        .query_map(
            params![filters.date_from, filters.date_to, filters.party_id, filters.payment_status],
            map_invoice_row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_purchase_invoice(conn: &Connection, id: i64) -> Result<InvoiceDetail, AppError> {
    let invoice = conn
        .query_row(
            "SELECT pi.id, pi.invoice_number, pi.invoice_date, s.name,
                    pi.subtotal_cents, pi.discount_cents, pi.tax_cents, pi.shipping_cents,
                    pi.total_cents, pi.paid_cents, pi.remaining_cents, pi.payment_status,
                    pi.status, pi.notes, pi.created_at
             FROM purchase_invoices pi
             JOIN suppliers s ON s.id = pi.supplier_id
             WHERE pi.id = ?1",
            [id],
            map_invoice_row,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => AppError::not_found("Purchase invoice not found."),
            other => other.into(),
        })?;
    let mut stmt = conn.prepare(
        "SELECT pii.id, pii.product_id, p.sku, p.name, pii.quantity,
                pii.unit_cost_cents, 0, pii.total_cost_cents, NULL
         FROM purchase_invoice_items pii
         JOIN products p ON p.id = pii.product_id
         WHERE pii.purchase_invoice_id = ?1
         ORDER BY pii.id",
    )?;
    let items = stmt
        .query_map([id], map_item_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(InvoiceDetail { invoice, items })
}

pub fn purchase_invoice_html(conn: &Connection, id: i64) -> Result<String, AppError> {
    let settings = get_company_settings(conn)?;
    let detail = get_purchase_invoice(conn, id)?;
    let rows = detail
        .items
        .iter()
        .map(|item| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{:.3}</td><td>{}</td><td>{}</td></tr>",
                escape(&item.sku),
                escape(&item.product_name),
                item.quantity,
                money(item.unit_cost_cents),
                money(item.row_total_cents)
            )
        })
        .collect::<String>();
    Ok(invoice_html(
        "Purchase Invoice",
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
        "Shipping",
        detail.invoice.total_cents,
        detail.invoice.paid_cents,
        detail.invoice.remaining_cents,
        detail.invoice.notes.as_deref().unwrap_or(""),
    ))
}

fn validate_purchase_payload(payload: &PurchaseInvoicePayload) -> Result<(), AppError> {
    validate_date(&payload.invoice_date, "Invoice date")?;
    if payload.items.is_empty() {
        return Err(AppError::validation("At least one invoice item is required."));
    }
    non_negative_i64(payload.discount_cents, "Discount")?;
    non_negative_i64(payload.tax_cents, "Tax")?;
    non_negative_i64(payload.shipping_cents, "Shipping")?;
    non_negative_i64(payload.paid_cents, "Paid amount")?;
    for item in payload.items.iter() {
        positive_f64(item.quantity, "Quantity")?;
        non_negative_i64(item.unit_cost_cents, "Unit cost")?;
    }
    Ok(())
}

fn ensure_unique_purchase_number(conn: &Connection, invoice_number: &str) -> Result<(), AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM purchase_invoices WHERE invoice_number = ?1",
        [invoice_number],
        |row| row.get(0),
    )?;
    if count > 0 {
        Err(AppError::duplicate_invoice_number())
    } else {
        Ok(())
    }
}

pub fn next_invoice_number(conn: &Connection, table: &str, prefix: &str) -> Result<String, AppError> {
    let allowed = ["purchase_invoices", "sales_invoices"];
    if !allowed.contains(&table) {
        return Err(AppError::database("Invalid invoice table."));
    }
    let sql = format!("SELECT COALESCE(MAX(id), 0) + 1 FROM {table}");
    let next: i64 = conn.query_row(&sql, [], |row| row.get(0))?;
    Ok(format!("{}-{next:06}", prefix.trim().to_uppercase()))
}

fn update_latest_cost_price(
    conn: &Connection,
    product_id: i64,
    unit_cost_cents: i64,
    currency: &str,
    now: &str,
) -> Result<(), AppError> {
    let latest: Option<(i64, i64)> = conn
        .query_row(
            "SELECT selling_price_cents, wholesale_price_cents
             FROM product_prices
             WHERE product_id = ?1
             ORDER BY effective_from DESC, id DESC
             LIMIT 1",
            [product_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if let Some((selling, wholesale)) = latest {
        conn.execute(
            "INSERT INTO product_prices
             (product_id, cost_price_cents, selling_price_cents, wholesale_price_cents, currency, effective_from, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![product_id, unit_cost_cents, selling, wholesale, currency, now],
        )?;
    }
    Ok(())
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

pub fn invoice_html(
    title: &str,
    company_name: &str,
    company_phone: &str,
    company_address: &str,
    invoice_number: &str,
    invoice_date: &str,
    party_name: &str,
    rows: &str,
    subtotal_cents: i64,
    discount_cents: i64,
    tax_cents: i64,
    extra_cents: i64,
    extra_label: &str,
    total_cents: i64,
    paid_cents: i64,
    remaining_cents: i64,
    notes: &str,
) -> String {
    format!(
        r#"<!doctype html>
<html><head><meta charset="utf-8"><title>{title} {invoice_number}</title>
<style>
body{{font-family:Arial,sans-serif;color:#16202a;margin:32px}} .header{{display:flex;justify-content:space-between;border-bottom:2px solid #16202a;padding-bottom:16px;margin-bottom:24px}}
h1{{margin:0;font-size:24px}} .muted{{color:#5b6773;font-size:13px}} table{{width:100%;border-collapse:collapse;margin-top:20px}} th,td{{border-bottom:1px solid #d9e0e7;padding:10px;text-align:left}} th{{background:#f3f6f8}} .totals{{margin-left:auto;width:320px;margin-top:20px}} .totals div{{display:flex;justify-content:space-between;padding:6px 0}} .total{{font-weight:700;border-top:2px solid #16202a}} @media print{{button{{display:none}} body{{margin:12mm}}}}
</style></head>
<body>
<button onclick="window.print()">Print / Save PDF</button>
<div class="header"><div><h1>{company}</h1><div class="muted">{phone}</div><div class="muted">{address}</div></div><div><h1>{title}</h1><div>{invoice_number}</div><div>{invoice_date}</div></div></div>
<div><strong>Party:</strong> {party}</div>
<table><thead><tr><th>SKU</th><th>Product</th><th>Quantity</th><th>Unit Price</th><th>Total</th></tr></thead><tbody>{rows}</tbody></table>
<div class="totals"><div><span>Subtotal</span><span>{subtotal}</span></div><div><span>Discount</span><span>{discount}</span></div><div><span>Tax</span><span>{tax}</span></div><div><span>{extra_label}</span><span>{extra}</span></div><div class="total"><span>Total</span><span>{total}</span></div><div><span>Paid</span><span>{paid}</span></div><div><span>Remaining</span><span>{remaining}</span></div></div>
<p><strong>Notes:</strong> {notes}</p>
</body></html>"#,
        title = escape(title),
        invoice_number = escape(invoice_number),
        invoice_date = escape(invoice_date),
        company = escape(company_name),
        phone = escape(company_phone),
        address = escape(company_address),
        party = escape(party_name),
        rows = rows,
        subtotal = money(subtotal_cents),
        discount = money(discount_cents),
        tax = money(tax_cents),
        extra_label = escape(extra_label),
        extra = money(extra_cents),
        total = money(total_cents),
        paid = money(paid_cents),
        remaining = money(remaining_cents),
        notes = escape(notes)
    )
}

pub fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn money(value: i64) -> String {
    format!("{:.2}", value as f64 / 100.0)
}
