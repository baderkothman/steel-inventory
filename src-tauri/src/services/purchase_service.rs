use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    models::{
        InvoiceDetail, InvoiceFilters, InvoiceItemRow, InvoiceListRow, InvoiceSaveResult,
        PurchaseInvoicePayload,
    },
    services::{
        inventory_service::{insert_inventory_transaction, recalculate_stock, update_stock},
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
    let invoice_number = match payload
        .invoice_number
        .as_ref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
    {
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
        return Err(AppError::validation(
            "Paid amount cannot exceed invoice total.",
        ));
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
        update_latest_cost_price(
            &tx,
            item.product_id,
            item.unit_cost_cents,
            &settings.default_currency,
            &now,
            invoice_id,
        )?;
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
    let active_returns: i64 = conn.query_row(
        "SELECT COUNT(*) FROM purchase_returns
         WHERE purchase_invoice_id = ?1 AND status = 'active'",
        [id],
        |row| row.get(0),
    )?;
    if active_returns > 0 {
        return Err(AppError::validation(
            "Cancel the active purchase returns before cancelling the original purchase invoice.",
        ));
    }
    let now = now_iso();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE inventory_transactions
         SET status = 'cancelled', deleted_at = ?1
         WHERE reference_type = 'purchase_invoice' AND reference_id = ?2
           AND purchase_return_id IS NULL",
        params![now, id],
    )?;
    for item in &invoice.items {
        recalculate_stock(&tx, item.product_id)?;
    }
    tx.execute(
        "UPDATE product_prices
         SET status = 'cancelled'
         WHERE reference_type = 'purchase_invoice' AND reference_id = ?1",
        [id],
    )?;
    tx.execute(
        "UPDATE purchase_invoices
         SET status = 'cancelled', payment_status = 'unpaid', paid_cents = 0,
             remaining_cents = total_cents, updated_at = ?1, deleted_at = ?1
         WHERE id = ?2",
        params![now, id],
    )?;
    tx.execute(
        "UPDATE payments
         SET status = 'cancelled', deleted_at = ?1, cancelled_by_invoice = 1
         WHERE reference_type = 'purchase_invoice' AND reference_id = ?2
           AND status = 'active'",
        params![now, id],
    )?;
    insert_audit_log(&tx, user_id, "cancel", "purchase_invoices", id, None, None)?;
    tx.commit()?;
    Ok(())
}

pub fn restore_purchase_invoice(conn: &Connection, user_id: i64, id: i64) -> Result<(), AppError> {
    let invoice = get_purchase_invoice(conn, id)?;
    if invoice.invoice.status != "cancelled" {
        return Ok(());
    }
    let now = now_iso();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE inventory_transactions
         SET status = 'active', deleted_at = NULL
         WHERE reference_type = 'purchase_invoice' AND reference_id = ?1
           AND purchase_return_id IS NULL",
        [id],
    )?;
    for item in &invoice.items {
        recalculate_stock(&tx, item.product_id)?;
    }
    tx.execute(
        "UPDATE product_prices
         SET status = 'active'
         WHERE reference_type = 'purchase_invoice' AND reference_id = ?1",
        [id],
    )?;
    tx.execute(
        "UPDATE payments
         SET status = 'active', deleted_at = NULL, cancelled_by_invoice = 0
         WHERE reference_type = 'purchase_invoice' AND reference_id = ?1
           AND cancelled_by_invoice = 1",
        [id],
    )?;
    let paid_cents: i64 = tx.query_row(
        "SELECT COALESCE(SUM(amount_cents), 0)
         FROM payments
         WHERE reference_type = 'purchase_invoice' AND reference_id = ?1
           AND status = 'active'",
        [id],
        |row| row.get(0),
    )?;
    tx.execute(
        "UPDATE purchase_invoices
         SET status = 'active', paid_cents = ?1,
             returned_cents = 0,
             remaining_cents = MAX(total_cents - ?1, 0),
             payment_status = CASE
               WHEN ?1 <= 0 THEN 'unpaid'
               WHEN ?1 >= total_cents THEN 'paid'
               ELSE 'partial'
             END,
             updated_at = ?2, deleted_at = NULL
         WHERE id = ?3",
        params![paid_cents, now, id],
    )?;
    insert_audit_log(&tx, user_id, "restore", "purchase_invoices", id, None, None)?;
    tx.commit()?;
    Ok(())
}

