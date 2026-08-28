use std::path::Path;

use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};

use crate::{
    models::{
        InvoiceSaveResult, QuotationConversionPayload, QuotationDetail, QuotationFilters,
        QuotationItemRow, QuotationListRow, QuotationPayload, QuotationStatusPayload,
        SalesInvoicePayload, SalesItemPayload,
    },
    services::{
        logo_service,
        purchase_service::{escape, money},
        sales_service::create_sales_invoice_in_transaction,
        settings_service::get_company_settings,
    },
    utils::{
        audit::insert_audit_log,
        dates::{now_iso, today_date, validate_date},
        errors::AppError,
        money::checked_total,
        validation::{non_negative_i64, positive_f64},
    },
};

#[derive(Debug)]
struct CustomerSnapshot {
    id: i64,
    name: String,
    company_name: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    address: Option<String>,
    tax_number: Option<String>,
}

#[derive(Debug)]
struct ItemSnapshot {
    product_id: i64,
    sku: String,
    product_name: String,
    quantity: f64,
    unit_price_cents: i64,
    line_total_cents: i64,
}

pub fn create_quotation(
    conn: &Connection,
    user_id: i64,
    payload: QuotationPayload,
) -> Result<QuotationDetail, AppError> {
    validate_payload(&payload)?;
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
    let customer = load_customer(&tx, payload.customer_id)?;
    let items = snapshot_items(&tx, &payload)?;
    let subtotal = quotation_subtotal(&items)?;
    let total = checked_total(subtotal, payload.discount_cents, payload.tax_cents, 0)?;
    let settings = get_company_settings(&tx)?;
    let quotation_number = normalize_number(payload.quotation_number.as_deref())
        .unwrap_or(next_quotation_number(&tx, &settings.quotation_prefix)?);
    ensure_unique_number(&tx, &quotation_number, None)?;
    let now = now_iso();

    tx.execute(
        "INSERT INTO quotations
         (customer_id, quotation_number, quotation_date, valid_until,
          customer_name, customer_company_name, customer_phone, customer_email,
          customer_address, customer_tax_number, subtotal_cents, discount_cents,
          tax_cents, total_cents, notes, terms, status, created_by, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                 ?15, ?16, 'draft', ?17, ?18, ?18)",
        params![
            customer.id,
            quotation_number,
            payload.quotation_date.trim(),
            payload.valid_until.trim(),
            customer.name,
            customer.company_name,
            customer.phone,
            customer.email,
            customer.address,
            customer.tax_number,
            subtotal,
            payload.discount_cents,
            payload.tax_cents,
            total,
            normalize_optional(payload.notes),
            normalize_optional(payload.terms),
            user_id,
            now
        ],
    )?;
    let quotation_id = tx.last_insert_rowid();
    insert_items(&tx, quotation_id, &items, &now)?;
    insert_audit_log(
        &tx,
        user_id,
        "create",
        "quotations",
        quotation_id,
        None,
        Some(serde_json::json!({
            "quotation_number": quotation_number,
            "total_cents": total,
            "status": "draft"
        })),
    )?;
    tx.commit()?;
    get_quotation(conn, quotation_id)
}

