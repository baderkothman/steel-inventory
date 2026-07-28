use std::collections::BTreeMap;

use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};

use crate::{
    models::{
        PurchaseReturnContext, PurchaseReturnDetail, PurchaseReturnItemPayload,
        PurchaseReturnItemRow, PurchaseReturnPayload, PurchaseReturnRow,
        PurchaseReturnUpdatePayload, PurchaseReturnableItemRow,
    },
    services::{
        inventory_service::{recalculate_stock, update_stock},
        purchase_service::{escape, get_purchase_invoice, money},
        settings_service::get_company_settings,
    },
    utils::{
        audit::insert_audit_log,
        dates::{now_iso, validate_date},
        errors::AppError,
        money::checked_total,
        validation::{positive_f64, required},
    },
};

const QUANTITY_EPSILON: f64 = 0.000_001;

#[derive(Debug, Clone)]
struct OriginalLine {
    purchase_invoice_item_id: i64,
    product_id: i64,
    quantity: f64,
    unit_cost_cents: i64,
    total_cost_cents: i64,
}

#[derive(Debug)]
struct OriginalInvoice {
    supplier_id: i64,
    subtotal_cents: i64,
    discount_cents: i64,
    tax_cents: i64,
    shipping_cents: i64,
    status: String,
}

#[derive(Debug)]
struct ReturnAmounts {
    subtotal_cents: i64,
    discount_cents: i64,
    tax_cents: i64,
    shipping_cents: i64,
    total_cents: i64,
}

struct ReturnLineInsert<'a> {
    return_id: i64,
    invoice_id: i64,
    return_number: &'a str,
    revision: i64,
    user_id: i64,
    now: &'a str,
}