pub fn permanently_delete_purchase_invoice(
    conn: &Connection,
    user_id: i64,
    id: i64,
) -> Result<(), AppError> {
    let invoice = get_purchase_invoice(conn, id)?;
    if invoice.invoice.status != "cancelled" {
        return Err(AppError::validation(
            "Only a cancelled purchase can be permanently deleted.",
        ));
    }
    let return_history: i64 = conn.query_row(
        "SELECT COUNT(*) FROM purchase_returns WHERE purchase_invoice_id = ?1",
        [id],
        |row| row.get(0),
    )?;
    if return_history > 0 {
        return Err(AppError::validation(
            "A purchase with return history cannot be permanently deleted.",
        ));
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM payments
         WHERE reference_type = 'purchase_invoice' AND reference_id = ?1",
        [id],
    )?;
    tx.execute(
        "DELETE FROM inventory_transactions
         WHERE reference_type = 'purchase_invoice' AND reference_id = ?1",
        [id],
    )?;
    tx.execute(
        "DELETE FROM product_prices
         WHERE reference_type = 'purchase_invoice' AND reference_id = ?1",
        [id],
    )?;
    tx.execute(
        "DELETE FROM purchase_invoice_items WHERE purchase_invoice_id = ?1",
        [id],
    )?;
    tx.execute("DELETE FROM purchase_invoices WHERE id = ?1", [id])?;
    insert_audit_log(&tx, user_id, "delete", "purchase_invoices", id, None, None)?;
    tx.commit()?;
    Ok(())
}