pub fn update_quotation(
    conn: &Connection,
    user_id: i64,
    id: i64,
    payload: QuotationPayload,
) -> Result<QuotationDetail, AppError> {
    validate_payload(&payload)?;
    expire_quotations(conn)?;
    let existing = get_quotation(conn, id)?;
    if existing.quotation.status != "draft" {
        return Err(AppError::validation(
            "Only draft quotations can be edited. Create a new revision for a sent or accepted quote.",
        ));
    }

    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
    let customer = load_customer(&tx, payload.customer_id)?;
    let items = snapshot_items(&tx, &payload)?;
    let subtotal = quotation_subtotal(&items)?;
    let total = checked_total(subtotal, payload.discount_cents, payload.tax_cents, 0)?;
    let quotation_number = normalize_number(payload.quotation_number.as_deref())
        .unwrap_or(existing.quotation.quotation_number.clone());
    ensure_unique_number(&tx, &quotation_number, Some(id))?;
    let now = now_iso();

    tx.execute(
        "UPDATE quotations
         SET customer_id = ?1, quotation_number = ?2, quotation_date = ?3, valid_until = ?4,
             customer_name = ?5, customer_company_name = ?6, customer_phone = ?7,
             customer_email = ?8, customer_address = ?9, customer_tax_number = ?10,
             subtotal_cents = ?11, discount_cents = ?12, tax_cents = ?13, total_cents = ?14,
             notes = ?15, terms = ?16, updated_at = ?17
         WHERE id = ?18 AND status = 'draft'",
        params![
            customer.id,
            quotation_number,
            payload.quotation_date.trim(),
            payload.valid_until.trim(),
            customer.name,
            customer.company_name,
            customer.phone,
            customer.email,
            customer.address,
            customer.tax_number,
            subtotal,
            payload.discount_cents,
            payload.tax_cents,
            total,
            normalize_optional(payload.notes),
            normalize_optional(payload.terms),
            now,
            id
        ],
    )?;
    tx.execute("DELETE FROM quotation_items WHERE quotation_id = ?1", [id])?;
    insert_items(&tx, id, &items, &now)?;
    insert_audit_log(
        &tx,
        user_id,
        "update",
        "quotations",
        id,
        Some(serde_json::to_value(existing).unwrap_or_default()),
        Some(serde_json::json!({"quotation_number": quotation_number, "total_cents": total})),
    )?;
    tx.commit()?;
    get_quotation(conn, id)
}

