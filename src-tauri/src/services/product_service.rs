use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    models::{InventoryTransactionRow, MovementFilters, ProductFilters, ProductPayload, ProductRow},
    services::{
        inventory_service::{ensure_stock_row, insert_inventory_transaction, list_product_movement, update_stock},
        settings_service::get_company_settings,
    },
    utils::{
        audit::insert_audit_log,
        dates::now_iso,
        errors::AppError,
        sku::generate_sku_from_product,
        validation::{non_negative_i64, optional_positive, required},
    },
};

pub fn generate_sku(payload: ProductPayload) -> Result<String, AppError> {
    validate_product_payload(&payload)?;
    Ok(resolve_sku(&payload))
}

pub fn list_products(conn: &Connection, filters: ProductFilters) -> Result<Vec<ProductRow>, AppError> {
    let search = filters
        .search
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let active_only = filters.active_only.unwrap_or(false);

    let mut stmt = conn.prepare(
        "SELECT p.id, p.sku, p.category_id, c.name, p.name, p.product_type, p.material,
                p.shape, p.finish, p.size_label, p.width_mm, p.height_mm, p.diameter_mm,
                p.thickness_mm, p.length_mm, p.unit, p.description, p.is_active,
                COALESCE(pp.cost_price_cents, 0), COALESCE(pp.selling_price_cents, 0),
                COALESCE(pp.wholesale_price_cents, 0),
                COALESCE(sl.current_quantity, 0), COALESCE(sl.minimum_quantity, 0),
                p.created_at, p.updated_at
         FROM products p
         JOIN categories c ON c.id = p.category_id
         LEFT JOIN stock_levels sl ON sl.product_id = p.id
         LEFT JOIN product_prices pp ON pp.id = (
             SELECT id FROM product_prices
             WHERE product_id = p.id
             ORDER BY effective_from DESC, id DESC
             LIMIT 1
         )
         WHERE (?1 IS NULL OR (
             p.name LIKE '%' || ?1 || '%' OR
             p.sku LIKE '%' || ?1 || '%' OR
             p.size_label LIKE '%' || ?1 || '%' OR
             p.material LIKE '%' || ?1 || '%' OR
             CAST(p.thickness_mm AS TEXT) LIKE '%' || ?1 || '%'
         ))
           AND (?2 IS NULL OR p.category_id = ?2)
           AND (?3 = 0 OR p.is_active = 1)
         ORDER BY p.name ASC",
    )?;
    let rows = stmt
        .query_map(params![search, filters.category_id, if active_only { 1 } else { 0 }], map_product)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_product(conn: &Connection, id: i64) -> Result<ProductRow, AppError> {
    let products = list_products(
        conn,
        ProductFilters {
            search: None,
            category_id: None,
            active_only: Some(false),
        },
    )?;
    products
        .into_iter()
        .find(|product| product.id == id)
        .ok_or_else(|| AppError::not_found("Product not found."))
}

pub fn create_product(
    conn: &Connection,
    user_id: i64,
    payload: ProductPayload,
) -> Result<ProductRow, AppError> {
    validate_product_payload(&payload)?;
    let sku = resolve_sku(&payload);
    ensure_unique_sku(conn, &sku, None)?;
    let settings = get_company_settings(conn)?;
    let now = now_iso();
    let wholesale = payload.wholesale_price_cents.unwrap_or(0);
    let initial_quantity = payload.initial_quantity.unwrap_or(0.0);
    if initial_quantity < 0.0 {
        return Err(AppError::validation("Initial quantity must be zero or greater."));
    }

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO products
         (sku, category_id, name, product_type, material, shape, finish, size_label,
          width_mm, height_mm, diameter_mm, thickness_mm, length_mm, unit, description,
          is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, 1, ?16, ?16)",
        params![
            sku,
            payload.category_id,
            payload.name.trim(),
            payload.product_type.trim(),
            payload.material.trim(),
            payload.shape.trim(),
            payload.finish.trim(),
            payload.size_label.trim(),
            payload.width_mm,
            payload.height_mm,
            payload.diameter_mm,
            payload.thickness_mm,
            payload.length_mm,
            payload.unit.trim(),
            payload.description,
            now
        ],
    )?;
    let product_id = tx.last_insert_rowid();
    ensure_stock_row(&tx, product_id, payload.minimum_quantity)?;
    tx.execute(
        "INSERT INTO product_prices
         (product_id, cost_price_cents, selling_price_cents, wholesale_price_cents, currency, effective_from, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![
            product_id,
            payload.cost_price_cents,
            payload.selling_price_cents,
            wholesale,
            settings.default_currency,
            now
        ],
    )?;
    if initial_quantity > 0.0 {
        update_stock(&tx, product_id, initial_quantity, true)?;
        insert_inventory_transaction(
            &tx,
            product_id,
            "opening_stock",
            "product",
            Some(product_id),
            initial_quantity,
            0.0,
            Some(payload.cost_price_cents),
            Some("Initial product stock".to_string()),
            user_id,
        )?;
    }
    insert_audit_log(
        &tx,
        user_id,
        "create",
        "products",
        product_id,
        None,
        Some(serde_json::json!({"id": product_id, "sku": sku})),
    )?;
    tx.commit()?;
    get_product(conn, product_id)
}

pub fn update_product(
    conn: &Connection,
    user_id: i64,
    id: i64,
    payload: ProductPayload,
) -> Result<ProductRow, AppError> {
    validate_product_payload(&payload)?;
    let sku = resolve_sku(&payload);
    ensure_product_exists(conn, id)?;
    ensure_unique_sku(conn, &sku, Some(id))?;
    let settings = get_company_settings(conn)?;
    let wholesale = payload.wholesale_price_cents.unwrap_or(0);
    let now = now_iso();

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE products
         SET sku = ?1, category_id = ?2, name = ?3, product_type = ?4, material = ?5,
             shape = ?6, finish = ?7, size_label = ?8, width_mm = ?9, height_mm = ?10,
             diameter_mm = ?11, thickness_mm = ?12, length_mm = ?13, unit = ?14,
             description = ?15, updated_at = ?16
         WHERE id = ?17",
        params![
            sku,
            payload.category_id,
            payload.name.trim(),
            payload.product_type.trim(),
            payload.material.trim(),
            payload.shape.trim(),
            payload.finish.trim(),
            payload.size_label.trim(),
            payload.width_mm,
            payload.height_mm,
            payload.diameter_mm,
            payload.thickness_mm,
            payload.length_mm,
            payload.unit.trim(),
            payload.description,
            now,
            id
        ],
    )?;
    ensure_stock_row(&tx, id, payload.minimum_quantity)?;
    let latest_price: Option<(i64, i64, i64)> = tx
        .query_row(
            "SELECT cost_price_cents, selling_price_cents, wholesale_price_cents
             FROM product_prices
             WHERE product_id = ?1
             ORDER BY effective_from DESC, id DESC
             LIMIT 1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    if latest_price != Some((payload.cost_price_cents, payload.selling_price_cents, wholesale)) {
        tx.execute(
            "INSERT INTO product_prices
             (product_id, cost_price_cents, selling_price_cents, wholesale_price_cents, currency, effective_from, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                id,
                payload.cost_price_cents,
                payload.selling_price_cents,
                wholesale,
                settings.default_currency,
                now
            ],
        )?;
    }
    insert_audit_log(
        &tx,
        user_id,
        "update",
        "products",
        id,
        None,
        Some(serde_json::json!({"id": id, "sku": sku})),
    )?;
    tx.commit()?;
    get_product(conn, id)
}

pub fn archive_product(conn: &Connection, user_id: i64, id: i64) -> Result<(), AppError> {
    ensure_product_exists(conn, id)?;
    conn.execute(
        "UPDATE products SET is_active = 0, updated_at = ?1 WHERE id = ?2",
        params![now_iso(), id],
    )?;
    insert_audit_log(conn, user_id, "archive", "products", id, None, None)?;
    Ok(())
}

pub fn product_stock(conn: &Connection, product_id: i64) -> Result<f64, AppError> {
    crate::services::inventory_service::current_stock(conn, product_id)
}

pub fn product_movement(
    conn: &Connection,
    product_id: i64,
    filters: MovementFilters,
) -> Result<Vec<InventoryTransactionRow>, AppError> {
    list_product_movement(conn, product_id, filters)
}

pub fn latest_price(conn: &Connection, product_id: i64) -> Result<(i64, i64, i64), AppError> {
    conn.query_row(
        "SELECT cost_price_cents, selling_price_cents, wholesale_price_cents
         FROM product_prices
         WHERE product_id = ?1
         ORDER BY effective_from DESC, id DESC
         LIMIT 1",
        [product_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => AppError::not_found("Product price not found."),
        other => other.into(),
    })
}

fn validate_product_payload(payload: &ProductPayload) -> Result<(), AppError> {
    required(&payload.name, "Product name")?;
    required(&payload.product_type, "Product type")?;
    required(&payload.material, "Material")?;
    required(&payload.shape, "Shape")?;
    required(&payload.finish, "Finish")?;
    required(&payload.unit, "Unit")?;
    non_negative_i64(payload.cost_price_cents, "Cost price")?;
    non_negative_i64(payload.selling_price_cents, "Selling price")?;
    non_negative_i64(payload.wholesale_price_cents.unwrap_or(0), "Wholesale price")?;
    optional_positive(payload.width_mm, "Width")?;
    optional_positive(payload.height_mm, "Height")?;
    optional_positive(payload.diameter_mm, "Diameter")?;
    optional_positive(payload.thickness_mm, "Thickness")?;
    optional_positive(payload.length_mm, "Length")?;
    if payload.minimum_quantity < 0.0 {
        return Err(AppError::validation("Minimum stock quantity must be zero or greater."));
    }
    Ok(())
}

fn resolve_sku(payload: &ProductPayload) -> String {
    payload
        .sku
        .as_ref()
        .map(|value| value.trim().to_uppercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| generate_sku_from_product(payload))
}

fn ensure_unique_sku(conn: &Connection, sku: &str, excluded_id: Option<i64>) -> Result<(), AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM products WHERE sku = ?1 AND (?2 IS NULL OR id <> ?2)",
        params![sku, excluded_id],
        |row| row.get(0),
    )?;
    if count > 0 {
        Err(AppError::duplicate_sku())
    } else {
        Ok(())
    }
}