pub fn create_purchase_return(
    conn: &Connection,
    user_id: i64,
    payload: PurchaseReturnPayload,
) -> Result<PurchaseReturnDetail, AppError> {
    let payload = normalize_create_payload(payload);
    validate_return_payload(
        &payload.return_date,
        &payload.items,
        Some(&payload.idempotency_key),
    )?;
    let request_payload_json =
        serde_json::to_string(&payload).map_err(|error| AppError::database(error.to_string()))?;
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;

    if let Some((id, invoice_id, stored_payload)) = tx
        .query_row(
            "SELECT id, purchase_invoice_id, request_payload_json
             FROM purchase_returns
             WHERE idempotency_key = ?1",
            [&payload.idempotency_key],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()?
    {
        if invoice_id != payload.purchase_invoice_id || stored_payload != request_payload_json {
            return Err(AppError::validation(
                "This idempotency key was already used for a different purchase return.",
            ));
        }
        let detail = get_purchase_return(&tx, id)?;
        tx.commit()?;
        return Ok(detail);
    }

    let invoice = original_invoice(&tx, payload.purchase_invoice_id)?;
    ensure_invoice_active(&invoice)?;
    let lines = validate_and_load_lines(&tx, payload.purchase_invoice_id, None, &payload.items)?;
    let amounts =
        calculate_return_amounts(&tx, payload.purchase_invoice_id, None, &invoice, &lines)?;
    let now = now_iso();
    let return_number = next_return_number(&tx)?;

    tx.execute(
        "INSERT INTO purchase_returns
         (purchase_invoice_id, supplier_id, return_number, return_date, reason, notes,
          subtotal_cents, discount_cents, tax_cents, shipping_cents, total_cents,
          status, idempotency_key, request_payload_json, current_revision,
          created_by, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                 'active', ?12, ?13, 1, ?14, ?15, ?15)",
        params![
            payload.purchase_invoice_id,
            invoice.supplier_id,
            return_number,
            payload.return_date,
            payload.reason,
            payload.notes,
            amounts.subtotal_cents,
            amounts.discount_cents,
            amounts.tax_cents,
            amounts.shipping_cents,
            amounts.total_cents,
            payload.idempotency_key,
            request_payload_json,
            user_id,
            now
        ],
    )?;
    let return_id = tx.last_insert_rowid();
    insert_return_lines(
        &tx,
        ReturnLineInsert {
            return_id,
            invoice_id: payload.purchase_invoice_id,
            return_number: &return_number,
            revision: 1,
            user_id,
            now: &now,
        },
        &lines,
    )?;
    recalculate_purchase_invoice_accounting(&tx, payload.purchase_invoice_id)?;
    insert_audit_log(
        &tx,
        user_id,
        "create",
        "purchase_returns",
        return_id,
        None,
        Some(serde_json::json!({
            "return_number": return_number,
            "purchase_invoice_id": payload.purchase_invoice_id,
            "total_cents": amounts.total_cents
        })),
    )?;
    tx.commit()?;
    get_purchase_return(conn, return_id)
}

pub fn update_purchase_return(
    conn: &Connection,
    user_id: i64,
    id: i64,
    payload: PurchaseReturnUpdatePayload,
) -> Result<PurchaseReturnDetail, AppError> {
    let payload = normalize_update_payload(payload);
    validate_return_payload(&payload.return_date, &payload.items, None)?;
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
    let existing = get_purchase_return(&tx, id)?;
    if existing.return_record.status != "active" {
        return Err(AppError::validation(
            "Only an active purchase return can be edited.",
        ));
    }
    let invoice_id = existing.return_record.purchase_invoice_id;
    let invoice = original_invoice(&tx, invoice_id)?;
    ensure_invoice_active(&invoice)?;
    let old_value = serde_json::to_value(&existing).unwrap_or_default();
    let lines = validate_and_load_lines(&tx, invoice_id, Some(id), &payload.items)?;
    let amounts = calculate_return_amounts(&tx, invoice_id, Some(id), &invoice, &lines)?;
    let now = now_iso();
    let revision: i64 = tx.query_row(
        "SELECT current_revision + 1 FROM purchase_returns WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;

    tx.execute(
        "UPDATE inventory_transactions
         SET status = 'cancelled', deleted_at = ?1
         WHERE purchase_return_id = ?2 AND status = 'active'",
        params![now, id],
    )?;
    tx.execute(
        "UPDATE purchase_return_items
         SET status = 'superseded', superseded_at = ?1
         WHERE purchase_return_id = ?2 AND status = 'active'",
        params![now, id],
    )?;
    for product_id in existing.items.iter().map(|item| item.product_id) {
        recalculate_stock(&tx, product_id)?;
    }

    tx.execute(
        "UPDATE purchase_returns
         SET return_date = ?1, reason = ?2, notes = ?3,
             subtotal_cents = ?4, discount_cents = ?5, tax_cents = ?6,
             shipping_cents = ?7, total_cents = ?8,
             current_revision = ?9, updated_at = ?10
         WHERE id = ?11",
        params![
            payload.return_date,
            payload.reason,
            payload.notes,
            amounts.subtotal_cents,
            amounts.discount_cents,
            amounts.tax_cents,
            amounts.shipping_cents,
            amounts.total_cents,
            revision,
            now,
            id
        ],
    )?;
    insert_return_lines(
        &tx,
        ReturnLineInsert {
            return_id: id,
            invoice_id,
            return_number: &existing.return_record.return_number,
            revision,
            user_id,
            now: &now,
        },
        &lines,
    )?;
    recalculate_purchase_invoice_accounting(&tx, invoice_id)?;
    let updated = get_purchase_return(&tx, id)?;
    insert_audit_log(
        &tx,
        user_id,
        "update",
        "purchase_returns",
        id,
        Some(old_value),
        Some(serde_json::to_value(&updated).unwrap_or_default()),
    )?;
    tx.commit()?;
    get_purchase_return(conn, id)
}

pub fn cancel_purchase_return(conn: &Connection, user_id: i64, id: i64) -> Result<(), AppError> {
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
    let detail = get_purchase_return(&tx, id)?;
    if detail.return_record.status == "cancelled" {
        tx.commit()?;
        return Ok(());
    }
    let now = now_iso();
    tx.execute(
        "UPDATE purchase_returns
         SET status = 'cancelled', cancelled_by = ?1, cancelled_at = ?2, updated_at = ?2
         WHERE id = ?3",
        params![user_id, now, id],
    )?;
    tx.execute(
        "UPDATE inventory_transactions
         SET status = 'cancelled', deleted_at = ?1
         WHERE purchase_return_id = ?2 AND status = 'active'",
        params![now, id],
    )?;
    for product_id in detail.items.iter().map(|item| item.product_id) {
        recalculate_stock(&tx, product_id)?;
    }
    recalculate_purchase_invoice_accounting(&tx, detail.return_record.purchase_invoice_id)?;
    insert_audit_log(
        &tx,
        user_id,
        "cancel",
        "purchase_returns",
        id,
        Some(serde_json::to_value(&detail).unwrap_or_default()),
        None,
    )?;
    tx.commit()?;
    Ok(())
}

pub fn restore_purchase_return(conn: &Connection, user_id: i64, id: i64) -> Result<(), AppError> {
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
    let detail = get_purchase_return(&tx, id)?;
    if detail.return_record.status == "active" {
        tx.commit()?;
        return Ok(());
    }
    let invoice = original_invoice(&tx, detail.return_record.purchase_invoice_id)?;
    ensure_invoice_active(&invoice)?;
    validate_restore(&tx, &detail, &invoice)?;

    for item in &detail.items {
        update_stock(&tx, item.product_id, -item.quantity, false)?;
    }
    tx.execute(
        "UPDATE inventory_transactions
         SET status = 'active', deleted_at = NULL
         WHERE purchase_return_item_id IN (
             SELECT id
             FROM purchase_return_items
             WHERE purchase_return_id = ?1 AND status = 'active'
         )",
        [id],
    )?;
    tx.execute(
        "UPDATE purchase_returns
         SET status = 'active', cancelled_by = NULL, cancelled_at = NULL, updated_at = ?1
         WHERE id = ?2",
        params![now_iso(), id],
    )?;
    recalculate_purchase_invoice_accounting(&tx, detail.return_record.purchase_invoice_id)?;
    let restored = get_purchase_return(&tx, id)?;
    insert_audit_log(
        &tx,
        user_id,
        "restore",
        "purchase_returns",
        id,
        None,
        Some(serde_json::to_value(&restored).unwrap_or_default()),
    )?;
    tx.commit()?;
    Ok(())
}

pub fn get_purchase_return_context(
    conn: &Connection,
    purchase_invoice_id: i64,
) -> Result<PurchaseReturnContext, AppError> {
    let invoice = get_purchase_invoice(conn, purchase_invoice_id)?.invoice;
    let mut stmt = conn.prepare(
        "SELECT pii.id, pii.product_id, p.sku, p.name, pii.quantity,
                COALESCE(SUM(CASE
                    WHEN pr.status = 'active' AND pri.status = 'active' THEN pri.quantity
                    ELSE 0
                END), 0),
                pii.unit_cost_cents
         FROM purchase_invoice_items pii
         JOIN products p ON p.id = pii.product_id
         LEFT JOIN purchase_return_items pri ON pri.purchase_invoice_item_id = pii.id
         LEFT JOIN purchase_returns pr ON pr.id = pri.purchase_return_id
         WHERE pii.purchase_invoice_id = ?1
         GROUP BY pii.id
         ORDER BY pii.id",
    )?;
    let items = stmt
        .query_map([purchase_invoice_id], |row| {
            let purchased_quantity = row.get::<_, f64>(4)?;
            let returned_quantity = row.get::<_, f64>(5)?;
            Ok(PurchaseReturnableItemRow {
                purchase_invoice_item_id: row.get(0)?,
                product_id: row.get(1)?,
                sku: row.get(2)?,
                product_name: row.get(3)?,
                purchased_quantity,
                returned_quantity,
                returnable_quantity: (purchased_quantity - returned_quantity).max(0.0),
                unit_cost_cents: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PurchaseReturnContext {
        invoice,
        items,
        returns: list_purchase_returns(conn, purchase_invoice_id)?,
    })
}

pub fn list_purchase_returns(
    conn: &Connection,
    purchase_invoice_id: i64,
) -> Result<Vec<PurchaseReturnDetail>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id
         FROM purchase_returns
         WHERE purchase_invoice_id = ?1
         ORDER BY return_date DESC, id DESC",
    )?;
    let ids = stmt
        .query_map([purchase_invoice_id], |row| row.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    ids.into_iter()
        .map(|id| get_purchase_return(conn, id))
        .collect()
}

pub fn get_purchase_return(conn: &Connection, id: i64) -> Result<PurchaseReturnDetail, AppError> {
    let return_record = conn
        .query_row(
            "SELECT id, purchase_invoice_id, supplier_id, return_number, return_date,
                    reason, notes, subtotal_cents, discount_cents, tax_cents,
                    shipping_cents, total_cents, status, created_at, updated_at, cancelled_at
             FROM purchase_returns
             WHERE id = ?1",
            [id],
            map_return_row,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::not_found("Purchase return not found.")
            }
            other => other.into(),
        })?;
    let mut stmt = conn.prepare(
        "SELECT pri.id, pri.purchase_return_id, pri.purchase_invoice_item_id,
                pri.product_id, p.sku, p.name, pri.quantity,
                pri.unit_cost_cents, pri.total_cost_cents
         FROM purchase_return_items pri
         JOIN products p ON p.id = pri.product_id
         WHERE pri.purchase_return_id = ?1 AND pri.status = 'active'
         ORDER BY pri.id",
    )?;
    let items = stmt
        .query_map([id], |row| {
            Ok(PurchaseReturnItemRow {
                id: row.get(0)?,
                purchase_return_id: row.get(1)?,
                purchase_invoice_item_id: row.get(2)?,
                product_id: row.get(3)?,
                sku: row.get(4)?,
                product_name: row.get(5)?,
                quantity: row.get(6)?,
                unit_cost_cents: row.get(7)?,
                total_cost_cents: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PurchaseReturnDetail {
        return_record,
        items,
    })
}

pub fn purchase_return_html(conn: &Connection, id: i64) -> Result<String, AppError> {
    let detail = get_purchase_return(conn, id)?;
    let invoice = get_purchase_invoice(conn, detail.return_record.purchase_invoice_id)?;
    let settings = get_company_settings(conn)?;
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
                money(item.total_cost_cents)
            )
        })
        .collect::<String>();
    let record = detail.return_record;
    Ok(format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><title>Purchase Return {number}</title>
<style>*{{box-sizing:border-box}}body{{font-family:Inter,"Segoe UI",Arial,sans-serif;color:#16202a;margin:14mm 12mm;font-size:12px}}header{{display:flex;justify-content:space-between;border-bottom:2px solid #245a61;padding-bottom:14px;margin-bottom:20px}}h1{{margin:0;font-size:22px}}.muted{{color:#5b6773;margin-top:3px}}table{{width:100%;border-collapse:collapse;margin-top:18px}}th,td{{border-bottom:1px solid #d9e0e7;padding:8px;text-align:left}}th{{background:#e9f0f1}}.totals{{margin-left:auto;width:300px;margin-top:18px}}.totals div{{display:flex;justify-content:space-between;padding:5px 0}}.total{{font-weight:700;border-top:2px solid #245a61}}@page{{margin:14mm 12mm}}@media print{{button{{display:none}}body{{margin:0}}}}</style>
</head><body><button onclick="window.print()">Print / Save PDF</button>
<header><div><h1>{company}</h1><div class="muted">{phone}</div><div class="muted">{address}</div></div><div><h1>Purchase Return</h1><div>{number}</div><div>{date}</div><div class="muted">{status}</div></div></header>
<div><strong>Supplier:</strong> {supplier}</div><div><strong>Original invoice:</strong> {invoice_number}</div>
<table><thead><tr><th>SKU</th><th>Product</th><th>Quantity</th><th>Unit cost</th><th>Total</th></tr></thead><tbody>{rows}</tbody></table>
<div class="totals"><div><span>Subtotal</span><span>{subtotal}</span></div><div><span>Discount credit</span><span>{discount}</span></div><div><span>Tax credit</span><span>{tax}</span></div><div><span>Shipping credit</span><span>{shipping}</span></div><div class="total"><span>Return total</span><span>{total}</span></div></div>
<p><strong>Reason:</strong> {reason}</p><p><strong>Notes:</strong> {notes}</p>
</body></html>"#,
        company = escape(&settings.company_name),
        phone = escape(settings.phone.as_deref().unwrap_or("")),
        address = escape(settings.address.as_deref().unwrap_or("")),
        number = escape(&record.return_number),
        date = escape(&record.return_date),
        status = escape(&record.status),
        supplier = escape(&invoice.invoice.party_name),
        invoice_number = escape(&invoice.invoice.invoice_number),
        rows = rows,
        subtotal = money(record.subtotal_cents),
        discount = money(record.discount_cents),
        tax = money(record.tax_cents),
        shipping = money(record.shipping_cents),
        total = money(record.total_cents),
        reason = escape(record.reason.as_deref().unwrap_or("")),
        notes = escape(record.notes.as_deref().unwrap_or(""))
    ))
}

pub fn recalculate_purchase_invoice_accounting(
    conn: &Connection,
    purchase_invoice_id: i64,
) -> Result<(), AppError> {
    let (total_cents, supplier_id): (i64, i64) = conn
        .query_row(
            "SELECT total_cents, supplier_id
             FROM purchase_invoices
             WHERE id = ?1 AND status = 'active'",
            [purchase_invoice_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::validation("The original purchase invoice is not active.")
            }
            other => other.into(),
        })?;
    let returned_cents: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total_cents), 0)
         FROM purchase_returns
         WHERE purchase_invoice_id = ?1 AND status = 'active'",
        [purchase_invoice_id],
        |row| row.get(0),
    )?;
    if returned_cents > total_cents {
        return Err(AppError::validation(
            "Purchase return credits cannot exceed the original invoice total.",
        ));
    }
    let paid_cents: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_cents), 0)
         FROM payments
         WHERE party_type = 'supplier' AND party_id = ?1
           AND reference_type = 'purchase_invoice' AND reference_id = ?2
           AND status = 'active'",
        params![supplier_id, purchase_invoice_id],
        |row| row.get(0),
    )?;
    let net_total = total_cents - returned_cents;
    let remaining_cents = (net_total - paid_cents).max(0);
    let payment_status = if remaining_cents == 0 {
        "paid"
    } else if paid_cents == 0 {
        "unpaid"
    } else {
        "partial"
    };
    conn.execute(
        "UPDATE purchase_invoices
         SET returned_cents = ?1, paid_cents = ?2, remaining_cents = ?3,
             payment_status = ?4, updated_at = ?5
         WHERE id = ?6",
        params![
            returned_cents,
            paid_cents,
            remaining_cents,
            payment_status,
            now_iso(),
            purchase_invoice_id
        ],
    )?;
    Ok(())
}

