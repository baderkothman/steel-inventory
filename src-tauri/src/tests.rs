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
        supplier_id: None,
        location: None,
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
            supplier_id: None,
            location: None,
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

// ---------------------------------------------------------------------------
// Supplier-specific product variant + settlement feature tests
// ---------------------------------------------------------------------------

fn make_admin(conn: &Connection) -> i64 {
    setup_admin(
        conn,
        SetupAdminPayload {
            full_name: "Admin".to_string(),
            email: "admin@example.com".to_string(),
            password: "1234".to_string(),
        },
    )
    .unwrap()
    .id
}

fn make_supplier(conn: &Connection, user_id: i64, name: &str) -> i64 {
    create_party(
        conn,
        user_id,
        PartyKind::Supplier,
        PartyPayload {
            name: name.to_string(),
            company_name: None,
            phone: None,
            email: None,
            address: None,
            tax_number: None,
            opening_balance_cents: 0,
            notes: None,
        },
    )
    .unwrap()
    .id
}

fn round_pipe_payload(
    supplier_id: Option<i64>,
    name: &str,
    cost: i64,
    selling: i64,
) -> ProductPayload {
    ProductPayload {
        sku: None,
        category_id: 6,
        supplier_id,
        location: Some("Rack A".to_string()),
        name: name.to_string(),
        product_type: "pipe".to_string(),
        material: "steel".to_string(),
        shape: "round".to_string(),
        finish: "black".to_string(),
        size_label: "2 inch".to_string(),
        width_mm: None,
        height_mm: None,
        diameter_mm: Some(50.8),
        thickness_mm: Some(2.0),
        length_mm: None,
        unit: "piece".to_string(),
        description: None,
        cost_price_cents: cost,
        selling_price_cents: selling,
        wholesale_price_cents: Some(selling),
        minimum_quantity: 2.0,
        initial_quantity: Some(0.0),
    }
}

#[test]
fn same_spec_two_suppliers_share_spec_key_and_compare_by_price() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier_x = make_supplier(&conn, user, "Company X");
    let supplier_y = make_supplier(&conn, user, "Company Y");

    let prod_x = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier_x), "Round Pipe 2 inch 2mm", 1000, 1500),
    )
    .unwrap();
    let prod_y = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier_y), "Round Pipe 2 inch 2mm", 900, 1300),
    )
    .unwrap();

    // Same physical specification -> identical spec_key across suppliers.
    assert_eq!(prod_x.spec_key, prod_y.spec_key);
    assert_eq!(prod_x.supplier_name, "Company X");
    assert_eq!(prod_y.supplier_name, "Company Y");

    // Give both stock so the comparison shows availability.
    crate::services::inventory_service::adjust_stock(
        &conn,
        user,
        crate::models::StockAdjustmentPayload {
            product_id: prod_x.id,
            transaction_type: "adjustment_in".to_string(),
            quantity: 10.0,
            unit_cost_cents: Some(1000),
            notes: None,
        },
    )
    .unwrap();
    crate::services::inventory_service::adjust_stock(
        &conn,
        user,
        crate::models::StockAdjustmentPayload {
            product_id: prod_y.id,
            transaction_type: "adjustment_in".to_string(),
            quantity: 10.0,
            unit_cost_cents: Some(900),
            notes: None,
        },
    )
    .unwrap();

    let variants = crate::services::product_service::list_supplier_variants(
        &conn,
        crate::models::VariantFilters {
            search: Some("round pipe".to_string()),
            category_id: None,
            in_stock_only: Some(true),
        },
    )
    .unwrap();
    assert_eq!(variants.len(), 2);
    // Cheapest (Company Y at 1300) is sorted first within the shared spec.
    assert_eq!(variants[0].supplier_name, "Company Y");
    assert_eq!(variants[0].selling_price_cents, 1300);
}

