use rusqlite::Connection;

use crate::{
    db::migrations::run_migrations,
    models::{
        PartyPayload, ProductPayload, PurchaseInvoicePayload, PurchaseItemPayload, SalesInvoicePayload,
        SalesItemPayload, SetupAdminPayload,
    },
    services::{
        auth_service::setup_admin,
        party_service::{create_party, PartyKind},
        product_service::create_product,
        purchase_service::create_purchase_invoice,
        sales_service::create_sales_invoice,
        seed_service::seed_demo_data,
    },
    utils::{
        dates::today_date,
        money::{checked_total, payment_status},
        sku::generate_sku_from_product,
    },
};

fn test_conn() -> Connection {
    let mut conn = Connection::open_in_memory().expect("open in-memory sqlite");
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    run_migrations(&mut conn).expect("run migrations");
    conn
}

#[test]
fn money_totals_and_payment_status_are_stable() {
    assert_eq!(checked_total(10_000, 500, 0, 250).unwrap(), 9_750);
    assert_eq!(payment_status(10_000, 0), "unpaid");
    assert_eq!(payment_status(10_000, 4_000), "partial");
    assert_eq!(payment_status(10_000, 10_000), "paid");
}

#[test]
fn sku_generation_uses_steel_attributes() {
    let payload = ProductPayload {
        sku: None,
        category_id: 4,
        name: "Galvanized Square Pipe 20x20 0.5mm".to_string(),
        product_type: "pipe".to_string(),
        material: "steel".to_string(),
        shape: "square".to_string(),
        finish: "galvanized".to_string(),
        size_label: "20x20".to_string(),
        width_mm: Some(20.0),
        height_mm: Some(20.0),
        diameter_mm: None,
        thickness_mm: Some(0.5),
        length_mm: None,
        unit: "piece".to_string(),
        description: None,
        cost_price_cents: 1700,
        selling_price_cents: 2200,
        wholesale_price_cents: Some(2000),
        minimum_quantity: 5.0,
        initial_quantity: Some(0.0),
    };
    assert_eq!(generate_sku_from_product(&payload), "GSP-SQ-20X20-0.5");
}

#[test]
fn migrations_create_required_seed_data() {
    let conn = test_conn();
    let expense_categories: i64 = conn
        .query_row("SELECT COUNT(*) FROM expense_categories", [], |row| row.get(0))
        .unwrap();
    let settings: i64 = conn
        .query_row("SELECT COUNT(*) FROM company_settings WHERE id = 1", [], |row| row.get(0))
        .unwrap();
    let categories: i64 = conn
        .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
        .unwrap();
    assert_eq!(expense_categories, 9);
    assert_eq!(settings, 1);
    assert!(categories >= 20);
}

#[test]
fn purchase_then_sale_updates_stock_and_profit_snapshots() {
    let conn = test_conn();
    let user = setup_admin(
        &conn,
        SetupAdminPayload {
            full_name: "Admin".to_string(),
            email: "admin@example.com".to_string(),
            password: "1234".to_string(),
        },
    )
    .unwrap();

    let supplier = create_party(
        &conn,
        user.id,
        PartyKind::Supplier,
        PartyPayload {
            name: "Supplier".to_string(),
            company_name: None,
            phone: None,
            email: None,
            address: None,
            tax_number: None,
            opening_balance_cents: 0,
            notes: None,
        },
    )
    .unwrap();
    let customer = create_party(
        &conn,
        user.id,
        PartyKind::Customer,
        PartyPayload {
            name: "Customer".to_string(),
            company_name: None,
            phone: None,
            email: None,
            address: None,
            tax_number: None,
            opening_balance_cents: 0,
            notes: None,
        },
    )
    .unwrap();
    let product = create_product(
        &conn,
        user.id,
        ProductPayload {
            sku: Some("GSP-SQ-20X20-0.5".to_string()),
            category_id: 4,
            name: "Galvanized Square Pipe 20x20 0.5mm".to_string(),
            product_type: "pipe".to_string(),
            material: "steel".to_string(),
            shape: "square".to_string(),
            finish: "galvanized".to_string(),
            size_label: "20x20".to_string(),
            width_mm: Some(20.0),
            height_mm: Some(20.0),
            diameter_mm: None,
            thickness_mm: Some(0.5),
            length_mm: None,
            unit: "piece".to_string(),
            description: None,
            cost_price_cents: 1000,
            selling_price_cents: 1500,
            wholesale_price_cents: Some(1400),
            minimum_quantity: 2.0,
            initial_quantity: Some(0.0),
        },
    )
    .unwrap();

    create_purchase_invoice(
        &conn,
        user.id,
        PurchaseInvoicePayload {
            supplier_id: supplier.id,
            invoice_number: Some("PI-TEST-1".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            shipping_cents: 0,
            paid_cents: 0,
            notes: None,
            items: vec![PurchaseItemPayload {
                product_id: product.id,
                quantity: 10.0,
                unit_cost_cents: 1000,
            }],
        },
    )
    .unwrap();
    create_sales_invoice(
        &conn,
        user.id,
        SalesInvoicePayload {
            customer_id: Some(customer.id),
            invoice_number: Some("SI-TEST-1".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            delivery_cents: 0,
            paid_cents: 3000,
            notes: None,
            items: vec![SalesItemPayload {
                product_id: product.id,
                quantity: 3.0,
                unit_price_cents: 1500,
            }],
        },
    )
    .unwrap();

    let stock: f64 = conn
        .query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get(0),
        )
        .unwrap();
    let profit: i64 = conn
        .query_row("SELECT profit_cents FROM sales_invoice_items LIMIT 1", [], |row| row.get(0))
        .unwrap();
    assert_eq!(stock, 7.0);
    assert_eq!(profit, 1500);
}

#[test]
fn demo_seed_populates_all_main_sections_once() {
    let mut conn = test_conn();
    let user = setup_admin(
        &conn,
        SetupAdminPayload {
            full_name: "Admin".to_string(),
            email: "admin@example.com".to_string(),
            password: "1234".to_string(),
        },
    )
    .unwrap();

    let first = seed_demo_data(&mut conn, user.id).unwrap();
    let second = seed_demo_data(&mut conn, user.id).unwrap();

    assert!(first.inserted);
    assert!(!second.inserted);

    for table in [
        "products",
        "suppliers",
        "customers",
        "purchase_invoices",
        "sales_invoices",
        "expenses",
        "payments",
        "inventory_transactions",
        "backups",
    ] {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        let count: i64 = conn.query_row(&sql, [], |row| row.get(0)).unwrap();
        assert!(count > 0, "{table} should have demo rows");
    }
}