fn validate_and_load_lines(
    conn: &Connection,
    invoice_id: i64,
    exclude_return_id: Option<i64>,
    items: &[PurchaseReturnItemPayload],
) -> Result<Vec<OriginalLine>, AppError> {
    let mut requested = BTreeMap::<i64, f64>::new();
    for item in items {
        positive_f64(item.quantity, "Return quantity")?;
        *requested
            .entry(item.purchase_invoice_item_id)
            .or_insert(0.0) += item.quantity;
    }

    let mut lines = Vec::with_capacity(requested.len());
    for (invoice_item_id, quantity) in requested {
        let original = conn
            .query_row(
                "SELECT pii.product_id, p.sku, p.name, pii.quantity, pii.unit_cost_cents
                 FROM purchase_invoice_items pii
                 JOIN products p ON p.id = pii.product_id
                 WHERE pii.id = ?1 AND pii.purchase_invoice_id = ?2",
                params![invoice_item_id, invoice_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                },
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => AppError::validation(
                    "A selected product does not belong to the original purchase invoice.",
                ),
                other => other.into(),
            })?;
        let returned: f64 = conn.query_row(
            "SELECT COALESCE(SUM(pri.quantity), 0)
             FROM purchase_return_items pri
             JOIN purchase_returns pr ON pr.id = pri.purchase_return_id
             WHERE pri.purchase_invoice_item_id = ?1
               AND pri.status = 'active'
               AND pr.status = 'active'
               AND (?2 IS NULL OR pr.id <> ?2)",
            params![invoice_item_id, exclude_return_id],
            |row| row.get(0),
        )?;
        let returnable = (original.3 - returned).max(0.0);
        if quantity - returnable > QUANTITY_EPSILON {
            return Err(AppError::validation(format!(
                "Return quantity for {} exceeds the remaining returnable quantity of {:.3}.",
                original.2, returnable
            )));
        }
        lines.push(OriginalLine {
            purchase_invoice_item_id: invoice_item_id,
            product_id: original.0,
            quantity,
            unit_cost_cents: original.4,
            total_cost_cents: (quantity * original.4 as f64).round() as i64,
        });
    }
    Ok(lines)
}

