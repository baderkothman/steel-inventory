use rusqlite::{params, Connection};
use serde_json::{json, Value};

use crate::{
    models::{DashboardSummary, InvoiceFilters, MovementFilters, PartyFilters, ProductFilters, ReportFilters},
    services::{
        inventory_service::list_stock_movement,
        party_service::{list_parties, party_balance, PartyKind},
        product_service::list_products,
        purchase_service::list_purchase_invoices,
        sales_service::list_sales_invoices,
    },
    utils::{dates::today_date, errors::AppError},
};

pub fn dashboard_summary(conn: &Connection, date: Option<String>) -> Result<DashboardSummary, AppError> {
    let date = date.unwrap_or_else(today_date);
    let today_sales_cents = scalar_i64(
        conn,
        "SELECT COALESCE(SUM(total_cents), 0) FROM sales_invoices WHERE sales_status = 'completed' AND date(invoice_date) = date(?1)",
        &[&date],
    )?;
    let today_paid_cents = scalar_i64(
        conn,
        "SELECT COALESCE(SUM(paid_cents), 0) FROM sales_invoices WHERE sales_status = 'completed' AND date(invoice_date) = date(?1)",
        &[&date],
    )?;
    let today_remaining_cents = scalar_i64(
        conn,
        "SELECT COALESCE(SUM(remaining_cents), 0) FROM sales_invoices WHERE sales_status = 'completed' AND date(invoice_date) = date(?1)",
        &[&date],
    )?;
    let today_profit_cents = scalar_i64(
        conn,
        "SELECT COALESCE(SUM(sii.profit_cents), 0)
         FROM sales_invoice_items sii
         JOIN sales_invoices si ON si.id = sii.sales_invoice_id
         WHERE si.sales_status = 'completed' AND date(si.invoice_date) = date(?1)",
        &[&date],
    )?;
    let today_expenses_cents = scalar_i64(
        conn,
        "SELECT COALESCE(SUM(amount_cents), 0) FROM expenses WHERE date(expense_date) = date(?1)",
        &[&date],
    )?;
    let total_customer_debts_cents = total_debt(conn, PartyKind::Customer)?;
    let total_supplier_debts_cents = total_debt(conn, PartyKind::Supplier)?;
    let current_stock_value_cents = scalar_i64(
        conn,
        "SELECT COALESCE(SUM(sl.current_quantity * COALESCE(pp.cost_price_cents, 0)), 0)
         FROM stock_levels sl
         JOIN products p ON p.id = sl.product_id
         LEFT JOIN product_prices pp ON pp.id = (
             SELECT id FROM product_prices WHERE product_id = p.id ORDER BY effective_from DESC, id DESC LIMIT 1
         )",
        &[],
    )?;

    let mut low_stock_products = list_products(
        conn,
        ProductFilters {
            search: None,
            category_id: None,
            supplier_id: None,
            active_only: Some(true),
        },
    )?
    .into_iter()
    .filter(|product| product.current_quantity <= product.minimum_quantity)
    .collect::<Vec<_>>();
    low_stock_products.truncate(8);
    let low_stock_count = low_stock_products.len() as i64;

    let recent_sales_invoices = list_sales_invoices(
        conn,
        InvoiceFilters {
            date_from: None,
            date_to: None,
            party_id: None,
            payment_status: None,
        },
    )?
    .into_iter()
    .take(6)
    .collect();
    let recent_purchase_invoices = list_purchase_invoices(
        conn,
        InvoiceFilters {
            date_from: None,
            date_to: None,
            party_id: None,
            payment_status: None,
        },
    )?
    .into_iter()
    .take(6)
    .collect();

    Ok(DashboardSummary {
        date,
        today_sales_cents,
        today_paid_cents,
        today_remaining_cents,
        today_profit_cents,
        today_expenses_cents,
        net_profit_cents: today_profit_cents - today_expenses_cents,
        total_customer_debts_cents,
        total_supplier_debts_cents,
        low_stock_count,
        current_stock_value_cents,
        low_stock_products,
        recent_sales_invoices,
        recent_purchase_invoices,
    })
}

