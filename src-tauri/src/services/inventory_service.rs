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
    conn.query_row(
        "SELECT COALESCE(SUM(quantity_in - quantity_out), 0)
         FROM inventory_transactions
         WHERE product_id = ?1 AND status = 'active'",
        [product_id],
        |row| row.get(0),
    )
    .map_err(Into::into)
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

pub fn recalculate_stock(conn: &Connection, product_id: i64) -> Result<f64, AppError> {
    let quantity: f64 = conn.query_row(
        "SELECT COALESCE(SUM(quantity_in - quantity_out), 0)
         FROM inventory_transactions
         WHERE product_id = ?1 AND status = 'active'",
        [product_id],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO stock_levels (product_id, current_quantity, minimum_quantity, updated_at)
         VALUES (?1, ?2, 0, ?3)
         ON CONFLICT(product_id) DO UPDATE
         SET current_quantity = excluded.current_quantity, updated_at = excluded.updated_at",
        params![product_id, quantity, now_iso()],
    )?;
    Ok(quantity)
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
    let allowed = [
        "opening_stock",
        "adjustment_in",
        "adjustment_out",
        "damaged_stock",
    ];
    if !allowed.contains(&payload.transaction_type.as_str()) {
        return Err(AppError::validation("Invalid stock adjustment type."));
    }

    let settings = get_company_settings(conn)?;
    let quantity_in = if payload.transaction_type == "opening_stock"
        || payload.transaction_type == "adjustment_in"
    {
        payload.quantity
    } else {
        0.0
    };
    let quantity_out = if quantity_in > 0.0 {
        0.0
    } else {
        payload.quantity
    };
    let delta = quantity_in - quantity_out;

    let tx = conn.unchecked_transaction()?;
    update_stock(
        &tx,
        payload.product_id,
        delta,
        settings.allow_negative_stock,
    )?;
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
        Some(
            serde_json::json!({"transaction_type": payload.transaction_type, "quantity": payload.quantity}),
        ),
    )?;
    tx.commit()?;
    Ok(())
}

pub fn cancel_stock_adjustment(
    conn: &Connection,
    user_id: i64,
    transaction_id: i64,
) -> Result<(), AppError> {
    let (product_id, reference_type, status): (i64, String, String) = conn
        .query_row(
            "SELECT product_id, reference_type, status
             FROM inventory_transactions
             WHERE id = ?1",
            [transaction_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::not_found("Inventory transaction not found.")
            }
            other => other.into(),
        })?;
    if status == "cancelled" {
        return Ok(());
    }
    if !["manual", "product"].contains(&reference_type.as_str()) {
        return Err(AppError::validation(
            "Invoice inventory must be cancelled from its invoice.",
        ));
    }

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE inventory_transactions
         SET status = 'cancelled', deleted_at = ?1
         WHERE id = ?2",
        params![now_iso(), transaction_id],
    )?;
    recalculate_stock(&tx, product_id)?;
    insert_audit_log(
        &tx,
        user_id,
        "cancel",
        "inventory_transactions",
        transaction_id,
        None,
        None,
    )?;
    tx.commit()?;
    Ok(())
}

pub fn restore_stock_adjustment(
    conn: &Connection,
    user_id: i64,
    transaction_id: i64,
) -> Result<(), AppError> {
    let (product_id, reference_type, quantity_out, status): (i64, String, f64, String) = conn
        .query_row(
            "SELECT product_id, reference_type, quantity_out, status
             FROM inventory_transactions WHERE id = ?1",
            [transaction_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::not_found("Inventory transaction not found.")
            }
            other => other.into(),
        })?;
    if status == "active" {
        return Ok(());
    }
    if !["manual", "product"].contains(&reference_type.as_str()) {
        return Err(AppError::validation(
            "Restore invoice inventory from its invoice.",
        ));
    }
    let settings = get_company_settings(conn)?;
    if !settings.allow_negative_stock
        && quantity_out > 0.0
        && current_stock(conn, product_id)? + f64::EPSILON < quantity_out
    {
        return Err(AppError::insufficient_stock(
            "Restoring this stock adjustment would make inventory negative.",
        ));
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE inventory_transactions
         SET status = 'active', deleted_at = NULL
         WHERE id = ?1",
        [transaction_id],
    )?;
    recalculate_stock(&tx, product_id)?;
    insert_audit_log(
        &tx,
        user_id,
        "restore",
        "inventory_transactions",
        transaction_id,
        None,
        None,
    )?;
    tx.commit()?;
    Ok(())
}

pub fn permanently_delete_stock_adjustment(
    conn: &Connection,
    user_id: i64,
    transaction_id: i64,
) -> Result<(), AppError> {
    let (reference_type, status): (String, String) = conn
        .query_row(
            "SELECT reference_type, status FROM inventory_transactions WHERE id = ?1",
            [transaction_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::not_found("Inventory transaction not found.")
            }
            other => other.into(),
        })?;
    if status != "cancelled" || !["manual", "product"].contains(&reference_type.as_str()) {
        return Err(AppError::validation(
            "Only a cancelled manual stock adjustment can be permanently deleted.",
        ));
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM inventory_transactions WHERE id = ?1",
        [transaction_id],
    )?;
    insert_audit_log(
        &tx,
        user_id,
        "delete",
        "inventory_transactions",
        transaction_id,
        None,
        None,
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
                it.notes, it.status, it.created_at, it.deleted_at
         FROM inventory_transactions it
         JOIN products p ON p.id = it.product_id
         WHERE it.product_id = ?1
           AND (?2 IS NULL OR date(it.created_at) >= date(?2))
           AND (?3 IS NULL OR date(it.created_at) <= date(?3))
           AND (?4 = 0 OR it.status = 'active')
         ORDER BY it.created_at DESC, it.id DESC",
    )?;
    let rows = stmt
        .query_map(
            params![
                product_id,
                filters.date_from,
                filters.date_to,
                if filters.active_only.unwrap_or(false) {
                    1
                } else {
                    0
                }
            ],
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
                it.notes, it.status, it.created_at, it.deleted_at
         FROM inventory_transactions it
         JOIN products p ON p.id = it.product_id
         WHERE (?1 IS NULL OR date(it.created_at) >= date(?1))
           AND (?2 IS NULL OR date(it.created_at) <= date(?2))
           AND (?3 = 0 OR it.status = 'active')
         ORDER BY it.created_at DESC, it.id DESC",
    )?;
    let rows = stmt
        .query_map(
            params![
                filters.date_from,
                filters.date_to,
                if filters.active_only.unwrap_or(false) {
                    1
                } else {
                    0
                }
            ],
            map_transaction,
        )?
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
        status: row.get(11)?,
        created_at: row.get(12)?,
        deleted_at: row.get(13)?,
    })
}