fn calculate_return_amounts(
    conn: &Connection,
    invoice_id: i64,
    exclude_return_id: Option<i64>,
    invoice: &OriginalInvoice,
    lines: &[OriginalLine],
) -> Result<ReturnAmounts, AppError> {
    let subtotal_cents = lines.iter().map(|line| line.total_cost_cents).sum::<i64>();
    let existing: (i64, i64, i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(subtotal_cents), 0),
                COALESCE(SUM(discount_cents), 0),
                COALESCE(SUM(tax_cents), 0),
                COALESCE(SUM(shipping_cents), 0)
         FROM purchase_returns
         WHERE purchase_invoice_id = ?1 AND status = 'active'
           AND (?2 IS NULL OR id <> ?2)",
        params![invoice_id, exclude_return_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )?;
    if existing.0 + subtotal_cents > invoice.subtotal_cents {
        return Err(AppError::validation(
            "The returned merchandise value exceeds the original purchase subtotal.",
        ));
    }
    let full_remaining =
        all_invoice_quantities_returned(conn, invoice_id, exclude_return_id, lines)?;
    let discount_cents = prorated_component(
        invoice.discount_cents,
        existing.1,
        subtotal_cents,
        invoice.subtotal_cents,
        full_remaining,
    );
    let tax_cents = prorated_component(
        invoice.tax_cents,
        existing.2,
        subtotal_cents,
        invoice.subtotal_cents,
        full_remaining,
    );
    let shipping_cents = prorated_component(
        invoice.shipping_cents,
        existing.3,
        subtotal_cents,
        invoice.subtotal_cents,
        full_remaining,
    );
    let total_cents = checked_total(subtotal_cents, discount_cents, tax_cents, shipping_cents)?;
    Ok(ReturnAmounts {
        subtotal_cents,
        discount_cents,
        tax_cents,
        shipping_cents,
        total_cents,
    })
}