pub fn daily_sales_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    let rows = list_sales_invoices(
        conn,
        InvoiceFilters {
            date_from: filters.date_from,
            date_to: filters.date_to,
            party_id: filters.customer_id,
            payment_status: filters.payment_status,
        },
    )?;
    Ok(rows
        .into_iter()
        .map(|row| {
            json!({
                "date": row.invoice_date,
                "invoice": row.invoice_number,
                "customer": row.party_name,
                "total_cents": row.total_cents,
                "paid_cents": row.paid_cents,
                "remaining_cents": row.remaining_cents,
                "payment_status": row.payment_status,
                "status": row.status
            })
        })
        .collect())
}

pub fn purchase_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    let rows = list_purchase_invoices(
        conn,
        InvoiceFilters {
            date_from: filters.date_from,
            date_to: filters.date_to,
            party_id: filters.supplier_id,
            payment_status: filters.payment_status,
        },
    )?;
    Ok(rows
        .into_iter()
        .map(|row| {
            json!({
                "date": row.invoice_date,
                "invoice": row.invoice_number,
                "supplier": row.party_name,
                "total_cents": row.total_cents,
                "paid_cents": row.paid_cents,
                "remaining_cents": row.remaining_cents,
                "payment_status": row.payment_status,
                "status": row.status
            })
        })
        .collect())
}

