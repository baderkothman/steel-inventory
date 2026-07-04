use rusqlite::{params, Connection};

use crate::{
    models::DemoSeedResult,
    utils::{audit::insert_audit_log, dates::{now_iso, today_date}, errors::AppError},
};

pub fn seed_demo_data(conn: &mut Connection, user_id: i64) -> Result<DemoSeedResult, AppError> {
    let existing: i64 = conn.query_row(
        "SELECT COUNT(*) FROM products WHERE sku LIKE 'DEMO-%'",
        [],
        |row| row.get(0),
    )?;
    if existing > 0 {
        return Ok(DemoSeedResult {
            inserted: false,
            message: "Demo data is already present. No duplicate rows were inserted.".to_string(),
        });
    }

    let now = now_iso();
    let today = today_date();
    let tx = conn.transaction()?;

    let supplier_metal = insert_supplier(
        &tx,
        "Demo Metal Supply",
        Some("Demo Metal Supply SAL"),
        Some("+961 1 555 100"),
        Some("sales@demometal.test"),
        Some("Industrial Zone, Beirut"),
        Some("TX-DEMO-SUP-01"),
        150_000,
        Some("Opening balance from old spreadsheet."),
        &now,
    )?;
    let supplier_tools = insert_supplier(
        &tx,
        "Demo Tools Import",
        Some("Demo Tools Import Co."),
        Some("+961 3 222 889"),
        None,
        Some("Tripoli Road"),
        None,
        0,
        None,
        &now,
    )?;
    let supplier_transport = insert_supplier(
        &tx,
        "Demo Transport Supplier",
        None,
        Some("+961 70 441 220"),
        None,
        None,
        None,
        45_000,
        Some("Used for delivery-related stock purchases."),
        &now,
    )?;

    let customer_builders = insert_customer(
        &tx,
        "Demo Builders",
        Some("Demo Builders Contracting"),
        Some("+961 1 700 201"),
        Some("accounts@demobuilders.test"),
        Some("Hamra, Beirut"),
        Some("TX-DEMO-CUS-01"),
        80_000,
        Some("Usually pays partial invoices."),
        &now,
    )?;
    let customer_workshop = insert_customer(
        &tx,
        "Demo Workshop",
        Some("Workshop 88"),
        Some("+961 76 808 808"),
        None,
        Some("Saida"),
        None,
        0,
        None,
        &now,
    )?;
    let customer_cash = insert_customer(
        &tx,
        "Demo Cash Customer",
        None,
        Some("+961 81 111 333"),
        None,
        None,
        None,
        0,
        Some("Mostly cash sales."),
        &now,
    )?;

    let p_pipe_sq = insert_product(
        &tx,
        "DEMO-GSP-SQ-20X20-0.5",
        4,
        "Demo Galvanized Square Pipe 20x20 0.5mm",
        "pipe",
        "galvanized steel",
        "square",
        "galvanized",
        Some("20x20"),
        Some(20.0),
        Some(20.0),
        None,
        Some(0.5),
        Some(6000.0),
        "piece",
        Some("Common lightweight galvanized square pipe."),
        1700,
        2400,
        2200,
        25.0,
        &now,
    )?;
    let p_pipe_round = insert_product(
        &tx,
        "DEMO-BSP-RD-30-1.5",
        7,
        "Demo Black Round Pipe 30mm 1.5mm",
        "pipe",
        "steel",
        "round",
        "black",
        Some("30mm"),
        None,
        None,
        Some(30.0),
        Some(1.5),
        Some(6000.0),
        "piece",
        None,
        3800,
        5200,
        4900,
        12.0,
        &now,
    )?;
    let p_sheet_galv = insert_product(
        &tx,
        "DEMO-GS-SHEET-1.0",
        9,
        "Demo Galvanized Sheet 1.0mm",
        "sheet",
        "galvanized steel",
        "flat",
        "galvanized",
        Some("1000x2000"),
        Some(1000.0),
        Some(2000.0),
        None,
        Some(1.0),
        None,
        "sheet",
        Some("Standard sheet size."),
        24000,
        31500,
        29500,
        8.0,
        &now,
    )?;
    let p_sheet_ss = insert_product(
        &tx,
        "DEMO-SS-SHEET-2.0",
        11,
        "Demo Stainless Steel Sheet 2.0mm",
        "sheet",
        "stainless steel",
        "flat",
        "stainless",
        Some("1220x2440"),
        Some(1220.0),
        Some(2440.0),
        None,
        Some(2.0),
        None,
        "sheet",
        None,
        95000,
        125000,
        118000,
        3.0,
        &now,
    )?;
    let p_rebar = insert_product(
        &tx,
        "DEMO-REBAR-12MM",
        21,
        "Demo Rebar 12mm",
        "bar",
        "steel",
        "round",
        "black",
        Some("12mm"),
        None,
        None,
        Some(12.0),
        None,
        Some(12000.0),
        "piece",
        None,
        420,
        600,
        560,
        40.0,
        &now,
    )?;
    let p_angle = insert_product(
        &tx,
        "DEMO-ANGLE-40X40",
        17,
        "Demo Angle Bar 40x40",
        "bar",
        "steel",
        "angle",
        "black",
        Some("40x40"),
        Some(40.0),
        Some(40.0),
        None,
        Some(3.0),
        Some(6000.0),
        "piece",
        Some("Used for frames and supports."),
        2600,
        3700,
        3400,
        15.0,
        &now,
    )?;
    let p_welder = insert_product(
        &tx,
        "DEMO-WELDER-MIG-250",
        23,
        "Demo MIG Welder 250A",
        "equipment",
        "steel",
        "box",
        "painted",
        Some("250A"),
        None,
        None,
        None,
        None,
        None,
        "piece",
        Some("Equipment item with low quantity."),
        250000,
        330000,
        315000,
        2.0,
        &now,
    )?;
    let p_bolt = insert_product(
        &tx,
        "DEMO-BOLT-M10",
        22,
        "Demo Bolt M10",
        "accessory",
        "steel",
        "round",
        "galvanized",
        Some("M10"),
        None,
        None,
        Some(10.0),
        None,
        None,
        "piece",
        None,
        35,
        65,
        55,
        250.0,
        &now,
    )?;

    // Assign seeded products to their supplier and populate spec_key (matches migration 003).
    tx.execute(
        "UPDATE products SET supplier_id = ?1 WHERE supplier_id IS NULL AND sku LIKE 'DEMO-%'",
        params![supplier_metal],
    )?;
    tx.execute(
        "UPDATE products
         SET spec_key = upper(
                 trim(product_type) || '|' || trim(material) || '|' || trim(shape) || '|' ||
                 trim(finish) || '|' || COALESCE(trim(size_label), '') || '|' ||
                 COALESCE(CAST(thickness_mm AS TEXT), ''))
         WHERE (spec_key = '' OR spec_key IS NULL) AND sku LIKE 'DEMO-%'",
        [],
    )?;

    // Same round-pipe specification from a second supplier at a lower price,
    // so the cheapest-supplier comparison has demo data out of the box.
    let p_pipe_round_alt = insert_product(
        &tx,
        "DEMO-BSP-RD-30-1.5-ALT",
        7,
        "Demo Black Round Pipe 30mm 1.5mm",
        "pipe",
        "steel",
        "round",
        "black",
        Some("30mm"),
        None,
        None,
        Some(30.0),
        Some(1.5),
        Some(6000.0),
        "piece",
        Some("Alternative supplier variant for cheapest comparison."),
        3500,
        4900,
        4600,
        12.0,
        &now,
    )?;
    tx.execute(
        "UPDATE products
         SET supplier_id = ?1,
             spec_key = (SELECT spec_key FROM products WHERE id = ?2)
         WHERE id = ?3",
        params![supplier_tools, p_pipe_round, p_pipe_round_alt],
    )?;

    insert_purchase(
        &tx,
        user_id,
        supplier_metal,
        "PI-DEMO-1001",
        &today,
        25_000,
        0,
        15_000,
        120_000,
        Some("Main pipe and sheet stock-in."),
        &[
            (p_pipe_sq, 100.0, 1700),
            (p_pipe_round, 60.0, 3800),
            (p_sheet_galv, 30.0, 24000),
        ],
        &now,
    )?;
    insert_purchase(
        &tx,
        user_id,
        supplier_transport,
        "PI-DEMO-1002",
        &today,
        0,
        0,
        8_000,
        150_000,
        None,
        &[(p_rebar, 200.0, 420), (p_angle, 80.0, 2600)],
        &now,
    )?;
    insert_purchase(
        &tx,
        user_id,
        supplier_tools,
        "PI-DEMO-1003",
        &today,
        10_000,
        0,
        0,
        0,
        Some("Equipment and accessories, unpaid."),
        &[(p_welder, 4.0, 250000), (p_bolt, 1000.0, 35), (p_sheet_ss, 6.0, 95000)],
        &now,
    )?;

    insert_sale(
        &tx,
        user_id,
        Some(customer_builders),
        "SI-DEMO-1001",
        &today,
        3_000,
        0,
        5_000,
        20_000,
        Some("Partial payment from contractor."),
        &[(p_pipe_sq, 12.0, 2400), (p_rebar, 20.0, 600)],
        &now,
    )?;
    insert_sale(
        &tx,
        user_id,
        None,
        "SI-DEMO-1002",
        &today,
        0,
        0,
        0,
        6_500,
        Some("Walk-in cash customer."),
        &[(p_bolt, 100.0, 65)],
        &now,
    )?;
    insert_sale(
        &tx,
        user_id,
        Some(customer_workshop),
        "SI-DEMO-1003",
        &today,
        5_000,
        0,
        0,
        0,
        None,
        &[(p_sheet_galv, 5.0, 31500), (p_angle, 8.0, 3700)],
        &now,
    )?;
    insert_sale(
        &tx,
        user_id,
        Some(customer_cash),
        "SI-DEMO-1004",
        &today,
        0,
        0,
        10_000,
        330_000,
        Some("Paid equipment sale with delivery."),
        &[(p_welder, 1.0, 330000)],
        &now,
    )?;

    insert_expense(&tx, user_id, 1, "Demo shop rent", 120_000, "USD", &today, "bank", Some("Monthly rent."), &now)?;
    insert_expense(&tx, user_id, 2, "Demo electricity bill", 36_500, "USD", &today, "cash", None, &now)?;
    insert_expense(&tx, user_id, 4, "Demo local delivery", 18_000, "USD", &today, "cash", Some("Delivery for SI-DEMO-1004."), &now)?;
    insert_expense(&tx, user_id, 6, "Demo machine maintenance", 52_000, "USD", &today, "card", None, &now)?;
    insert_expense(&tx, user_id, 8, "Demo packaging material", 7_500, "USD", &today, "cash", None, &now)?;

    insert_general_payment(&tx, user_id, "customer", customer_builders, "in", 30_000, "USD", "bank", &today, Some("Extra payment against old balance."), &now)?;
    insert_general_payment(&tx, user_id, "supplier", supplier_metal, "out", 50_000, "USD", "bank", &today, Some("General payment to supplier."), &now)?;

    tx.execute(
        "INSERT INTO backups (backup_path, backup_type, status, notes, created_at)
         VALUES (?1, 'manual', 'success', ?2, ?3)",
        params![
            "C:\\Users\\Public\\Documents\\SteelInventoryBackups\\steel_inventory_backup_demo.db",
            "Demo backup log row only; no file is created by demo seed.",
            now
        ],
    )?;

    insert_audit_log(
        &tx,
        user_id,
        "create",
        "demo_seed",
        0,
        None,
        Some(serde_json::json!({"products": 8, "suppliers": 3, "customers": 3, "purchases": 3, "sales": 4})),
    )?;

    tx.commit()?;

    Ok(DemoSeedResult {
        inserted: true,
        message: "Demo data inserted across products, parties, invoices, stock, expenses, payments, reports, and backup logs.".to_string(),
    })
}