fn all_invoice_quantities_returned(
    conn: &Connection,
    invoice_id: i64,
    exclude_return_id: Option<i64>,
    lines: &[OriginalLine],
) -> Result<bool, AppError> {
    let selected = lines
        .iter()
        .map(|line| (line.purchase_invoice_item_id, line.quantity))
        .collect::<BTreeMap<_, _>>();
    let mut stmt = conn.prepare(
        "SELECT pii.id, pii.quantity,
                COALESCE(SUM(CASE
                    WHEN pri.status = 'active' AND pr.status = 'active'
                     AND (?2 IS NULL OR pr.id <> ?2)
                    THEN pri.quantity ELSE 0 END), 0)
         FROM purchase_invoice_items pii
         LEFT JOIN purchase_return_items pri ON pri.purchase_invoice_item_id = pii.id
         LEFT JOIN purchase_returns pr ON pr.id = pri.purchase_return_id
         WHERE pii.purchase_invoice_id = ?1
         GROUP BY pii.id",
    )?;
    let quantities = stmt
        .query_map(params![invoice_id, exclude_return_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, f64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(quantities.into_iter().all(|(id, purchased, returned)| {
        purchased - returned - selected.get(&id).copied().unwrap_or(0.0) <= QUANTITY_EPSILON
    }))
}

fn prorated_component(
    original: i64,
    already_allocated: i64,
    subtotal: i64,
    original_subtotal: i64,
    full_remaining: bool,
) -> i64 {
    let remaining = (original - already_allocated).max(0);
    if full_remaining {
        return remaining;
    }
    if original_subtotal <= 0 {
        return 0;
    }
    ((original as f64 * subtotal as f64 / original_subtotal as f64).round() as i64)
        .clamp(0, remaining)
}

fn insert_return_lines(
    conn: &Connection,
    insert: ReturnLineInsert<'_>,
    lines: &[OriginalLine],
) -> Result<(), AppError> {
    for line in lines {
        update_stock(conn, line.product_id, -line.quantity, false)?;
        conn.execute(
            "INSERT INTO purchase_return_items
             (purchase_return_id, purchase_invoice_item_id, product_id, quantity,
              unit_cost_cents, total_cost_cents, revision, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'active', ?8)",
            params![
                insert.return_id,
                line.purchase_invoice_item_id,
                line.product_id,
                line.quantity,
                line.unit_cost_cents,
                line.total_cost_cents,
                insert.revision,
                insert.now
            ],
        )?;
        let return_item_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO inventory_transactions
             (product_id, transaction_type, reference_type, reference_id,
              quantity_in, quantity_out, unit_cost_cents, notes, created_by,
              created_at, purchase_return_id, purchase_return_item_id)
             VALUES (?1, 'supplier_return', 'purchase_invoice', ?2,
                     0, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                line.product_id,
                insert.invoice_id,
                line.quantity,
                line.unit_cost_cents,
                Some(format!("Purchase return {}", insert.return_number)),
                insert.user_id,
                insert.now,
                insert.return_id,
                return_item_id
            ],
        )?;
    }
    Ok(())
}

fn validate_restore(
    conn: &Connection,
    detail: &PurchaseReturnDetail,
    invoice: &OriginalInvoice,
) -> Result<(), AppError> {
    let payload_items = detail
        .items
        .iter()
        .map(|item| PurchaseReturnItemPayload {
            purchase_invoice_item_id: item.purchase_invoice_item_id,
            quantity: item.quantity,
        })
        .collect::<Vec<_>>();
    validate_and_load_lines(
        conn,
        detail.return_record.purchase_invoice_id,
        Some(detail.return_record.id),
        &payload_items,
    )?;
    let allocated: (i64, i64, i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(subtotal_cents), 0),
                COALESCE(SUM(discount_cents), 0),
                COALESCE(SUM(tax_cents), 0),
                COALESCE(SUM(shipping_cents), 0)
         FROM purchase_returns
         WHERE purchase_invoice_id = ?1 AND status = 'active'",
        [detail.return_record.purchase_invoice_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )?;
    if allocated.0 + detail.return_record.subtotal_cents > invoice.subtotal_cents
        || allocated.1 + detail.return_record.discount_cents > invoice.discount_cents
        || allocated.2 + detail.return_record.tax_cents > invoice.tax_cents
        || allocated.3 + detail.return_record.shipping_cents > invoice.shipping_cents
    {
        return Err(AppError::validation(
            "This return can no longer be restored because later returns used its remaining quantity or invoice credits.",
        ));
    }
    Ok(())
}

fn original_invoice(conn: &Connection, id: i64) -> Result<OriginalInvoice, AppError> {
    conn.query_row(
        "SELECT supplier_id, subtotal_cents, discount_cents,
                tax_cents, shipping_cents, status
         FROM purchase_invoices
         WHERE id = ?1",
        [id],
        |row| {
            Ok(OriginalInvoice {
                supplier_id: row.get(0)?,
                subtotal_cents: row.get(1)?,
                discount_cents: row.get(2)?,
                tax_cents: row.get(3)?,
                shipping_cents: row.get(4)?,
                status: row.get(5)?,
            })
        },
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => AppError::not_found("Purchase invoice not found."),
        other => other.into(),
    })
}