fn ensure_product_exists(conn: &Connection, id: i64) -> Result<(), AppError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM products WHERE id = ?1", [id], |row| {
        row.get(0)
    })?;
    if count == 0 {
        Err(AppError::not_found("Product not found."))
    } else {
        Ok(())
    }
}

fn map_product(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProductRow> {
    Ok(ProductRow {
        id: row.get(0)?,
        sku: row.get(1)?,
        category_id: row.get(2)?,
        category_name: row.get(3)?,
        name: row.get(4)?,
        product_type: row.get(5)?,
        material: row.get(6)?,
        shape: row.get(7)?,
        finish: row.get(8)?,
        size_label: row.get(9)?,
        width_mm: row.get(10)?,
        height_mm: row.get(11)?,
        diameter_mm: row.get(12)?,
        thickness_mm: row.get(13)?,
        length_mm: row.get(14)?,
        unit: row.get(15)?,
        description: row.get(16)?,
        is_active: row.get::<_, i64>(17)? == 1,
        cost_price_cents: row.get(18)?,
        selling_price_cents: row.get(19)?,
        wholesale_price_cents: row.get(20)?,
        current_quantity: row.get(21)?,
        minimum_quantity: row.get(22)?,
        created_at: row.get(23)?,
        updated_at: row.get(24)?,
    })
}