fn insert_supplier(
    conn: &Connection,
    name: &str,
    company_name: Option<&str>,
    phone: Option<&str>,
    email: Option<&str>,
    address: Option<&str>,
    tax_number: Option<&str>,
    opening_balance_cents: i64,
    notes: Option<&str>,
    now: &str,
) -> Result<i64, AppError> {
    conn.execute(
        "INSERT INTO suppliers (name, company_name, phone, email, address, tax_number, opening_balance_cents, notes, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?9)",
        params![name, company_name, phone, email, address, tax_number, opening_balance_cents, notes, now],
    )?;
    Ok(conn.last_insert_rowid())
}

fn insert_customer(
    conn: &Connection,
    name: &str,
    company_name: Option<&str>,
    phone: Option<&str>,
    email: Option<&str>,
    address: Option<&str>,
    tax_number: Option<&str>,
    opening_balance_cents: i64,
    notes: Option<&str>,
    now: &str,
) -> Result<i64, AppError> {
    conn.execute(
        "INSERT INTO customers (name, company_name, phone, email, address, tax_number, opening_balance_cents, notes, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?9)",
        params![name, company_name, phone, email, address, tax_number, opening_balance_cents, notes, now],
    )?;
    Ok(conn.last_insert_rowid())
}

#[allow(clippy::too_many_arguments)]
fn insert_product(
    conn: &Connection,
    sku: &str,
    category_id: i64,
    name: &str,
    product_type: &str,
    material: &str,
    shape: &str,
    finish: &str,
    size_label: Option<&str>,
    width_mm: Option<f64>,
    height_mm: Option<f64>,
    diameter_mm: Option<f64>,
    thickness_mm: Option<f64>,
    length_mm: Option<f64>,
    unit: &str,
    description: Option<&str>,
    cost_price_cents: i64,
    selling_price_cents: i64,
    wholesale_price_cents: i64,
    minimum_quantity: f64,
    now: &str,
) -> Result<i64, AppError> {
    conn.execute(
        "INSERT INTO products (sku, category_id, name, product_type, material, shape, finish, size_label, width_mm, height_mm, diameter_mm, thickness_mm, length_mm, unit, description, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, 1, ?16, ?16)",
        params![sku, category_id, name, product_type, material, shape, finish, size_label, width_mm, height_mm, diameter_mm, thickness_mm, length_mm, unit, description, now],
    )?;
    let product_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO product_prices (product_id, cost_price_cents, selling_price_cents, wholesale_price_cents, currency, effective_from, created_at)
         VALUES (?1, ?2, ?3, ?4, 'USD', ?5, ?5)",
        params![product_id, cost_price_cents, selling_price_cents, wholesale_price_cents, now],
    )?;
    conn.execute(
        "INSERT INTO stock_levels (product_id, current_quantity, minimum_quantity, updated_at)
         VALUES (?1, 0, ?2, ?3)",
        params![product_id, minimum_quantity, now],
    )?;
    Ok(product_id)
}