fn ensure_invoice_active(invoice: &OriginalInvoice) -> Result<(), AppError> {
    if invoice.status != "active" {
        Err(AppError::validation(
            "Purchase returns can only be recorded against an active purchase invoice.",
        ))
    } else {
        Ok(())
    }
}

fn next_return_number(conn: &Connection) -> Result<String, AppError> {
    let next: i64 = conn.query_row(
        "SELECT COALESCE(MAX(id), 0) + 1 FROM purchase_returns",
        [],
        |row| row.get(0),
    )?;
    Ok(format!("PR-{next:06}"))
}

fn validate_return_payload(
    return_date: &str,
    items: &[PurchaseReturnItemPayload],
    idempotency_key: Option<&str>,
) -> Result<(), AppError> {
    validate_date(return_date, "Return date")?;
    if items.is_empty() {
        return Err(AppError::validation(
            "Select at least one product and enter a return quantity.",
        ));
    }
    if let Some(key) = idempotency_key {
        required(key, "Idempotency key")?;
        if key.trim().len() < 8 {
            return Err(AppError::validation("The idempotency key is invalid."));
        }
    }
    for item in items {
        positive_f64(item.quantity, "Return quantity")?;
    }
    Ok(())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn normalize_create_payload(mut payload: PurchaseReturnPayload) -> PurchaseReturnPayload {
    payload.return_date = payload.return_date.trim().to_string();
    payload.reason = normalize_optional(payload.reason);
    payload.notes = normalize_optional(payload.notes);
    payload.idempotency_key = payload.idempotency_key.trim().to_string();
    payload
}

fn normalize_update_payload(
    mut payload: PurchaseReturnUpdatePayload,
) -> PurchaseReturnUpdatePayload {
    payload.return_date = payload.return_date.trim().to_string();
    payload.reason = normalize_optional(payload.reason);
    payload.notes = normalize_optional(payload.notes);
    payload
}

fn map_return_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PurchaseReturnRow> {
    Ok(PurchaseReturnRow {
        id: row.get(0)?,
        purchase_invoice_id: row.get(1)?,
        supplier_id: row.get(2)?,
        return_number: row.get(3)?,
        return_date: row.get(4)?,
        reason: row.get(5)?,
        notes: row.get(6)?,
        subtotal_cents: row.get(7)?,
        discount_cents: row.get(8)?,
        tax_cents: row.get(9)?,
        shipping_cents: row.get(10)?,
        total_cents: row.get(11)?,
        status: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
        cancelled_at: row.get(15)?,
    })
}