pub fn profit_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT si.invoice_date, si.invoice_number, COALESCE(c.name, 'Walk-in Customer'),
                SUM(sii.total_price_cents), SUM(sii.total_cost_cents), SUM(sii.profit_cents)
         FROM sales_invoice_items sii
         JOIN sales_invoices si ON si.id = sii.sales_invoice_id
         LEFT JOIN customers c ON c.id = si.customer_id
         WHERE si.sales_status = 'completed'
           AND (?1 IS NULL OR date(si.invoice_date) >= date(?1))
           AND (?2 IS NULL OR date(si.invoice_date) <= date(?2))
           AND (?3 IS NULL OR si.customer_id = ?3)
           AND (?4 IS NULL OR sii.product_id = ?4)
         GROUP BY si.id
         ORDER BY si.invoice_date DESC, si.id DESC",
    )?;
    let rows = stmt
        .query_map(
            params![filters.date_from, filters.date_to, filters.customer_id, filters.product_id],
            |row| {
                Ok(json!({
                    "date": row.get::<_, String>(0)?,
                    "invoice": row.get::<_, String>(1)?,
                    "customer": row.get::<_, String>(2)?,
                    "sales_cents": row.get::<_, i64>(3)?,
                    "cost_cents": row.get::<_, i64>(4)?,
                    "profit_cents": row.get::<_, i64>(5)?
                }))
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn monthly_profit_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT substr(si.invoice_date, 1, 7) AS month,
                COALESCE(SUM(sii.total_price_cents), 0),
                COALESCE(SUM(sii.total_cost_cents), 0),
                COALESCE(SUM(sii.profit_cents), 0),
                COALESCE((SELECT SUM(amount_cents) FROM expenses e WHERE substr(e.expense_date, 1, 7) = substr(si.invoice_date, 1, 7)), 0)
         FROM sales_invoice_items sii
         JOIN sales_invoices si ON si.id = sii.sales_invoice_id
         WHERE si.sales_status = 'completed'
           AND (?1 IS NULL OR date(si.invoice_date) >= date(?1))
           AND (?2 IS NULL OR date(si.invoice_date) <= date(?2))
         GROUP BY month
         ORDER BY month DESC",
    )?;
    let rows = stmt
        .query_map(params![filters.date_from, filters.date_to], |row| {
            let profit: i64 = row.get(3)?;
            let expenses: i64 = row.get(4)?;
            Ok(json!({
                "month": row.get::<_, String>(0)?,
                "sales_cents": row.get::<_, i64>(1)?,
                "cost_cents": row.get::<_, i64>(2)?,
                "gross_profit_cents": profit,
                "expenses_cents": expenses,
                "net_profit_cents": profit - expenses
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn stock_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    let products = list_products(
        conn,
        ProductFilters {
            search: None,
            category_id: filters.category_id,
            supplier_id: filters.supplier_id,
            active_only: Some(true),
        },
    )?;
    Ok(products
        .into_iter()
        .map(|p| {
            json!({
                "sku": p.sku,
                "product": p.name,
                "supplier": p.supplier_name,
                "category": p.category_name,
                "unit": p.unit,
                "current_quantity": p.current_quantity,
                "minimum_quantity": p.minimum_quantity,
                "cost_price_cents": p.cost_price_cents,
                "stock_value_cents": (p.current_quantity * p.cost_price_cents as f64).round() as i64
            })
        })
        .collect())
}

/// Printable physical stock-count sheet: system quantity plus blank counted/difference
/// columns for manual entry, with supplier, category, unit, and storage location.
/// Filters: category, supplier, and low-stock-only.
pub fn stock_count_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    let products = list_products(
        conn,
        ProductFilters {
            search: None,
            category_id: filters.category_id,
            supplier_id: filters.supplier_id,
            active_only: Some(true),
        },
    )?;
    let low_only = filters
        .payment_status
        .as_deref()
        .map(|value| value.eq_ignore_ascii_case("low"))
        .unwrap_or(false);
    Ok(products
        .into_iter()
        .filter(|p| !low_only || p.current_quantity <= p.minimum_quantity)
        .map(|p| {
            json!({
                "sku": p.sku,
                "product": p.name,
                "supplier": p.supplier_name,
                "category": p.category_name,
                "location": p.location.clone().unwrap_or_default(),
                "unit": p.unit,
                "system_quantity": p.current_quantity,
                "counted_quantity": "",
                "difference": ""
            })
        })
        .collect())
}

/// Supplier settlement / payable report: how much is owed to each supplier based on
/// goods of theirs that were actually sold (completed sales only). Groups by supplier
/// and product, with quantity sold, unit cost, and amount owed per product and per supplier.
pub fn supplier_settlement_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(s.name, 'Unknown Supplier') AS supplier,
                p.sku, p.name,
                SUM(sii.quantity) AS qty,
                CASE WHEN SUM(sii.quantity) = 0 THEN 0
                     ELSE CAST(ROUND(SUM(sii.total_cost_cents) / SUM(sii.quantity)) AS INTEGER) END AS unit_cost_cents,
                SUM(sii.total_cost_cents) AS owed_cents
         FROM sales_invoice_items sii
         JOIN sales_invoices si ON si.id = sii.sales_invoice_id
         JOIN products p ON p.id = sii.product_id
         LEFT JOIN suppliers s ON s.id = p.supplier_id
         WHERE si.sales_status = 'completed'
           AND (?1 IS NULL OR date(si.invoice_date) >= date(?1))
           AND (?2 IS NULL OR date(si.invoice_date) <= date(?2))
           AND (?3 IS NULL OR p.supplier_id = ?3)
         GROUP BY p.supplier_id, p.id
         ORDER BY supplier ASC, owed_cents DESC",
    )?;
    let rows = stmt
        .query_map(
            params![filters.date_from, filters.date_to, filters.supplier_id],
            |row| {
                Ok(json!({
                    "supplier": row.get::<_, String>(0)?,
                    "sku": row.get::<_, String>(1)?,
                    "product": row.get::<_, String>(2)?,
                    "quantity_sold": row.get::<_, f64>(3)?,
                    "unit_cost_cents": row.get::<_, i64>(4)?,
                    "owed_cents": row.get::<_, i64>(5)?
                }))
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Per-supplier settlement summary: grand total owed (from completed sales) minus
/// settlement payments already recorded for the period, with remaining balance.
pub fn supplier_settlement_summary(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT s.id, COALESCE(s.name, 'Unknown Supplier') AS supplier,
                COALESCE(owed.owed_cents, 0) AS owed_cents,
                COALESCE(paid.paid_cents, 0) AS paid_cents
         FROM suppliers s
         LEFT JOIN (
             SELECT p.supplier_id AS sid, SUM(sii.total_cost_cents) AS owed_cents
             FROM sales_invoice_items sii
             JOIN sales_invoices si ON si.id = sii.sales_invoice_id
             JOIN products p ON p.id = sii.product_id
             WHERE si.sales_status = 'completed'
               AND (?1 IS NULL OR date(si.invoice_date) >= date(?1))
               AND (?2 IS NULL OR date(si.invoice_date) <= date(?2))
             GROUP BY p.supplier_id
         ) owed ON owed.sid = s.id
         LEFT JOIN (
             SELECT supplier_id AS sid, SUM(amount_cents) AS paid_cents
             FROM supplier_settlement_payments
             WHERE (?1 IS NULL OR date(payment_date) >= date(?1))
               AND (?2 IS NULL OR date(payment_date) <= date(?2))
             GROUP BY supplier_id
         ) paid ON paid.sid = s.id
         WHERE (?3 IS NULL OR s.id = ?3)
           AND (COALESCE(owed.owed_cents, 0) > 0 OR COALESCE(paid.paid_cents, 0) > 0)
         ORDER BY owed_cents DESC, supplier ASC",
    )?;
    let rows = stmt
        .query_map(
            params![filters.date_from, filters.date_to, filters.supplier_id],
            |row| {
                let owed: i64 = row.get(2)?;
                let paid: i64 = row.get(3)?;
                Ok(json!({
                    "supplier": row.get::<_, String>(1)?,
                    "owed_cents": owed,
                    "settled_cents": paid,
                    "remaining_cents": owed - paid
                }))
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Cheapest-supplier comparison: same specification across suppliers, sorted by
/// selling price (cheapest first) within each shared product. Marks the cheapest row.
pub fn cheapest_supplier_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    use std::collections::HashMap;
    let variants = crate::services::product_service::list_supplier_variants(
        conn,
        crate::models::VariantFilters {
            search: None,
            category_id: filters.category_id,
            in_stock_only: None,
        },
    )?;
    let mut cheapest: HashMap<String, i64> = HashMap::new();
    for v in &variants {
        cheapest
            .entry(v.spec_key.clone())
            .and_modify(|min| {
                if v.selling_price_cents < *min {
                    *min = v.selling_price_cents;
                }
            })
            .or_insert(v.selling_price_cents);
    }
    Ok(variants
        .into_iter()
        .map(|v| {
            let is_cheapest = cheapest.get(&v.spec_key).map(|min| *min == v.selling_price_cents).unwrap_or(false);
            json!({
                "product": v.name,
                "supplier": v.supplier_name,
                "category": v.category_name,
                "unit": v.unit,
                "available_quantity": v.current_quantity,
                "cost_price_cents": v.cost_price_cents,
                "selling_price_cents": v.selling_price_cents,
                "cheapest": if is_cheapest { "Yes" } else { "" }
            })
        })
        .collect())
}

pub fn low_stock_report(conn: &Connection) -> Result<Vec<Value>, AppError> {
    Ok(stock_report(conn, ReportFilters::default())?
        .into_iter()
        .filter(|row| {
            let current = row["current_quantity"].as_f64().unwrap_or_default();
            let minimum = row["minimum_quantity"].as_f64().unwrap_or_default();
            current <= minimum
        })
        .collect())
}

pub fn customer_debt_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    debt_report(conn, PartyKind::Customer, filters)
}

pub fn supplier_debt_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    debt_report(conn, PartyKind::Supplier, filters)
}

pub fn expense_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT e.expense_date, ec.name, e.title, e.amount_cents, e.payment_method, e.notes
         FROM expenses e
         JOIN expense_categories ec ON ec.id = e.expense_category_id
         WHERE (?1 IS NULL OR date(e.expense_date) >= date(?1))
           AND (?2 IS NULL OR date(e.expense_date) <= date(?2))
         ORDER BY e.expense_date DESC, e.id DESC",
    )?;
    let rows = stmt
        .query_map(params![filters.date_from, filters.date_to], |row| {
            Ok(json!({
                "date": row.get::<_, String>(0)?,
                "category": row.get::<_, String>(1)?,
                "title": row.get::<_, String>(2)?,
                "amount_cents": row.get::<_, i64>(3)?,
                "payment_method": row.get::<_, String>(4)?,
                "notes": row.get::<_, Option<String>>(5)?
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn payment_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    let party_type = if filters.customer_id.is_some() {
        Some("customer".to_string())
    } else if filters.supplier_id.is_some() {
        Some("supplier".to_string())
    } else {
        None
    };
    let party_id = filters.customer_id.or(filters.supplier_id);
    let rows = crate::services::payment_service::list_payments(
        conn,
        crate::models::PaymentFilters {
            date_from: filters.date_from,
            date_to: filters.date_to,
            party_type,
            party_id,
        },
    )?;
    Ok(rows
        .into_iter()
        .map(|p| {
            json!({
                "date": p.payment_date,
                "party_type": p.party_type,
                "party": p.party_name,
                "direction": p.payment_direction,
                "amount_cents": p.amount_cents,
                "method": p.payment_method,
                "reference_type": p.reference_type,
                "reference_id": p.reference_id
            })
        })
        .collect())
}

pub fn inventory_value_report(conn: &Connection) -> Result<Vec<Value>, AppError> {
    stock_report(conn, ReportFilters::default())
}

pub fn best_selling_products_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT p.sku, p.name, SUM(sii.quantity), SUM(sii.total_price_cents), SUM(sii.profit_cents)
         FROM sales_invoice_items sii
         JOIN sales_invoices si ON si.id = sii.sales_invoice_id
         JOIN products p ON p.id = sii.product_id
         WHERE si.sales_status = 'completed'
           AND (?1 IS NULL OR date(si.invoice_date) >= date(?1))
           AND (?2 IS NULL OR date(si.invoice_date) <= date(?2))
           AND (?3 IS NULL OR p.category_id = ?3)
         GROUP BY p.id
         ORDER BY SUM(sii.quantity) DESC, SUM(sii.total_price_cents) DESC",
    )?;
    let rows = stmt
        .query_map(params![filters.date_from, filters.date_to, filters.category_id], |row| {
            Ok(json!({
                "sku": row.get::<_, String>(0)?,
                "product": row.get::<_, String>(1)?,
                "quantity_sold": row.get::<_, f64>(2)?,
                "sales_cents": row.get::<_, i64>(3)?,
                "profit_cents": row.get::<_, i64>(4)?
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn stock_movement_report(conn: &Connection, filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    let rows = list_stock_movement(
        conn,
        MovementFilters {
            date_from: filters.date_from,
            date_to: filters.date_to,
        },
    )?;
    Ok(rows
        .into_iter()
        .map(|row| {
            json!({
                "date": row.created_at,
                "sku": row.sku,
                "product": row.product_name,
                "type": row.transaction_type,
                "reference": row.reference_type,
                "reference_id": row.reference_id,
                "quantity_in": row.quantity_in,
                "quantity_out": row.quantity_out,
                "unit_cost_cents": row.unit_cost_cents,
                "notes": row.notes
            })
        })
        .collect())
}

fn debt_report(conn: &Connection, kind: PartyKind, _filters: ReportFilters) -> Result<Vec<Value>, AppError> {
    let parties = list_parties(
        conn,
        kind,
        PartyFilters {
            search: None,
            active_only: Some(true),
        },
    )?;
    Ok(parties
        .into_iter()
        .map(|party| {
            json!({
                "name": party.name,
                "company": party.company_name,
                "phone": party.phone,
                "opening_balance_cents": party.opening_balance_cents,
                "balance_cents": party.balance_cents
            })
        })
        .collect())
}

fn total_debt(conn: &Connection, kind: PartyKind) -> Result<i64, AppError> {
    let parties = list_parties(
        conn,
        kind,
        PartyFilters {
            search: None,
            active_only: Some(true),
        },
    )?;
    let mut total = 0;
    for party in parties {
        total += party_balance(conn, kind, party.id)?;
    }
    Ok(total)
}

fn scalar_i64(conn: &Connection, sql: &str, params_values: &[&str]) -> Result<i64, AppError> {
    let mut stmt = conn.prepare(sql)?;
    let params = rusqlite::params_from_iter(params_values.iter());
    let value = stmt.query_row(params, |row| row.get::<_, f64>(0))?;
    Ok(value.round() as i64)
}