pub fn list_quotations(
    conn: &Connection,
    filters: QuotationFilters,
) -> Result<Vec<QuotationListRow>, AppError> {
    expire_quotations(conn)?;
    if let Some(date) = filters.date_from.as_deref() {
        validate_date(date, "Start date")?;
    }
    if let Some(date) = filters.date_to.as_deref() {
        validate_date(date, "End date")?;
    }
    if let Some(status) = filters.status.as_deref() {
        validate_status(status)?;
    }
    let search = filters
        .search
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let mut stmt = conn.prepare(
        "SELECT id, customer_id, quotation_number, quotation_date, valid_until, customer_name,
                subtotal_cents, discount_cents, tax_cents, total_cents, status,
                converted_sales_invoice_id, notes, terms, created_at, updated_at
         FROM quotations
         WHERE (?1 IS NULL OR quotation_number LIKE '%' || ?1 || '%'
                          OR customer_name LIKE '%' || ?1 || '%'
                          OR customer_phone LIKE '%' || ?1 || '%'
                          OR customer_email LIKE '%' || ?1 || '%')
           AND (?2 IS NULL OR status = ?2)
           AND (?3 IS NULL OR date(quotation_date) >= date(?3))
           AND (?4 IS NULL OR date(quotation_date) <= date(?4))
         ORDER BY quotation_date DESC, id DESC",
    )?;
    let rows = stmt
        .query_map(
            params![search, filters.status, filters.date_from, filters.date_to],
            map_list_row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_quotation(conn: &Connection, id: i64) -> Result<QuotationDetail, AppError> {
    expire_quotations(conn)?;
    let (quotation, company_name, phone, email, address, tax_number) = conn
        .query_row(
            "SELECT id, customer_id, quotation_number, quotation_date, valid_until, customer_name,
                    subtotal_cents, discount_cents, tax_cents, total_cents, status,
                    converted_sales_invoice_id, notes, terms, created_at, updated_at,
                    customer_company_name, customer_phone, customer_email, customer_address,
                    customer_tax_number
             FROM quotations WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    map_list_row(row)?,
                    row.get(16)?,
                    row.get(17)?,
                    row.get(18)?,
                    row.get(19)?,
                    row.get(20)?,
                ))
            },
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => AppError::not_found("Quotation not found."),
            other => other.into(),
        })?;
    let mut stmt = conn.prepare(
        "SELECT id, product_id, sku, product_name, quantity, unit_price_cents, line_total_cents
         FROM quotation_items WHERE quotation_id = ?1 ORDER BY id",
    )?;
    let items = stmt
        .query_map([id], |row| {
            Ok(QuotationItemRow {
                id: row.get(0)?,
                product_id: row.get(1)?,
                sku: row.get(2)?,
                product_name: row.get(3)?,
                quantity: row.get(4)?,
                unit_price_cents: row.get(5)?,
                line_total_cents: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(QuotationDetail {
        quotation,
        customer_company_name: company_name,
        customer_phone: phone,
        customer_email: email,
        customer_address: address,
        customer_tax_number: tax_number,
        items,
    })
}

pub fn change_quotation_status(
    conn: &Connection,
    user_id: i64,
    id: i64,
    payload: QuotationStatusPayload,
) -> Result<QuotationDetail, AppError> {
    expire_quotations(conn)?;
    let current = get_quotation(conn, id)?;
    let next = payload.status.trim().to_ascii_lowercase();
    validate_status(&next)?;
    let allowed = matches!(
        (current.quotation.status.as_str(), next.as_str()),
        ("draft", "sent")
            | ("draft", "rejected")
            | ("sent", "accepted")
            | ("sent", "rejected")
            | ("sent", "expired")
            | ("accepted", "rejected")
    );
    if !allowed {
        return Err(AppError::validation(format!(
            "Quotation status cannot change from {} to {}.",
            current.quotation.status, next
        )));
    }
    let now = now_iso();
    conn.execute(
        "UPDATE quotations SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![next, now, id],
    )?;
    insert_audit_log(
        conn,
        user_id,
        "status_change",
        "quotations",
        id,
        Some(serde_json::json!({"status": current.quotation.status})),
        Some(serde_json::json!({"status": next})),
    )?;
    get_quotation(conn, id)
}

pub fn delete_draft_quotation(conn: &Connection, user_id: i64, id: i64) -> Result<(), AppError> {
    expire_quotations(conn)?;
    let detail = get_quotation(conn, id)?;
    if detail.quotation.status != "draft" {
        return Err(AppError::validation(
            "Only a draft quotation can be deleted. Reject sent quotations to preserve their history.",
        ));
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM quotations WHERE id = ?1", [id])?;
    insert_audit_log(
        &tx,
        user_id,
        "delete",
        "quotations",
        id,
        Some(serde_json::to_value(detail).unwrap_or_default()),
        None,
    )?;
    tx.commit()?;
    Ok(())
}

pub fn convert_quotation(
    conn: &Connection,
    user_id: i64,
    id: i64,
    payload: QuotationConversionPayload,
) -> Result<InvoiceSaveResult, AppError> {
    validate_date(&payload.invoice_date, "Invoice date")?;
    non_negative_i64(payload.delivery_cents, "Delivery")?;
    non_negative_i64(payload.paid_cents, "Paid amount")?;
    expire_quotations(conn)?;
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
    let detail = get_quotation(&tx, id)?;
    if detail.quotation.status == "converted"
        || detail.quotation.converted_sales_invoice_id.is_some()
    {
        return Err(AppError::validation(
            "This quotation has already been converted to a sales invoice.",
        ));
    }
    if detail.quotation.status != "accepted" {
        return Err(AppError::validation(
            "Only an accepted, unexpired quotation can be converted.",
        ));
    }
    let customer_id = detail.quotation.customer_id.ok_or_else(|| {
        AppError::validation(
            "The quotation customer no longer exists. Select a customer on a new quotation.",
        )
    })?;
    let customer_active: bool = tx
        .query_row(
            "SELECT is_active = 1 FROM customers WHERE id = ?1",
            [customer_id],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or(false);
    if !customer_active {
        return Err(AppError::validation(
            "The quotation customer is archived or unavailable and cannot be used for a sale.",
        ));
    }

    let mut sales_items = Vec::with_capacity(detail.items.len());
    for item in &detail.items {
        let product_id = item.product_id.ok_or_else(|| {
            AppError::validation(format!(
                "{} ({}) is no longer available and must be replaced before conversion.",
                item.product_name, item.sku
            ))
        })?;
        let active: bool = tx
            .query_row(
                "SELECT is_active = 1 FROM products WHERE id = ?1",
                [product_id],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(false);
        if !active {
            return Err(AppError::validation(format!(
                "{} ({}) is archived or unavailable and cannot be converted.",
                item.product_name, item.sku
            )));
        }
        sales_items.push(SalesItemPayload {
            product_id,
            quantity: item.quantity,
            unit_price_cents: item.unit_price_cents,
        });
    }
    let invoice_notes = match detail.quotation.notes.as_deref() {
        Some(notes) if !notes.trim().is_empty() => Some(format!(
            "Converted from quotation {}. {}",
            detail.quotation.quotation_number, notes
        )),
        _ => Some(format!(
            "Converted from quotation {}.",
            detail.quotation.quotation_number
        )),
    };
    let result = create_sales_invoice_in_transaction(
        &tx,
        user_id,
        SalesInvoicePayload {
            customer_id: Some(customer_id),
            invoice_number: payload.invoice_number,
            invoice_date: payload.invoice_date,
            discount_cents: detail.quotation.discount_cents,
            tax_cents: detail.quotation.tax_cents,
            delivery_cents: payload.delivery_cents,
            paid_cents: payload.paid_cents,
            notes: invoice_notes,
            items: sales_items,
        },
    )?;
    let now = now_iso();
    let changed = tx.execute(
        "UPDATE quotations
         SET status = 'converted', converted_sales_invoice_id = ?1, updated_at = ?2
         WHERE id = ?3 AND status = 'accepted' AND converted_sales_invoice_id IS NULL",
        params![result.id, now, id],
    )?;
    if changed != 1 {
        return Err(AppError::validation(
            "This quotation was converted by another operation.",
        ));
    }
    insert_audit_log(
        &tx,
        user_id,
        "convert",
        "quotations",
        id,
        Some(serde_json::json!({"status": "accepted"})),
        Some(serde_json::json!({
            "status": "converted",
            "sales_invoice_id": result.id,
            "invoice_number": result.invoice_number
        })),
    )?;
    tx.commit()?;
    Ok(result)
}

pub fn quotation_html(conn: &Connection, db_path: &Path, id: i64) -> Result<String, AppError> {
    let detail = get_quotation(conn, id)?;
    let settings = get_company_settings(conn)?;
    let company_logo = logo_service::logo_data_uri(db_path, settings.logo_path.as_deref());
    let logo = match company_logo {
        Some(uri) => format!(
            r#"<img class="logo" src="{}" alt="Company logo">"#,
            escape(&uri)
        ),
        None => format!(
            r#"<div class="logo-fallback" aria-label="Company logo unavailable">{}</div>"#,
            escape(&initials(&settings.company_name))
        ),
    };
    let contact = [
        settings.address.as_deref(),
        settings.phone.as_deref(),
        settings.email.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter(|value| !value.trim().is_empty())
    .map(escape)
    .collect::<Vec<_>>()
    .join(" · ");
    let customer_contact = [
        detail.customer_phone.as_deref(),
        detail.customer_email.as_deref(),
        detail.customer_address.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter(|value| !value.trim().is_empty())
    .map(escape)
    .collect::<Vec<_>>()
    .join("<br>");
    let rows = detail
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            format!(
                "<tr><td>{}</td><td><strong>{}</strong><small>{}</small></td><td class=\"num\">{:.3}</td><td class=\"num\">{}</td><td class=\"num\">{}</td></tr>",
                index + 1,
                escape(&item.product_name),
                escape(&item.sku),
                item.quantity,
                money(item.unit_price_cents),
                money(item.line_total_cents)
            )
        })
        .collect::<String>();
    let company_tax = settings
        .tax_number
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("<div>Tax / VAT: {}</div>", escape(value)))
        .unwrap_or_default();
    let customer_company = detail
        .customer_company_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("<div>{}</div>", escape(value)))
        .unwrap_or_default();
    let customer_tax = detail
        .customer_tax_number
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("<div>Tax / VAT: {}</div>", escape(value)))
        .unwrap_or_default();
    let notes = optional_section("Notes", detail.quotation.notes.as_deref());
    let terms = optional_section("Terms and conditions", detail.quotation.terms.as_deref());
    let currency = escape(&settings.default_currency);
    Ok(format!(
        r#"<!doctype html>
<html><head><meta charset="utf-8"><title>Quotation {number}</title>
<style>
*{{box-sizing:border-box}}html{{background:#fff}}body{{font-family:Inter,"Segoe UI",Arial,sans-serif;color:#16202a;margin:0;font-size:11px;line-height:1.45}}.sheet{{max-width:210mm;margin:0 auto;padding:12mm 13mm 16mm}}.top{{display:flex;justify-content:space-between;align-items:flex-start;gap:12mm;padding-bottom:8mm;border-bottom:2px solid #245a61}}.brand{{display:flex;gap:5mm;min-width:0;align-items:flex-start}}.logo,.logo-fallback{{width:25mm;height:25mm;flex:0 0 25mm;object-fit:contain}}.logo-fallback{{display:grid;place-items:center;border:1px solid #aebdc2;color:#245a61;font-size:16px;font-weight:800;letter-spacing:.06em}}.company{{min-width:0}}.company h1{{margin:0;font-size:20px;line-height:1.15;overflow-wrap:anywhere}}.muted{{color:#5b6773;margin-top:2px}}.doc{{text-align:right;flex:0 0 54mm}}.doc .eyebrow{{color:#1f6f78;font-size:9px;font-weight:800;letter-spacing:.15em;text-transform:uppercase}}.doc h2{{margin:2px 0 5px;font-size:24px;line-height:1;letter-spacing:-.03em}}.doc dl{{display:grid;grid-template-columns:auto auto;gap:2px 10px;justify-content:end;margin:0}}.doc dt{{color:#687680}}.doc dd{{margin:0;font-weight:700}}.status{{display:inline-block;margin-top:5px;padding:2px 7px;border:1px solid #88a1a7;text-transform:capitalize;font-size:9px;font-weight:700}}.customer{{display:grid;grid-template-columns:1fr 1fr;gap:10mm;margin:8mm 0}}.customer h3,.section h3{{margin:0 0 2mm;color:#456069;font-size:9px;letter-spacing:.12em;text-transform:uppercase}}.customer strong{{font-size:13px}}table{{width:100%;border-collapse:collapse;table-layout:fixed}}thead{{display:table-header-group}}tr{{break-inside:avoid;page-break-inside:avoid}}th,td{{padding:7px 8px;border-bottom:1px solid #d8e1e5;text-align:left;vertical-align:top;overflow-wrap:anywhere}}th{{background:#e9f0f1;color:#20383c;font-size:9px;letter-spacing:.04em;text-transform:uppercase}}th:nth-child(1){{width:7%}}th:nth-child(2){{width:49%}}th:nth-child(3){{width:12%}}th:nth-child(4),th:nth-child(5){{width:16%}}td small{{display:block;color:#687680;margin-top:2px}}.num{{text-align:right;font-variant-numeric:tabular-nums;white-space:nowrap}}.summary{{display:grid;grid-template-columns:minmax(0,1fr) 72mm;gap:12mm;margin-top:7mm;align-items:start}}.totals{{break-inside:avoid;page-break-inside:avoid}}.totals div{{display:flex;justify-content:space-between;gap:10mm;padding:4px 0;border-bottom:1px solid #e2e8eb}}.totals .grand{{margin-top:2px;padding:7px 0;border-top:2px solid #245a61;border-bottom:0;font-size:14px;font-weight:800}}.currency{{color:#687680;font-size:9px;margin-left:3px}}.section{{margin-top:6mm;break-inside:avoid;page-break-inside:avoid}}.section p{{margin:0;white-space:normal;overflow-wrap:anywhere}}.notice{{margin-top:8mm;padding:4mm;border:1px solid #9bb0b5;background:#f4f7f8;font-weight:700;text-align:center}}footer{{margin-top:10mm;padding-top:3mm;border-top:1px solid #cfdadd;color:#687680;font-size:9px}}@page{{size:A4;margin:12mm 13mm 16mm}}@media print{{body{{margin:0}}.sheet{{max-width:none;padding:0}}button{{display:none}}footer{{position:running(quoteFooter)}}}}@media (max-width:700px){{.top,.customer,.summary{{grid-template-columns:1fr;display:grid}}.doc{{text-align:left}}.doc dl{{justify-content:start}}}}
</style></head><body><main class="sheet">
<header class="top"><div class="brand">{logo}<div class="company"><h1>{company}</h1><div class="muted">{contact}</div>{company_tax}</div></div><div class="doc"><div class="eyebrow">Price quote</div><h2>QUOTATION</h2><dl><dt>Quote no.</dt><dd>{number}</dd><dt>Date</dt><dd>{date}</dd><dt>Valid until</dt><dd>{valid_until}</dd></dl><div class="status">{status}</div></div></header>
<section class="customer"><div><h3>Prepared for</h3><strong>{customer}</strong>{customer_company}{customer_tax}</div><div><h3>Customer contact</h3><div>{customer_contact}</div></div></section>
<table><thead><tr><th>#</th><th>Product / SKU</th><th class="num">Qty</th><th class="num">Unit price</th><th class="num">Line total</th></tr></thead><tbody>{rows}</tbody></table>
<div class="summary"><div>{notes}{terms}</div><div class="totals"><div><span>Subtotal</span><span>{subtotal} <span class="currency">{currency}</span></span></div><div><span>Discount</span><span>- {discount}</span></div><div><span>Tax / VAT</span><span>{tax}</span></div><div class="grand"><span>Quoted total</span><span>{total} <span class="currency">{currency}</span></span></div></div></div>
<div class="notice">This document is a quotation only. It is not an invoice, receipt, completed sale, or stock reservation.</div>
<footer>{company} · Quotation {number}</footer></main></body></html>"#,
        logo = logo,
        company = escape(&settings.company_name),
        contact = contact,
        company_tax = company_tax,
        number = escape(&detail.quotation.quotation_number),
        date = escape(&detail.quotation.quotation_date),
        valid_until = escape(&detail.quotation.valid_until),
        status = escape(&detail.quotation.status),
        customer = escape(&detail.quotation.customer_name),
        customer_company = customer_company,
        customer_tax = customer_tax,
        customer_contact = if customer_contact.is_empty() {
            "Not provided".to_string()
        } else {
            customer_contact
        },
        rows = rows,
        notes = notes,
        terms = terms,
        subtotal = money(detail.quotation.subtotal_cents),
        discount = money(detail.quotation.discount_cents),
        tax = money(detail.quotation.tax_cents),
        total = money(detail.quotation.total_cents),
        currency = currency,
    ))
}

fn validate_payload(payload: &QuotationPayload) -> Result<(), AppError> {
    validate_date(&payload.quotation_date, "Quotation date")?;
    validate_date(&payload.valid_until, "Valid-until date")?;
    let quotation_date = NaiveDate::parse_from_str(payload.quotation_date.trim(), "%Y-%m-%d")
        .map_err(|_| AppError::validation("Quotation date is invalid."))?;
    let valid_until = NaiveDate::parse_from_str(payload.valid_until.trim(), "%Y-%m-%d")
        .map_err(|_| AppError::validation("Valid-until date is invalid."))?;
    if valid_until < quotation_date {
        return Err(AppError::validation(
            "Valid-until date cannot be before the quotation date.",
        ));
    }
    if payload.items.is_empty() {
        return Err(AppError::validation(
            "Add at least one product to the quotation.",
        ));
    }
    non_negative_i64(payload.discount_cents, "Discount")?;
    non_negative_i64(payload.tax_cents, "Tax")?;
    for item in &payload.items {
        positive_f64(item.quantity, "Quantity")?;
        non_negative_i64(item.unit_price_cents, "Unit price")?;
        checked_line_total(item.quantity, item.unit_price_cents)?;
    }
    Ok(())
}

fn snapshot_items(
    conn: &Connection,
    payload: &QuotationPayload,
) -> Result<Vec<ItemSnapshot>, AppError> {
    payload
        .items
        .iter()
        .map(|item| {
            let product = conn
                .query_row(
                    "SELECT sku, name FROM products WHERE id = ?1 AND is_active = 1",
                    [item.product_id],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()?;
            let (sku, product_name) = product.ok_or_else(|| {
                AppError::validation(format!(
                    "Product {} is archived, missing, or unavailable.",
                    item.product_id
                ))
            })?;
            Ok(ItemSnapshot {
                product_id: item.product_id,
                sku,
                product_name,
                quantity: item.quantity,
                unit_price_cents: item.unit_price_cents,
                line_total_cents: checked_line_total(item.quantity, item.unit_price_cents)?,
            })
        })
        .collect()
}

fn checked_line_total(quantity: f64, unit_price_cents: i64) -> Result<i64, AppError> {
    let total = quantity * unit_price_cents as f64;
    if !total.is_finite() || total > i64::MAX as f64 {
        return Err(AppError::validation("A quotation line total is too large."));
    }
    Ok(total.round() as i64)
}

fn quotation_subtotal(items: &[ItemSnapshot]) -> Result<i64, AppError> {
    items.iter().try_fold(0_i64, |subtotal, item| {
        subtotal
            .checked_add(item.line_total_cents)
            .ok_or_else(|| AppError::validation("Quotation subtotal is too large."))
    })
}

fn load_customer(conn: &Connection, id: i64) -> Result<CustomerSnapshot, AppError> {
    conn.query_row(
        "SELECT id, name, company_name, phone, email, address, tax_number
         FROM customers WHERE id = ?1 AND is_active = 1",
        [id],
        |row| {
            Ok(CustomerSnapshot {
                id: row.get(0)?,
                name: row.get(1)?,
                company_name: row.get(2)?,
                phone: row.get(3)?,
                email: row.get(4)?,
                address: row.get(5)?,
                tax_number: row.get(6)?,
            })
        },
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::validation("Select an active customer for the quotation.")
        }
        other => other.into(),
    })
}

fn insert_items(
    conn: &Connection,
    quotation_id: i64,
    items: &[ItemSnapshot],
    now: &str,
) -> Result<(), AppError> {
    for item in items {
        conn.execute(
            "INSERT INTO quotation_items
             (quotation_id, product_id, sku, product_name, quantity,
              unit_price_cents, line_total_cents, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                quotation_id,
                item.product_id,
                item.sku,
                item.product_name,
                item.quantity,
                item.unit_price_cents,
                item.line_total_cents,
                now
            ],
        )?;
    }
    Ok(())
}

fn next_quotation_number(conn: &Connection, prefix: &str) -> Result<String, AppError> {
    let next: i64 = conn.query_row(
        "SELECT COALESCE(MAX(id), 0) + 1 FROM quotations",
        [],
        |row| row.get(0),
    )?;
    Ok(format!("{}-{next:06}", prefix.trim().to_uppercase()))
}

fn ensure_unique_number(
    conn: &Connection,
    number: &str,
    exclude_id: Option<i64>,
) -> Result<(), AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM quotations WHERE quotation_number = ?1 AND (?2 IS NULL OR id <> ?2)",
        params![number, exclude_id],
        |row| row.get(0),
    )?;
    if count > 0 {
        Err(AppError::new(
            "DUPLICATE_QUOTATION_NUMBER",
            "A quotation with this number already exists.",
        ))
    } else {
        Ok(())
    }
}

fn expire_quotations(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "UPDATE quotations
         SET status = 'expired', updated_at = ?1
         WHERE status IN ('draft', 'sent', 'accepted')
           AND date(valid_until) < date(?2)",
        params![now_iso(), today_date()],
    )?;
    Ok(())
}

fn validate_status(status: &str) -> Result<(), AppError> {
    if [
        "draft",
        "sent",
        "accepted",
        "rejected",
        "expired",
        "converted",
    ]
    .contains(&status)
    {
        Ok(())
    } else {
        Err(AppError::validation("Quotation status is invalid."))
    }
}

fn map_list_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<QuotationListRow> {
    Ok(QuotationListRow {
        id: row.get(0)?,
        customer_id: row.get(1)?,
        quotation_number: row.get(2)?,
        quotation_date: row.get(3)?,
        valid_until: row.get(4)?,
        customer_name: row.get(5)?,
        subtotal_cents: row.get(6)?,
        discount_cents: row.get(7)?,
        tax_cents: row.get(8)?,
        total_cents: row.get(9)?,
        status: row.get(10)?,
        converted_sales_invoice_id: row.get(11)?,
        notes: row.get(12)?,
        terms: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

fn normalize_number(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_uppercase)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn initials(value: &str) -> String {
    let result = value
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    if result.is_empty() {
        "CO".to_string()
    } else {
        result
    }
}

fn optional_section(title: &str, value: Option<&str>) -> String {
    value
        .filter(|text| !text.trim().is_empty())
        .map(|text| {
            format!(
                r#"<section class="section"><h3>{}</h3><p>{}</p></section>"#,
                escape(title),
                escape(text).replace('\n', "<br>")
            )
        })
        .unwrap_or_default()
}
