use rusqlite::{params, Connection};

use crate::{
    models::{InventoryTransactionRow, MovementFilters, StockAdjustmentPayload},
    services::settings_service::get_company_settings,
    utils::{
        audit::insert_audit_log,
        dates::{now_iso, validate_date},
        errors::AppError,
        validation::positive_f64,
    },
};

pub fn ensure_stock_row(
    conn: &Connection,
    product_id: i64,
    minimum_quantity: f64,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO stock_levels (product_id, current_quantity, minimum_quantity, updated_at)
         VALUES (?1, 0, ?2, ?3)
         ON CONFLICT(product_id) DO UPDATE SET minimum_quantity = excluded.minimum_quantity",
        params![product_id, minimum_quantity, now_iso()],
    )?;
    Ok(())
}

pub fn current_stock(conn: &Connection, product_id: i64) -> Result<f64, AppError> {
    let value = conn.query_row(
        "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
        [product_id],
        |row| row.get(0),
    );
    match value {
        Ok(quantity) => Ok(quantity),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0.0),
        Err(error) => Err(error.into()),
    }
}

pub fn update_stock(
    conn: &Connection,
    product_id: i64,
    quantity_delta: f64,
    allow_negative_stock: bool,
) -> Result<f64, AppError> {
    let current = current_stock(conn, product_id)?;
    let next = current + quantity_delta;
    if !allow_negative_stock && next < -0.000001 {
        return Err(AppError::insufficient_stock(format!(
            "Not enough stock available. Current stock is {current}."
        )));
    }
    conn.execute(
        "INSERT INTO stock_levels (product_id, current_quantity, minimum_quantity, updated_at)
         VALUES (?1, ?2, 0, ?3)
         ON CONFLICT(product_id) DO UPDATE SET current_quantity = ?2, updated_at = ?3",
        params![product_id, next, now_iso()],
    )?;
    Ok(next)
}

pub fn insert_inventory_transaction(
    conn: &Connection,
    product_id: i64,
    transaction_type: &str,
    reference_type: &str,
    reference_id: Option<i64>,
    quantity_in: f64,
    quantity_out: f64,
    unit_cost_cents: Option<i64>,
    notes: Option<String>,
    created_by: i64,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO inventory_transactions
         (product_id, transaction_type, reference_type, reference_id, quantity_in, quantity_out,
          unit_cost_cents, notes, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            product_id,
            transaction_type,
            reference_type,
            reference_id,
            quantity_in,
            quantity_out,
            unit_cost_cents,
            notes,
            created_by,
            now_iso()
        ],
    )?;
    Ok(())
}

pub fn adjust_stock(
    conn: &Connection,
    user_id: i64,
    payload: StockAdjustmentPayload,
) -> Result<(), AppError> {
    positive_f64(payload.quantity, "Quantity")?;
    let allowed = ["opening_stock", "adjustment_in", "adjustment_out", "damaged_stock"];
    if !allowed.contains(&payload.transaction_type.as_str()) {
        return Err(AppError::validation("Invalid stock adjustment type."));
    }

    let settings = get_company_settings(conn)?;
    let quantity_in = if payload.transaction_type == "opening_stock" || payload.transaction_type == "adjustment_in" {
        payload.quantity
    } else {
        0.0
    };
    let quantity_out = if quantity_in > 0.0 { 0.0 } else { payload.quantity };
    let delta = quantity_in - quantity_out;

    let tx = conn.unchecked_transaction()?;
    update_stock(&tx, payload.product_id, delta, settings.allow_negative_stock)?;
    insert_inventory_transaction(
        &tx,
        payload.product_id,
        &payload.transaction_type,
        "manual",
        None,
        quantity_in,
        quantity_out,
        payload.unit_cost_cents,
        payload.notes,
        user_id,
    )?;
    insert_audit_log(
        &tx,
        user_id,
        "create",
        "inventory_transactions",
        payload.product_id,
        None,
        Some(serde_json::json!({"transaction_type": payload.transaction_type, "quantity": payload.quantity})),
    )?;
    tx.commit()?;
    Ok(())
}

pub fn list_product_movement(
    conn: &Connection,
    product_id: i64,
    filters: MovementFilters,
) -> Result<Vec<InventoryTransactionRow>, AppError> {
    if let Some(date) = filters.date_from.as_deref() {
        validate_date(date, "Start date")?;
    }
    if let Some(date) = filters.date_to.as_deref() {
        validate_date(date, "End date")?;
    }

    let mut stmt = conn.prepare(
        "SELECT it.id, it.product_id, p.name, p.sku, it.transaction_type, it.reference_type,
                it.reference_id, it.quantity_in, it.quantity_out, it.unit_cost_cents,
                it.notes, it.created_at
         FROM inventory_transactions it
         JOIN products p ON p.id = it.product_id
         WHERE it.product_id = ?1
           AND (?2 IS NULL OR date(it.created_at) >= date(?2))
           AND (?3 IS NULL OR date(it.created_at) <= date(?3))
         ORDER BY it.created_at DESC, it.id DESC",
    )?;
    let rows = stmt
        .query_map(
            params![product_id, filters.date_from, filters.date_to],
            map_transaction,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn list_stock_movement(
    conn: &Connection,
    filters: MovementFilters,
) -> Result<Vec<InventoryTransactionRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT it.id, it.product_id, p.name, p.sku, it.transaction_type, it.reference_type,
                it.reference_id, it.quantity_in, it.quantity_out, it.unit_cost_cents,
                it.notes, it.created_at
         FROM inventory_transactions it
         JOIN products p ON p.id = it.product_id
         WHERE (?1 IS NULL OR date(it.created_at) >= date(?1))
           AND (?2 IS NULL OR date(it.created_at) <= date(?2))
         ORDER BY it.created_at DESC, it.id DESC",
    )?;
    let rows = stmt
        .query_map(params![filters.date_from, filters.date_to], map_transaction)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn map_transaction(row: &rusqlite::Row<'_>) -> rusqlite::Result<InventoryTransactionRow> {
    Ok(InventoryTransactionRow {
        id: row.get(0)?,
        product_id: row.get(1)?,
        product_name: row.get(2)?,
        sku: row.get(3)?,
        transaction_type: row.get(4)?,
        reference_type: row.get(5)?,
        reference_id: row.get(6)?,
        quantity_in: row.get(7)?,
        quantity_out: row.get(8)?,
        unit_cost_cents: row.get(9)?,
        notes: row.get(10)?,
        created_at: row.get(11)?,
    })
}