#[allow(clippy::too_many_arguments)]
fn insert_purchase(
    conn: &Connection,
    user_id: i64,
    supplier_id: i64,
    invoice_number: &str,
    date: &str,
    discount_cents: i64,
    tax_cents: i64,
    shipping_cents: i64,
    paid_cents: i64,
    notes: Option<&str>,
    items: &[(i64, f64, i64)],
    now: &str,
) -> Result<i64, AppError> {
    let subtotal = items.iter().map(|(_, qty, cost)| (qty * *cost as f64).round() as i64).sum::<i64>();
    let total = subtotal - discount_cents + tax_cents + shipping_cents;
    let remaining = total - paid_cents;
    let status = payment_status(total, paid_cents);
    conn.execute(
        "INSERT INTO purchase_invoices (supplier_id, invoice_number, invoice_date, subtotal_cents, discount_cents, tax_cents, shipping_cents, total_cents, paid_cents, remaining_cents, payment_status, status, notes, created_by, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'active', ?12, ?13, ?14, ?14)",
        params![supplier_id, invoice_number, date, subtotal, discount_cents, tax_cents, shipping_cents, total, paid_cents, remaining, status, notes, user_id, now],
    )?;
    let invoice_id = conn.last_insert_rowid();
    for (product_id, qty, unit_cost) in items {
        let total_cost = (*qty * *unit_cost as f64).round() as i64;
        conn.execute(
            "INSERT INTO purchase_invoice_items (purchase_invoice_id, product_id, quantity, unit_cost_cents, total_cost_cents, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![invoice_id, product_id, qty, unit_cost, total_cost, now],
        )?;
        conn.execute(
            "UPDATE stock_levels SET current_quantity = current_quantity + ?1, updated_at = ?2 WHERE product_id = ?3",
            params![qty, now, product_id],
        )?;
        conn.execute(
            "INSERT INTO inventory_transactions (product_id, transaction_type, reference_type, reference_id, quantity_in, quantity_out, unit_cost_cents, notes, created_by, created_at)
             VALUES (?1, 'purchase', 'purchase_invoice', ?2, ?3, 0, ?4, ?5, ?6, ?7)",
            params![product_id, invoice_id, qty, unit_cost, format!("Demo purchase {invoice_number}"), user_id, now],
        )?;
    }
    if paid_cents > 0 {
        conn.execute(
            "INSERT INTO payments (party_type, party_id, payment_direction, amount_cents, currency, payment_method, payment_date, reference_type, reference_id, notes, created_by, created_at)
             VALUES ('supplier', ?1, 'out', ?2, 'USD', 'bank', ?3, 'purchase_invoice', ?4, ?5, ?6, ?7)",
            params![supplier_id, paid_cents, date, invoice_id, format!("Demo payment with {invoice_number}"), user_id, now],
        )?;
    }
    Ok(invoice_id)
}