pub fn list_purchase_invoices(
    conn: &Connection,
    filters: InvoiceFilters,
) -> Result<Vec<InvoiceListRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT pi.id, pi.supplier_id, pi.invoice_number, pi.invoice_date, s.name,
                pi.subtotal_cents, pi.discount_cents, pi.tax_cents, pi.shipping_cents,
                pi.total_cents, pi.returned_cents, MAX(pi.total_cents - pi.returned_cents, 0),
                pi.paid_cents, pi.remaining_cents, pi.payment_status,
                pi.status, pi.notes, pi.created_at, pi.deleted_at
         FROM purchase_invoices pi
         JOIN suppliers s ON s.id = pi.supplier_id
         WHERE (?1 IS NULL OR date(pi.invoice_date) >= date(?1))
           AND (?2 IS NULL OR date(pi.invoice_date) <= date(?2))
           AND (?3 IS NULL OR pi.supplier_id = ?3)
           AND (?4 IS NULL OR pi.payment_status = ?4)
           AND (?5 = 0 OR pi.status = 'active')
         ORDER BY pi.invoice_date DESC, pi.id DESC",
    )?;
    let rows = stmt
        .query_map(
            params![
                filters.date_from,
                filters.date_to,
                filters.party_id,
                filters.payment_status,
                if filters.active_only.unwrap_or(false) {
                    1
                } else {
                    0
                }
            ],
            map_invoice_row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_purchase_invoice(conn: &Connection, id: i64) -> Result<InvoiceDetail, AppError> {
    let invoice = conn
        .query_row(
            "SELECT pi.id, pi.supplier_id, pi.invoice_number, pi.invoice_date, s.name,
                    pi.subtotal_cents, pi.discount_cents, pi.tax_cents, pi.shipping_cents,
                    pi.total_cents, pi.returned_cents, MAX(pi.total_cents - pi.returned_cents, 0),
                    pi.paid_cents, pi.remaining_cents, pi.payment_status,
                    pi.status, pi.notes, pi.created_at, pi.deleted_at
             FROM purchase_invoices pi
             JOIN suppliers s ON s.id = pi.supplier_id
             WHERE pi.id = ?1",
            [id],
            map_invoice_row,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::not_found("Purchase invoice not found.")
            }
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
        return Err(AppError::validation(
            "At least one invoice item is required.",
        ));
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

pub fn next_invoice_number(
    conn: &Connection,
    table: &str,
    prefix: &str,
) -> Result<String, AppError> {
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
    purchase_invoice_id: i64,
) -> Result<(), AppError> {
    let latest: Option<(i64, i64)> = conn
        .query_row(
            "SELECT selling_price_cents, wholesale_price_cents
             FROM product_prices
             WHERE product_id = ?1 AND status = 'active'
             ORDER BY effective_from DESC, id DESC
             LIMIT 1",
            [product_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if let Some((selling, wholesale)) = latest {
        conn.execute(
            "INSERT INTO product_prices
             (product_id, cost_price_cents, selling_price_cents, wholesale_price_cents,
              currency, effective_from, created_at, reference_type, reference_id, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, 'purchase_invoice', ?7, 'active')",
            params![
                product_id,
                unit_cost_cents,
                selling,
                wholesale,
                currency,
                now,
                purchase_invoice_id
            ],
        )?;
    }
    Ok(())
}

fn map_invoice_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<InvoiceListRow> {
    Ok(InvoiceListRow {
        id: row.get(0)?,
        party_id: row.get(1)?,
        invoice_number: row.get(2)?,
        invoice_date: row.get(3)?,
        party_name: row.get(4)?,
        subtotal_cents: row.get(5)?,
        discount_cents: row.get(6)?,
        tax_cents: row.get(7)?,
        extra_cents: row.get(8)?,
        total_cents: row.get(9)?,
        returned_cents: row.get(10)?,
        net_total_cents: row.get(11)?,
        paid_cents: row.get(12)?,
        remaining_cents: row.get(13)?,
        payment_status: row.get(14)?,
        status: row.get(15)?,
        notes: row.get(16)?,
        created_at: row.get(17)?,
        deleted_at: row.get(18)?,
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
*{{box-sizing:border-box}} body{{font-family:Inter,"Segoe UI",Arial,sans-serif;color:#16202a;margin:14mm 12mm 16mm;font-size:12px}} .header{{display:flex;justify-content:space-between;gap:24px;border-bottom:2px solid #245a61;padding-bottom:16px;margin-bottom:24px}}
h1{{margin:0;font-size:23px;letter-spacing:-.02em}} .muted{{color:#5b6773;font-size:12px;margin-top:3px}} table{{width:100%;border-collapse:collapse;margin-top:20px}} thead{{display:table-header-group}} tr{{break-inside:avoid;page-break-inside:avoid}} th,td{{border-bottom:1px solid #d9e0e7;padding:9px 10px;text-align:left;vertical-align:top}} th{{background:#e9f0f1;color:#20383c;font-weight:700}} tbody tr:nth-child(even){{background:#f8fafb}} .totals{{margin-left:auto;width:320px;margin-top:20px;break-inside:avoid}} .totals div{{display:flex;justify-content:space-between;padding:6px 0}} .total{{font-weight:700;border-top:2px solid #245a61}} .page-footer{{display:none}} @page{{size:auto;margin:14mm 12mm 16mm}} @media print{{button{{display:none}} body{{margin:0}} .page-footer{{display:block;position:fixed;left:0;right:0;bottom:-7mm;color:#687680;font-size:9px}} .page-number{{float:right}} .page-number:after{{content:counter(page)}}}}
</style></head>
<body>
<button onclick="window.print()">Print / Save PDF</button>
<div class="header"><div><h1>{company}</h1><div class="muted">{phone}</div><div class="muted">{address}</div></div><div><h1>{title}</h1><div>{invoice_number}</div><div>{invoice_date}</div></div></div>
<div><strong>Party:</strong> {party}</div>
<table><thead><tr><th>SKU</th><th>Product</th><th>Quantity</th><th>Unit Price</th><th>Total</th></tr></thead><tbody>{rows}</tbody></table>
<div class="totals"><div><span>Subtotal</span><span>{subtotal}</span></div><div><span>Discount</span><span>{discount}</span></div><div><span>Tax</span><span>{tax}</span></div><div><span>{extra_label}</span><span>{extra}</span></div><div class="total"><span>Total</span><span>{total}</span></div><div><span>Paid</span><span>{paid}</span></div><div><span>Remaining</span><span>{remaining}</span></div></div>
<p><strong>Notes:</strong> {notes}</p>
<div class="page-footer"><span>{company}</span><span class="page-number">Page </span></div>
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