#[test]
fn selling_one_supplier_variant_only_reduces_that_variant_stock() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier_x = make_supplier(&conn, user, "Company X");
    let supplier_y = make_supplier(&conn, user, "Company Y");
    let customer = create_party(
        &conn,
        user,
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

    let prod_x = create_product(&conn, user, round_pipe_payload(Some(supplier_x), "RP-X", 1000, 1500)).unwrap();
    let prod_y = create_product(&conn, user, round_pipe_payload(Some(supplier_y), "RP-Y", 900, 1300)).unwrap();

    for (id, cost) in [(prod_x.id, 1000), (prod_y.id, 900)] {
        create_purchase_invoice(
            &conn,
            user,
            PurchaseInvoicePayload {
                supplier_id: if id == prod_x.id { supplier_x } else { supplier_y },
                invoice_number: Some(format!("PI-{id}")),
                invoice_date: today_date(),
                discount_cents: 0,
                tax_cents: 0,
                shipping_cents: 0,
                paid_cents: 0,
                notes: None,
                items: vec![PurchaseItemPayload { product_id: id, quantity: 10.0, unit_cost_cents: cost }],
            },
        )
        .unwrap();
    }

    // Sell only the cheaper (Company Y) variant.
    create_sales_invoice(
        &conn,
        user,
        SalesInvoicePayload {
            customer_id: Some(customer.id),
            invoice_number: Some("SI-Y".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            delivery_cents: 0,
            paid_cents: 0,
            notes: None,
            items: vec![SalesItemPayload { product_id: prod_y.id, quantity: 4.0, unit_price_cents: 1300 }],
        },
    )
    .unwrap();

    let stock_x: f64 = conn
        .query_row("SELECT current_quantity FROM stock_levels WHERE product_id = ?1", [prod_x.id], |r| r.get(0))
        .unwrap();
    let stock_y: f64 = conn
        .query_row("SELECT current_quantity FROM stock_levels WHERE product_id = ?1", [prod_y.id], |r| r.get(0))
        .unwrap();
    assert_eq!(stock_x, 10.0, "Company X stock must be untouched");
    assert_eq!(stock_y, 6.0, "Company Y stock must drop by 4");
}

#[test]
fn settlement_report_owes_correct_supplier_and_excludes_cancelled() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier_x = make_supplier(&conn, user, "Company X");
    let supplier_y = make_supplier(&conn, user, "Company Y");
    let customer = create_party(
        &conn,
        user,
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

    let prod_x = create_product(&conn, user, round_pipe_payload(Some(supplier_x), "RP-X", 1000, 1500)).unwrap();
    let prod_y = create_product(&conn, user, round_pipe_payload(Some(supplier_y), "RP-Y", 900, 1300)).unwrap();
    for (id, sid, cost) in [(prod_x.id, supplier_x, 1000), (prod_y.id, supplier_y, 900)] {
        create_purchase_invoice(
            &conn,
            user,
            PurchaseInvoicePayload {
                supplier_id: sid,
                invoice_number: Some(format!("PI-{id}")),
                invoice_date: today_date(),
                discount_cents: 0,
                tax_cents: 0,
                shipping_cents: 0,
                paid_cents: 0,
                notes: None,
                items: vec![PurchaseItemPayload { product_id: id, quantity: 10.0, unit_cost_cents: cost }],
            },
        )
        .unwrap();
    }

    // Completed sale of Company Y goods: 5 @ cost 900 = 4500 owed to Y.
    create_sales_invoice(
        &conn,
        user,
        SalesInvoicePayload {
            customer_id: Some(customer.id),
            invoice_number: Some("SI-Y".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            delivery_cents: 0,
            paid_cents: 0,
            notes: None,
            items: vec![SalesItemPayload { product_id: prod_y.id, quantity: 5.0, unit_price_cents: 1300 }],
        },
    )
    .unwrap();

    // A cancelled sale of Company X goods must NOT add to X's payable.
    let cancelled = create_sales_invoice(
        &conn,
        user,
        SalesInvoicePayload {
            customer_id: Some(customer.id),
            invoice_number: Some("SI-X".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            delivery_cents: 0,
            paid_cents: 0,
            notes: None,
            items: vec![SalesItemPayload { product_id: prod_x.id, quantity: 3.0, unit_price_cents: 1500 }],
        },
    )
    .unwrap();
    crate::services::sales_service::cancel_sales_invoice(&conn, user, cancelled.id).unwrap();

    let report = crate::services::report_service::supplier_settlement_report(
        &conn,
        crate::models::ReportFilters::default(),
    )
    .unwrap();
    // Only Company Y should appear, owed 4500.
    assert_eq!(report.len(), 1);
    assert_eq!(report[0]["supplier"], "Company Y");
    assert_eq!(report[0]["quantity_sold"], 5.0);
    assert_eq!(report[0]["owed_cents"], 4500);

    // Record a partial settlement and confirm the summary remaining balance.
    crate::services::settlement_service::create_settlement_payment(
        &conn,
        user,
        crate::models::SettlementPaymentPayload {
            supplier_id: supplier_y,
            period_start: today_date(),
            period_end: today_date(),
            amount_cents: 2000,
            status: "partial".to_string(),
            payment_date: today_date(),
            reference: Some("REF-1".to_string()),
            notes: None,
        },
    )
    .unwrap();
    let summary = crate::services::report_service::supplier_settlement_summary(
        &conn,
        crate::models::ReportFilters::default(),
    )
    .unwrap();
    let y = summary.iter().find(|r| r["supplier"] == "Company Y").unwrap();
    assert_eq!(y["owed_cents"], 4500);
    assert_eq!(y["settled_cents"], 2000);
    assert_eq!(y["remaining_cents"], 2500);
}

#[test]
fn seed_creates_two_supplier_variants_sharing_one_spec_key() {
    let mut conn = test_conn();
    let user = make_admin(&conn);
    seed_demo_data(&mut conn, user).unwrap();

    // The two black round-pipe variants (different suppliers) must share a spec_key,
    // proving the SQL spec_key backfill matches the round-pipe specification.
    let distinct_keys: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT spec_key) FROM products WHERE sku LIKE 'DEMO-BSP-RD-30-1.5%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let variant_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM products WHERE sku LIKE 'DEMO-BSP-RD-30-1.5%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(variant_count, 2);
    assert_eq!(distinct_keys, 1);
}

#[test]
fn missing_supplier_falls_back_to_unknown_supplier() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let product = create_product(&conn, user, round_pipe_payload(None, "RP-Unknown", 1000, 1500)).unwrap();
    assert_eq!(product.supplier_name, "Unknown Supplier");
    assert!(product.supplier_id.is_some());
}