#[allow(clippy::too_many_arguments)]
fn insert_sale(
    conn: &Connection,
    user_id: i64,
    customer_id: Option<i64>,
    invoice_number: &str,
    date: &str,
    discount_cents: i64,
    tax_cents: i64,
    delivery_cents: i64,
    paid_cents: i64,
    notes: Option<&str>,
    items: &[(i64, f64, i64)],
    now: &str,
) -> Result<i64, AppError> {
    let subtotal = items.iter().map(|(_, qty, price)| (qty * *price as f64).round() as i64).sum::<i64>();
    let total = subtotal - discount_cents + tax_cents + delivery_cents;
    let remaining = total - paid_cents;
    let status = payment_status(total, paid_cents);
    conn.execute(
        "INSERT INTO sales_invoices (customer_id, invoice_number, invoice_date, subtotal_cents, discount_cents, tax_cents, delivery_cents, total_cents, paid_cents, remaining_cents, payment_status, sales_status, notes, created_by, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'completed', ?12, ?13, ?14, ?14)",
        params![customer_id, invoice_number, date, subtotal, discount_cents, tax_cents, delivery_cents, total, paid_cents, remaining, status, notes, user_id, now],
    )?;
    let invoice_id = conn.last_insert_rowid();
    for (product_id, qty, unit_price) in items {
        let unit_cost: i64 = conn.query_row(
            "SELECT cost_price_cents FROM product_prices WHERE product_id = ?1 ORDER BY effective_from DESC, id DESC LIMIT 1",
            [product_id],
            |row| row.get(0),
        )?;
        let total_price = (*qty * *unit_price as f64).round() as i64;
        let total_cost = (*qty * unit_cost as f64).round() as i64;
        conn.execute(
            "INSERT INTO sales_invoice_items (sales_invoice_id, product_id, quantity, unit_cost_cents, unit_price_cents, total_cost_cents, total_price_cents, profit_cents, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![invoice_id, product_id, qty, unit_cost, unit_price, total_cost, total_price, total_price - total_cost, now],
        )?;
        conn.execute(
            "UPDATE stock_levels SET current_quantity = current_quantity - ?1, updated_at = ?2 WHERE product_id = ?3",
            params![qty, now, product_id],
        )?;
        conn.execute(
            "INSERT INTO inventory_transactions (product_id, transaction_type, reference_type, reference_id, quantity_in, quantity_out, unit_cost_cents, notes, created_by, created_at)
             VALUES (?1, 'sale', 'sales_invoice', ?2, 0, ?3, ?4, ?5, ?6, ?7)",
            params![product_id, invoice_id, qty, unit_cost, format!("Demo sale {invoice_number}"), user_id, now],
        )?;
    }
    if paid_cents > 0 {
        if let Some(customer_id) = customer_id {
            conn.execute(
                "INSERT INTO payments (party_type, party_id, payment_direction, amount_cents, currency, payment_method, payment_date, reference_type, reference_id, notes, created_by, created_at)
                 VALUES ('customer', ?1, 'in', ?2, 'USD', 'cash', ?3, 'sales_invoice', ?4, ?5, ?6, ?7)",
                params![customer_id, paid_cents, date, invoice_id, format!("Demo payment with {invoice_number}"), user_id, now],
            )?;
        }
    }
    Ok(invoice_id)
}

fn insert_expense(
    conn: &Connection,
    user_id: i64,
    category_id: i64,
    title: &str,
    amount_cents: i64,
    currency: &str,
    date: &str,
    method: &str,
    notes: Option<&str>,
    now: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO expenses (expense_category_id, title, amount_cents, currency, expense_date, payment_method, notes, created_by, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![category_id, title, amount_cents, currency, date, method, notes, user_id, now],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn insert_general_payment(
    conn: &Connection,
    user_id: i64,
    party_type: &str,
    party_id: i64,
    direction: &str,
    amount_cents: i64,
    currency: &str,
    method: &str,
    date: &str,
    notes: Option<&str>,
    now: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO payments (party_type, party_id, payment_direction, amount_cents, currency, payment_method, payment_date, reference_type, reference_id, notes, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, NULL, ?8, ?9, ?10)",
        params![party_type, party_id, direction, amount_cents, currency, method, date, notes, user_id, now],
    )?;
    Ok(())
}

fn payment_status(total_cents: i64, paid_cents: i64) -> &'static str {
    if paid_cents <= 0 {
        "unpaid"
    } else if paid_cents >= total_cents {
        "paid"
    } else {
        "partial"
    }
}
