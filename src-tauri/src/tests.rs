use rusqlite::Connection;

use crate::{
    db::migrations::run_migrations,
    models::{
        ClearAllDataPayload, DateRangeFilters, ExpensePayload, InstallmentPaymentPayload,
        MovementFilters, PartyPayload, PaymentFilters, PaymentPayload, ProductPayload,
        PurchaseInvoicePayload, PurchaseItemPayload, PurchaseReturnItemPayload,
        PurchaseReturnPayload, PurchaseReturnUpdatePayload, ReportFilters, SalesInvoicePayload,
        SalesItemPayload, SetupAdminPayload,
    },
    services::{
        auth_service::setup_admin,
        data_lifecycle_service::clear_all_data,
        expense_service::{
            create_expense, delete_expense, list_expense_payments, list_expenses,
            permanently_delete_expense, record_expense_payment, restore_expense,
        },
        inventory_service::{
            adjust_stock, cancel_stock_adjustment, list_product_movement,
            permanently_delete_stock_adjustment, restore_stock_adjustment,
        },
        invoice_payment_service::{list_invoice_payments, record_invoice_payment},
        party_service::{archive_party, create_party, party_balance, statement, PartyKind},
        payment_service::{
            create_payment, delete_payment, list_payments, permanently_delete_payment,
            restore_payment,
        },
        product_service::{archive_product, create_product, delete_product, latest_price},
        purchase_return_service::{
            cancel_purchase_return, create_purchase_return, get_purchase_return_context,
            restore_purchase_return, update_purchase_return,
        },
        purchase_service::{
            cancel_purchase_invoice, create_purchase_invoice, list_purchase_invoices,
            permanently_delete_purchase_invoice, restore_purchase_invoice,
        },
        report_service::{
            daily_sales_report, dashboard_summary, expense_report, payment_report, profit_report,
            purchase_report, stock_count_report, stock_movement_report, stock_report,
        },
        sales_service::{
            cancel_sales_invoice, create_sales_invoice, list_sales_invoices,
            permanently_delete_sales_invoice, restore_sales_invoice,
        },
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
        .query_row("SELECT COUNT(*) FROM expense_categories", [], |row| {
            row.get(0)
        })
        .unwrap();
    let settings: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM company_settings WHERE id = 1",
            [],
            |row| row.get(0),
        )
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
        .query_row(
            "SELECT profit_cents FROM sales_invoice_items LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stock, 7.0);
    assert_eq!(profit, 1500);
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

    let prod_x = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier_x), "RP-X", 1000, 1500),
    )
    .unwrap();
    let prod_y = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier_y), "RP-Y", 900, 1300),
    )
    .unwrap();

    for (id, cost) in [(prod_x.id, 1000), (prod_y.id, 900)] {
        create_purchase_invoice(
            &conn,
            user,
            PurchaseInvoicePayload {
                supplier_id: if id == prod_x.id {
                    supplier_x
                } else {
                    supplier_y
                },
                invoice_number: Some(format!("PI-{id}")),
                invoice_date: today_date(),
                discount_cents: 0,
                tax_cents: 0,
                shipping_cents: 0,
                paid_cents: 0,
                notes: None,
                items: vec![PurchaseItemPayload {
                    product_id: id,
                    quantity: 10.0,
                    unit_cost_cents: cost,
                }],
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
            items: vec![SalesItemPayload {
                product_id: prod_y.id,
                quantity: 4.0,
                unit_price_cents: 1300,
            }],
        },
    )
    .unwrap();

    let stock_x: f64 = conn
        .query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [prod_x.id],
            |r| r.get(0),
        )
        .unwrap();
    let stock_y: f64 = conn
        .query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [prod_y.id],
            |r| r.get(0),
        )
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

    let prod_x = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier_x), "RP-X", 1000, 1500),
    )
    .unwrap();
    let prod_y = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier_y), "RP-Y", 900, 1300),
    )
    .unwrap();
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
                items: vec![PurchaseItemPayload {
                    product_id: id,
                    quantity: 10.0,
                    unit_cost_cents: cost,
                }],
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
            items: vec![SalesItemPayload {
                product_id: prod_y.id,
                quantity: 5.0,
                unit_price_cents: 1300,
            }],
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
            items: vec![SalesItemPayload {
                product_id: prod_x.id,
                quantity: 3.0,
                unit_price_cents: 1500,
            }],
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
    let settlement = crate::services::settlement_service::create_settlement_payment(
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
    let y = summary
        .iter()
        .find(|r| r["supplier"] == "Company Y")
        .unwrap();
    assert_eq!(y["owed_cents"], 4500);
    assert_eq!(y["settled_cents"], 2000);
    assert_eq!(y["remaining_cents"], 2500);

    crate::services::settlement_service::delete_settlement_payment(&conn, user, settlement.id)
        .unwrap();
    let summary_after_cancel = crate::services::report_service::supplier_settlement_summary(
        &conn,
        crate::models::ReportFilters::default(),
    )
    .unwrap();
    let y_after_cancel = summary_after_cancel
        .iter()
        .find(|row| row["supplier"] == "Company Y")
        .unwrap();
    assert_eq!(y_after_cancel["settled_cents"], 0);
    assert_eq!(y_after_cancel["remaining_cents"], 4500);
}

#[test]
fn missing_supplier_falls_back_to_unknown_supplier() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let product = create_product(
        &conn,
        user,
        round_pipe_payload(None, "RP-Unknown", 1000, 1500),
    )
    .unwrap();
    assert_eq!(product.supplier_name, "Unknown Supplier");
    assert!(product.supplier_id.is_some());
}

#[test]
fn autogenerated_skus_remain_unique_after_products_are_archived() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let payload = round_pipe_payload(None, "Round Pipe", 1000, 1500);

    let first = create_product(&conn, user, payload.clone()).unwrap();
    let second = create_product(&conn, user, payload.clone()).unwrap();
    assert_eq!(second.sku, format!("{}-2", first.sku));

    archive_product(&conn, user, first.id).unwrap();
    archive_product(&conn, user, second.id).unwrap();

    let replacement = create_product(&conn, user, payload).unwrap();
    assert_eq!(replacement.sku, format!("{}-3", first.sku));
}

#[test]
fn manually_entered_duplicate_sku_is_still_rejected() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let mut payload = round_pipe_payload(None, "Round Pipe", 1000, 1500);
    payload.sku = Some("CUSTOM-SKU".to_string());

    create_product(&conn, user, payload.clone()).unwrap();
    let error = create_product(&conn, user, payload).unwrap_err();
    assert_eq!(error.code, "DUPLICATE_SKU");
}

#[test]
fn retired_demo_data_migration_removes_the_seeded_product_graph() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let mut payload = round_pipe_payload(None, "Demo Round Pipe", 1000, 1500);
    payload.sku = Some("DEMO-ROUND-PIPE".to_string());
    let product = create_product(&conn, user, payload).unwrap();
    conn.execute(
        "INSERT INTO backups
         (backup_path, backup_type, status, notes, created_at)
         VALUES ('steel_inventory_backup_demo.db', 'manual', 'success',
                 'Demo backup log row only; no file is created by demo seed.',
                 datetime('now'))",
        [],
    )
    .unwrap();

    conn.execute_batch(include_str!("db/migrations/008_remove_demo_data.sql"))
        .unwrap();

    for table in ["products", "product_prices", "stock_levels", "backups"] {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0, "{table} should not retain demo records");
    }
    let audit_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_logs
             WHERE table_name = 'products' AND record_id = ?1",
            [product.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(audit_count, 0);
}

#[test]
fn unused_product_can_be_permanently_deleted_with_its_stock_records() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier = make_supplier(&conn, user, "Delete Test Supplier");
    let product = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier), "Unused Product", 1000, 1500),
    )
    .unwrap();

    delete_product(&conn, user, product.id).unwrap();

    let product_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM products WHERE id = ?1",
            [product.id],
            |row| row.get(0),
        )
        .unwrap();
    let stock_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(product_count, 0);
    assert_eq!(stock_count, 0);
}

#[test]
fn invoiced_product_must_be_archived_instead_of_deleted() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier = make_supplier(&conn, user, "History Supplier");
    let product = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier), "Invoiced Product", 1000, 1500),
    )
    .unwrap();
    create_purchase_invoice(
        &conn,
        user,
        PurchaseInvoicePayload {
            supplier_id: supplier,
            invoice_number: Some("PI-DELETE-GUARD".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            shipping_cents: 0,
            paid_cents: 0,
            notes: None,
            items: vec![PurchaseItemPayload {
                product_id: product.id,
                quantity: 1.0,
                unit_cost_cents: 1000,
            }],
        },
    )
    .unwrap();

    let error = delete_product(&conn, user, product.id).unwrap_err();
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(error.message.contains("Archive it instead"));
}

#[test]
fn stock_remaining_report_uses_product_details_without_sku() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier = make_supplier(&conn, user, "Stock Report Supplier");
    let product = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier), "Round Pipe 2 inch", 1000, 1750),
    )
    .unwrap();

    crate::services::inventory_service::adjust_stock(
        &conn,
        user,
        crate::models::StockAdjustmentPayload {
            product_id: product.id,
            transaction_type: "adjustment_in".to_string(),
            quantity: 7.5,
            unit_cost_cents: Some(1000),
            notes: None,
        },
    )
    .unwrap();

    let rows = stock_report(&conn, ReportFilters::default()).unwrap();
    let row = rows
        .iter()
        .find(|row| row["product_name"] == "Round Pipe 2 inch")
        .unwrap();

    assert!(row.get("sku").is_none());
    assert_eq!(row["product_type"], "pipe");
    assert_eq!(row["size"], "2 inch");
    assert_eq!(row["thickness_mm"], 2.0);
    assert_eq!(row["selling_price_cents"], 1750);
    assert_eq!(row["remaining_quantity"], 7.5);
}

#[test]
fn deleting_payment_recomputes_the_live_party_balance() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let customer = create_party(
        &conn,
        user,
        PartyKind::Customer,
        PartyPayload {
            name: "Balance Customer".to_string(),
            company_name: None,
            phone: None,
            email: None,
            address: None,
            tax_number: None,
            opening_balance_cents: 10_000,
            notes: None,
        },
    )
    .unwrap();

    let payment = create_payment(
        &conn,
        user,
        PaymentPayload {
            party_type: "customer".to_string(),
            party_id: customer.id,
            amount_cents: 3_000,
            currency: String::new(),
            payment_method: "cash".to_string(),
            payment_date: today_date(),
            reference_type: None,
            reference_id: None,
            notes: None,
        },
    )
    .unwrap();
    assert_eq!(
        party_balance(&conn, PartyKind::Customer, customer.id).unwrap(),
        7_000
    );

    delete_payment(&conn, user, payment.id).unwrap();
    assert_eq!(
        party_balance(&conn, PartyKind::Customer, customer.id).unwrap(),
        10_000
    );
}

#[test]
fn archived_balances_remain_in_debt_totals_and_credits_do_not_offset_debt() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let debtor = create_party(
        &conn,
        user,
        PartyKind::Customer,
        PartyPayload {
            name: "Archived Debtor".to_string(),
            company_name: None,
            phone: None,
            email: None,
            address: None,
            tax_number: None,
            opening_balance_cents: 8_000,
            notes: None,
        },
    )
    .unwrap();
    let credit_customer = create_party(
        &conn,
        user,
        PartyKind::Customer,
        PartyPayload {
            name: "Credit Customer".to_string(),
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
    create_payment(
        &conn,
        user,
        PaymentPayload {
            party_type: "customer".to_string(),
            party_id: credit_customer.id,
            amount_cents: 2_000,
            currency: String::new(),
            payment_method: "cash".to_string(),
            payment_date: today_date(),
            reference_type: None,
            reference_id: None,
            notes: None,
        },
    )
    .unwrap();
    archive_party(&conn, user, PartyKind::Customer, debtor.id).unwrap();

    let summary = dashboard_summary(&conn, Some(today_date())).unwrap();
    assert_eq!(summary.total_customer_debts_cents, 8_000);
}

#[test]
fn database_rejects_orphan_payment_parties() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let error = conn
        .execute(
            "INSERT INTO payments
             (party_type, party_id, payment_direction, amount_cents, currency, payment_method,
              payment_date, reference_type, reference_id, notes, created_by, created_at)
             VALUES ('customer', 999999, 'in', 1000, 'USD', 'cash', ?1, NULL, NULL, NULL, ?2, ?1)",
            rusqlite::params![today_date(), user],
        )
        .unwrap_err();
    assert!(error.to_string().contains("payment customer not found"));
}

#[test]
fn linked_payment_delete_restores_invoice_and_rejects_the_wrong_customer() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let customer = create_party(
        &conn,
        user,
        PartyKind::Customer,
        PartyPayload {
            name: "Invoice Customer".to_string(),
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
    let other_customer = create_party(
        &conn,
        user,
        PartyKind::Customer,
        PartyPayload {
            name: "Other Customer".to_string(),
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
    let date = today_date();
    conn.execute(
        "INSERT INTO sales_invoices
         (customer_id, invoice_number, invoice_date, subtotal_cents, discount_cents, tax_cents,
          delivery_cents, total_cents, paid_cents, remaining_cents, payment_status, sales_status,
          notes, created_by, created_at, updated_at)
         VALUES (?1, 'SI-BALANCE-TEST', ?2, 10000, 0, 0, 0, 10000, 0, 10000,
                 'unpaid', 'completed', NULL, ?3, ?2, ?2)",
        rusqlite::params![customer.id, date, user],
    )
    .unwrap();
    let invoice_id = conn.last_insert_rowid();

    let wrong_customer_error = create_payment(
        &conn,
        user,
        PaymentPayload {
            party_type: "customer".to_string(),
            party_id: other_customer.id,
            amount_cents: 1_000,
            currency: String::new(),
            payment_method: "cash".to_string(),
            payment_date: today_date(),
            reference_type: Some("sales_invoice".to_string()),
            reference_id: Some(invoice_id),
            notes: None,
        },
    )
    .unwrap_err();
    assert_eq!(wrong_customer_error.code, "VALIDATION_ERROR");

    let payment = create_payment(
        &conn,
        user,
        PaymentPayload {
            party_type: "customer".to_string(),
            party_id: customer.id,
            amount_cents: 4_000,
            currency: String::new(),
            payment_method: "cash".to_string(),
            payment_date: today_date(),
            reference_type: Some("sales_invoice".to_string()),
            reference_id: Some(invoice_id),
            notes: None,
        },
    )
    .unwrap();
    assert_eq!(
        party_balance(&conn, PartyKind::Customer, customer.id).unwrap(),
        6_000
    );
    let remaining: i64 = conn
        .query_row(
            "SELECT remaining_cents FROM sales_invoices WHERE id = ?1",
            [invoice_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(remaining, 6_000);

    delete_payment(&conn, user, payment.id).unwrap();
    assert_eq!(
        party_balance(&conn, PartyKind::Customer, customer.id).unwrap(),
        10_000
    );
    let remaining: i64 = conn
        .query_row(
            "SELECT remaining_cents FROM sales_invoices WHERE id = ?1",
            [invoice_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(remaining, 10_000);
}

#[test]
fn cancelled_invoices_leave_history_but_no_stock_financial_dashboard_or_report_effect() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier = make_supplier(&conn, user, "Lifecycle Supplier");
    let customer = create_party(
        &conn,
        user,
        PartyKind::Customer,
        PartyPayload {
            name: "Lifecycle Customer".to_string(),
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
        user,
        round_pipe_payload(Some(supplier), "Lifecycle Product", 800, 1_500),
    )
    .unwrap();

    let first_purchase = create_purchase_invoice(
        &conn,
        user,
        PurchaseInvoicePayload {
            supplier_id: supplier,
            invoice_number: Some("PI-LIFECYCLE-1".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            shipping_cents: 0,
            paid_cents: 2_000,
            notes: None,
            items: vec![PurchaseItemPayload {
                product_id: product.id,
                quantity: 10.0,
                unit_cost_cents: 1_000,
            }],
        },
    )
    .unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        10.0
    );
    assert_eq!(latest_price(&conn, product.id).unwrap().0, 1_000);

    cancel_purchase_invoice(&conn, user, first_purchase.id).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        0.0
    );
    assert_eq!(latest_price(&conn, product.id).unwrap().0, 800);
    assert_eq!(
        party_balance(&conn, PartyKind::Supplier, supplier).unwrap(),
        0
    );

    let second_purchase = create_purchase_invoice(
        &conn,
        user,
        PurchaseInvoicePayload {
            supplier_id: supplier,
            invoice_number: Some("PI-LIFECYCLE-2".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            shipping_cents: 0,
            paid_cents: 2_000,
            notes: None,
            items: vec![PurchaseItemPayload {
                product_id: product.id,
                quantity: 10.0,
                unit_cost_cents: 1_100,
            }],
        },
    )
    .unwrap();
    let sale = create_sales_invoice(
        &conn,
        user,
        SalesInvoicePayload {
            customer_id: Some(customer.id),
            invoice_number: Some("SI-LIFECYCLE-1".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            delivery_cents: 0,
            paid_cents: 2_000,
            notes: None,
            items: vec![SalesItemPayload {
                product_id: product.id,
                quantity: 4.0,
                unit_price_cents: 1_500,
            }],
        },
    )
    .unwrap();
    let active_summary = dashboard_summary(&conn, Some(today_date())).unwrap();
    assert_eq!(active_summary.today_sales_count, 1);
    assert_eq!(active_summary.today_purchase_count, 1);
    assert_eq!(active_summary.today_sales_cents, 6_000);
    assert_eq!(
        party_balance(&conn, PartyKind::Customer, customer.id).unwrap(),
        4_000
    );

    cancel_sales_invoice(&conn, user, sale.id).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        10.0
    );
    assert_eq!(
        party_balance(&conn, PartyKind::Customer, customer.id).unwrap(),
        0
    );
    cancel_purchase_invoice(&conn, user, second_purchase.id).unwrap();

    let summary = dashboard_summary(&conn, Some(today_date())).unwrap();
    assert_eq!(summary.today_sales_count, 0);
    assert_eq!(summary.today_purchase_count, 0);
    assert_eq!(summary.today_sales_cents, 0);
    assert_eq!(summary.today_paid_cents, 0);
    assert_eq!(summary.today_remaining_cents, 0);
    assert_eq!(summary.today_profit_cents, 0);
    assert_eq!(summary.current_stock_value_cents, 0);
    assert_eq!(summary.total_customer_debts_cents, 0);
    assert_eq!(summary.total_supplier_debts_cents, 0);
    assert!(summary.recent_sales_invoices.is_empty());
    assert!(summary.recent_purchase_invoices.is_empty());
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        0.0
    );

    assert!(daily_sales_report(&conn, ReportFilters::default())
        .unwrap()
        .is_empty());
    assert!(profit_report(&conn, ReportFilters::default())
        .unwrap()
        .is_empty());
    assert!(purchase_report(&conn, ReportFilters::default())
        .unwrap()
        .is_empty());
    assert!(stock_movement_report(&conn, ReportFilters::default())
        .unwrap()
        .is_empty());
    assert!(payment_report(&conn, ReportFilters::default())
        .unwrap()
        .is_empty());

    let sales_history = list_sales_invoices(&conn, Default::default()).unwrap();
    let purchase_history = list_purchase_invoices(&conn, Default::default()).unwrap();
    let payment_history = list_payments(&conn, PaymentFilters::default()).unwrap();
    let movement_history =
        list_product_movement(&conn, product.id, MovementFilters::default()).unwrap();
    assert_eq!(sales_history.len(), 1);
    assert!(sales_history.iter().all(|row| row.status == "cancelled"));
    assert_eq!(purchase_history.len(), 2);
    assert!(purchase_history.iter().all(|row| row.status == "cancelled"));
    assert_eq!(payment_history.len(), 3);
    assert!(payment_history.iter().all(|row| row.status == "cancelled"));
    assert_eq!(movement_history.len(), 3);
    assert!(movement_history.iter().all(|row| row.status == "cancelled"));
}

#[test]
fn cancelled_payments_expenses_and_manual_movements_are_history_only() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let customer = create_party(
        &conn,
        user,
        PartyKind::Customer,
        PartyPayload {
            name: "Lifecycle Cash Customer".to_string(),
            company_name: None,
            phone: None,
            email: None,
            address: None,
            tax_number: None,
            opening_balance_cents: 5_000,
            notes: None,
        },
    )
    .unwrap();
    let payment = create_payment(
        &conn,
        user,
        PaymentPayload {
            party_type: "customer".to_string(),
            party_id: customer.id,
            amount_cents: 2_000,
            currency: "USD".to_string(),
            payment_method: "cash".to_string(),
            payment_date: today_date(),
            reference_type: None,
            reference_id: None,
            notes: None,
        },
    )
    .unwrap();
    delete_payment(&conn, user, payment.id).unwrap();
    assert_eq!(
        party_balance(&conn, PartyKind::Customer, customer.id).unwrap(),
        5_000
    );
    assert!(payment_report(&conn, ReportFilters::default())
        .unwrap()
        .is_empty());

    let expense_category: i64 = conn
        .query_row(
            "SELECT id FROM expense_categories ORDER BY id LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let expense = create_expense(
        &conn,
        user,
        ExpensePayload {
            expense_category_id: expense_category,
            title: "Cancelled test expense".to_string(),
            amount_cents: 1_500,
            paid_cents: 1_500,
            currency: "USD".to_string(),
            expense_date: today_date(),
            payment_method: "cash".to_string(),
            notes: None,
        },
    )
    .unwrap();
    assert_eq!(
        dashboard_summary(&conn, Some(today_date()))
            .unwrap()
            .today_expenses_cents,
        1_500
    );
    delete_expense(&conn, user, expense.id).unwrap();
    assert_eq!(
        dashboard_summary(&conn, Some(today_date()))
            .unwrap()
            .today_expenses_cents,
        0
    );
    assert!(expense_report(&conn, ReportFilters::default())
        .unwrap()
        .is_empty());
    let expense_history = list_expenses(&conn, Default::default()).unwrap();
    assert_eq!(expense_history[0].status, "cancelled");

    let supplier = make_supplier(&conn, user, "Adjustment Supplier");
    let product = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier), "Adjustment Product", 1_000, 1_500),
    )
    .unwrap();
    adjust_stock(
        &conn,
        user,
        crate::models::StockAdjustmentPayload {
            product_id: product.id,
            transaction_type: "adjustment_in".to_string(),
            quantity: 7.0,
            unit_cost_cents: Some(1_000),
            notes: None,
        },
    )
    .unwrap();
    let movement_id: i64 = conn
        .query_row(
            "SELECT id FROM inventory_transactions WHERE product_id = ?1 ORDER BY id DESC LIMIT 1",
            [product.id],
            |row| row.get(0),
        )
        .unwrap();
    cancel_stock_adjustment(&conn, user, movement_id).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        0.0
    );
    assert!(stock_movement_report(&conn, ReportFilters::default())
        .unwrap()
        .is_empty());
}

#[test]
fn recently_deleted_records_restore_original_effects_and_can_be_permanently_removed() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier = make_supplier(&conn, user, "Restore Supplier");
    let customer = create_party(
        &conn,
        user,
        PartyKind::Customer,
        PartyPayload {
            name: "Restore Customer".to_string(),
            company_name: None,
            phone: None,
            email: None,
            address: None,
            tax_number: None,
            opening_balance_cents: 5_000,
            notes: None,
        },
    )
    .unwrap();
    let product = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier), "Restore Product", 800, 1_500),
    )
    .unwrap();

    let purchase = create_purchase_invoice(
        &conn,
        user,
        PurchaseInvoicePayload {
            supplier_id: supplier,
            invoice_number: Some("PI-RESTORE".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            shipping_cents: 0,
            paid_cents: 2_000,
            notes: None,
            items: vec![PurchaseItemPayload {
                product_id: product.id,
                quantity: 10.0,
                unit_cost_cents: 1_000,
            }],
        },
    )
    .unwrap();
    cancel_purchase_invoice(&conn, user, purchase.id).unwrap();
    let deleted_at: Option<String> = conn
        .query_row(
            "SELECT deleted_at FROM purchase_invoices WHERE id = ?1",
            [purchase.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(deleted_at.is_some());

    restore_purchase_invoice(&conn, user, purchase.id).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        10.0
    );
    assert_eq!(latest_price(&conn, product.id).unwrap().0, 1_000);
    assert_eq!(
        party_balance(&conn, PartyKind::Supplier, supplier).unwrap(),
        8_000
    );
    let restored_purchase = list_purchase_invoices(&conn, Default::default()).unwrap();
    assert_eq!(restored_purchase[0].status, "active");
    assert!(restored_purchase[0].deleted_at.is_none());

    let sale = create_sales_invoice(
        &conn,
        user,
        SalesInvoicePayload {
            customer_id: Some(customer.id),
            invoice_number: Some("SI-RESTORE".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            delivery_cents: 0,
            paid_cents: 1_000,
            notes: None,
            items: vec![SalesItemPayload {
                product_id: product.id,
                quantity: 4.0,
                unit_price_cents: 1_500,
            }],
        },
    )
    .unwrap();
    cancel_sales_invoice(&conn, user, sale.id).unwrap();
    restore_sales_invoice(&conn, user, sale.id).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        6.0
    );
    assert_eq!(
        party_balance(&conn, PartyKind::Customer, customer.id).unwrap(),
        10_000
    );

    let payment = create_payment(
        &conn,
        user,
        PaymentPayload {
            party_type: "customer".to_string(),
            party_id: customer.id,
            amount_cents: 2_000,
            currency: "USD".to_string(),
            payment_method: "cash".to_string(),
            payment_date: today_date(),
            reference_type: None,
            reference_id: None,
            notes: None,
        },
    )
    .unwrap();
    delete_payment(&conn, user, payment.id).unwrap();
    restore_payment(&conn, user, payment.id).unwrap();
    assert_eq!(
        party_balance(&conn, PartyKind::Customer, customer.id).unwrap(),
        8_000
    );
    delete_payment(&conn, user, payment.id).unwrap();
    permanently_delete_payment(&conn, user, payment.id).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM payments WHERE id = ?1",
            [payment.id],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );

    let expense_category: i64 = conn
        .query_row(
            "SELECT id FROM expense_categories ORDER BY id LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let expense = create_expense(
        &conn,
        user,
        ExpensePayload {
            expense_category_id: expense_category,
            title: "Restorable expense".to_string(),
            amount_cents: 1_250,
            paid_cents: 1_250,
            currency: "USD".to_string(),
            expense_date: today_date(),
            payment_method: "cash".to_string(),
            notes: None,
        },
    )
    .unwrap();
    delete_expense(&conn, user, expense.id).unwrap();
    restore_expense(&conn, user, expense.id).unwrap();
    assert_eq!(
        expense_report(&conn, ReportFilters::default())
            .unwrap()
            .len(),
        1
    );
    delete_expense(&conn, user, expense.id).unwrap();
    permanently_delete_expense(&conn, user, expense.id).unwrap();

    adjust_stock(
        &conn,
        user,
        crate::models::StockAdjustmentPayload {
            product_id: product.id,
            transaction_type: "adjustment_in".to_string(),
            quantity: 3.0,
            unit_cost_cents: Some(1_000),
            notes: None,
        },
    )
    .unwrap();
    let movement_id: i64 = conn
        .query_row(
            "SELECT id FROM inventory_transactions
             WHERE product_id = ?1 AND reference_type = 'manual'
             ORDER BY id DESC LIMIT 1",
            [product.id],
            |row| row.get(0),
        )
        .unwrap();
    cancel_stock_adjustment(&conn, user, movement_id).unwrap();
    restore_stock_adjustment(&conn, user, movement_id).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        9.0
    );
    cancel_stock_adjustment(&conn, user, movement_id).unwrap();
    permanently_delete_stock_adjustment(&conn, user, movement_id).unwrap();

    cancel_sales_invoice(&conn, user, sale.id).unwrap();
    permanently_delete_sales_invoice(&conn, user, sale.id).unwrap();
    cancel_purchase_invoice(&conn, user, purchase.id).unwrap();
    permanently_delete_purchase_invoice(&conn, user, purchase.id).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM sales_invoices WHERE id = ?1",
            [sale.id],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM purchase_invoices WHERE id = ?1",
            [purchase.id],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
}

#[test]
fn restoring_a_sale_rechecks_current_stock() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier = make_supplier(&conn, user, "Restore Guard Supplier");
    let product = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier), "Restore Guard Product", 1_000, 1_500),
    )
    .unwrap();
    adjust_stock(
        &conn,
        user,
        crate::models::StockAdjustmentPayload {
            product_id: product.id,
            transaction_type: "adjustment_in".to_string(),
            quantity: 5.0,
            unit_cost_cents: Some(1_000),
            notes: None,
        },
    )
    .unwrap();
    let first = create_sales_invoice(
        &conn,
        user,
        SalesInvoicePayload {
            customer_id: None,
            invoice_number: Some("SI-RESTORE-GUARD-1".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            delivery_cents: 0,
            paid_cents: 0,
            notes: None,
            items: vec![SalesItemPayload {
                product_id: product.id,
                quantity: 4.0,
                unit_price_cents: 1_500,
            }],
        },
    )
    .unwrap();
    cancel_sales_invoice(&conn, user, first.id).unwrap();
    create_sales_invoice(
        &conn,
        user,
        SalesInvoicePayload {
            customer_id: None,
            invoice_number: Some("SI-RESTORE-GUARD-2".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            delivery_cents: 0,
            paid_cents: 0,
            notes: None,
            items: vec![SalesItemPayload {
                product_id: product.id,
                quantity: 5.0,
                unit_price_cents: 1_500,
            }],
        },
    )
    .unwrap();
    let error = restore_sales_invoice(&conn, user, first.id).unwrap_err();
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(error.message.contains("enough stock"));
}

#[test]
fn expense_installments_preserve_each_payment_and_open_balance() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let category_id: i64 = conn
        .query_row(
            "SELECT id FROM expense_categories ORDER BY id LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let expense = create_expense(
        &conn,
        user,
        ExpensePayload {
            expense_category_id: category_id,
            title: "Equipment repair".to_string(),
            amount_cents: 10_000,
            paid_cents: 3_000,
            currency: "USD".to_string(),
            expense_date: today_date(),
            payment_method: "cash".to_string(),
            notes: None,
        },
    )
    .unwrap();
    assert_eq!(expense.payment_status, "partial");
    assert_eq!(expense.remaining_cents, 7_000);

    record_expense_payment(
        &conn,
        user,
        expense.id,
        InstallmentPaymentPayload {
            amount_cents: 2_500,
            payment_method: "bank".to_string(),
            payment_date: today_date(),
            notes: Some("Second installment".to_string()),
        },
    )
    .unwrap();
    let refreshed = list_expenses(
        &conn,
        crate::models::ExpenseFilters {
            active_only: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(refreshed[0].paid_cents, 5_500);
    assert_eq!(refreshed[0].remaining_cents, 4_500);
    assert_eq!(list_expense_payments(&conn, expense.id).unwrap().len(), 2);

    delete_expense(&conn, user, expense.id).unwrap();
    assert_eq!(list_expense_payments(&conn, expense.id).unwrap().len(), 0);
    restore_expense(&conn, user, expense.id).unwrap();
    assert_eq!(list_expense_payments(&conn, expense.id).unwrap().len(), 2);
}

#[test]
fn walk_in_invoice_installments_are_recorded_and_restored() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let product = create_product(
        &conn,
        user,
        round_pipe_payload(None, "WALK-IN-PAY", 500, 1_000),
    )
    .unwrap();
    adjust_stock(
        &conn,
        user,
        crate::models::StockAdjustmentPayload {
            product_id: product.id,
            transaction_type: "adjustment_in".to_string(),
            quantity: 5.0,
            unit_cost_cents: Some(500),
            notes: None,
        },
    )
    .unwrap();
    let invoice = create_sales_invoice(
        &conn,
        user,
        SalesInvoicePayload {
            customer_id: None,
            invoice_number: Some("WALK-IN-INSTALLMENTS".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            delivery_cents: 0,
            paid_cents: 500,
            notes: None,
            items: vec![SalesItemPayload {
                product_id: product.id,
                quantity: 2.0,
                unit_price_cents: 1_000,
            }],
        },
    )
    .unwrap();
    record_invoice_payment(
        &conn,
        user,
        "sales",
        invoice.id,
        InstallmentPaymentPayload {
            amount_cents: 700,
            payment_method: "card".to_string(),
            payment_date: today_date(),
            notes: Some("Second payment".to_string()),
        },
    )
    .unwrap();
    let refreshed = list_sales_invoices(&conn, Default::default()).unwrap();
    assert_eq!(refreshed[0].paid_cents, 1_200);
    assert_eq!(refreshed[0].remaining_cents, 800);
    assert_eq!(
        list_invoice_payments(&conn, "sales", invoice.id)
            .unwrap()
            .len(),
        2
    );

    cancel_sales_invoice(&conn, user, invoice.id).unwrap();
    assert!(list_invoice_payments(&conn, "sales", invoice.id)
        .unwrap()
        .is_empty());
    restore_sales_invoice(&conn, user, invoice.id).unwrap();
    let restored = list_sales_invoices(&conn, Default::default()).unwrap();
    assert_eq!(restored[0].paid_cents, 1_200);
    assert_eq!(
        list_invoice_payments(&conn, "sales", invoice.id)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn clear_all_data_is_credential_guarded_transactional_and_preserves_system_state() {
    let conn = test_conn();
    let user = make_admin(&conn);
    create_product(
        &conn,
        user,
        round_pipe_payload(None, "Product to clear", 1000, 1500),
    )
    .unwrap();
    let product_count_before: i64 = conn
        .query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))
        .unwrap();

    let unauthorized = clear_all_data(
        &conn,
        user,
        ClearAllDataPayload {
            admin_email: "admin@example.com".to_string(),
            admin_password: "wrong-password".to_string(),
            confirmation: "CLEAR ALL DATA".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(unauthorized.code, "UNAUTHORIZED");
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM products", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        product_count_before
    );

    conn.execute_batch(
        "CREATE TRIGGER fail_clear_all_data_test
         BEFORE INSERT ON categories
         WHEN NEW.id = 1
         BEGIN
             SELECT RAISE(ABORT, 'forced reset failure');
         END;",
    )
    .unwrap();
    clear_all_data(
        &conn,
        user,
        ClearAllDataPayload {
            admin_email: "admin@example.com".to_string(),
            admin_password: "1234".to_string(),
            confirmation: "CLEAR ALL DATA".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM products", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        product_count_before,
        "a failure after deletes must roll the entire reset back"
    );
    conn.execute_batch("DROP TRIGGER fail_clear_all_data_test;")
        .unwrap();

    let result = clear_all_data(
        &conn,
        user,
        ClearAllDataPayload {
            admin_email: "admin@example.com".to_string(),
            admin_password: "1234".to_string(),
            confirmation: "CLEAR ALL DATA".to_string(),
        },
    )
    .unwrap();
    assert!(result.deleted_records > 0);
    for table in [
        "products",
        "customers",
        "purchase_invoices",
        "purchase_returns",
        "purchase_return_items",
        "sales_invoices",
        "payments",
        "inventory_transactions",
        "expenses",
        "supplier_settlement_payments",
        "backups",
    ] {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0, "{table} must be empty after reset");
    }
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM company_settings", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        23
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM expense_categories", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        9
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM suppliers WHERE name = 'Unknown Supplier'",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM audit_logs WHERE action = 'clear_all_data'",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    let dashboard = dashboard_summary(&conn, Some(today_date())).unwrap();
    assert_eq!(dashboard.today_sales_count, 0);
    assert_eq!(dashboard.today_purchase_count, 0);
    assert_eq!(dashboard.today_sales_cents, 0);
    assert_eq!(dashboard.today_expenses_cents, 0);
    assert_eq!(dashboard.current_stock_value_cents, 0);
}

#[test]
fn partial_and_full_purchase_returns_adjust_stock_tax_and_supplier_balance() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier = make_supplier(&conn, user, "Return Supplier");
    let product = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier), "Damaged Pipe", 1_000, 1_500),
    )
    .unwrap();
    let purchase = create_purchase_invoice(
        &conn,
        user,
        PurchaseInvoicePayload {
            supplier_id: supplier,
            invoice_number: Some("PI-RETURN-001".to_string()),
            invoice_date: today_date(),
            discount_cents: 1_000,
            tax_cents: 500,
            shipping_cents: 500,
            paid_cents: 2_000,
            notes: None,
            items: vec![PurchaseItemPayload {
                product_id: product.id,
                quantity: 10.0,
                unit_cost_cents: 1_000,
            }],
        },
    )
    .unwrap();
    let context = get_purchase_return_context(&conn, purchase.id).unwrap();
    let invoice_item_id = context.items[0].purchase_invoice_item_id;

    let partial = create_purchase_return(
        &conn,
        user,
        PurchaseReturnPayload {
            purchase_invoice_id: purchase.id,
            return_date: today_date(),
            reason: Some("Damaged on arrival".to_string()),
            notes: None,
            idempotency_key: "return-partial-001".to_string(),
            items: vec![PurchaseReturnItemPayload {
                purchase_invoice_item_id: invoice_item_id,
                quantity: 4.0,
            }],
        },
    )
    .unwrap();
    assert_eq!(partial.return_record.subtotal_cents, 4_000);
    assert_eq!(partial.return_record.discount_cents, 400);
    assert_eq!(partial.return_record.tax_cents, 200);
    assert_eq!(partial.return_record.shipping_cents, 200);
    assert_eq!(partial.return_record.total_cents, 4_000);
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        6.0
    );
    assert_eq!(
        party_balance(&conn, PartyKind::Supplier, supplier).unwrap(),
        4_000
    );
    let after_partial = get_purchase_return_context(&conn, purchase.id).unwrap();
    assert_eq!(after_partial.invoice.returned_cents, 4_000);
    assert_eq!(after_partial.invoice.net_total_cents, 6_000);
    assert_eq!(after_partial.invoice.paid_cents, 2_000);
    assert_eq!(after_partial.invoice.remaining_cents, 4_000);
    assert_eq!(after_partial.items[0].returnable_quantity, 6.0);
    let supplier_statement = statement(
        &conn,
        PartyKind::Supplier,
        supplier,
        DateRangeFilters::default(),
    )
    .unwrap();
    assert!(supplier_statement.iter().any(|row| {
        row.entry_type == "Purchase Return"
            && row.reference == partial.return_record.return_number
            && row.credit_cents == 4_000
    }));

    let full = create_purchase_return(
        &conn,
        user,
        PurchaseReturnPayload {
            purchase_invoice_id: purchase.id,
            return_date: today_date(),
            reason: Some("Remaining batch rejected".to_string()),
            notes: None,
            idempotency_key: "return-full-remaining-001".to_string(),
            items: vec![PurchaseReturnItemPayload {
                purchase_invoice_item_id: invoice_item_id,
                quantity: 6.0,
            }],
        },
    )
    .unwrap();
    assert_eq!(full.return_record.discount_cents, 600);
    assert_eq!(full.return_record.tax_cents, 300);
    assert_eq!(full.return_record.shipping_cents, 300);
    assert_eq!(full.return_record.total_cents, 6_000);
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        0.0
    );
    assert_eq!(
        party_balance(&conn, PartyKind::Supplier, supplier).unwrap(),
        -2_000,
        "an already-paid fully returned purchase becomes supplier credit"
    );
    let after_full = get_purchase_return_context(&conn, purchase.id).unwrap();
    assert_eq!(after_full.invoice.returned_cents, 10_000);
    assert_eq!(after_full.invoice.net_total_cents, 0);
    assert_eq!(after_full.invoice.remaining_cents, 0);
    assert_eq!(after_full.invoice.payment_status, "paid");
}

#[test]
fn purchase_return_rejects_excess_quantity_without_partial_side_effects() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier = make_supplier(&conn, user, "Quantity Guard Supplier");
    let product = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier), "Quantity Guard Pipe", 1_000, 1_500),
    )
    .unwrap();
    let purchase = create_purchase_invoice(
        &conn,
        user,
        PurchaseInvoicePayload {
            supplier_id: supplier,
            invoice_number: Some("PI-RETURN-GUARD".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            shipping_cents: 0,
            paid_cents: 0,
            notes: None,
            items: vec![PurchaseItemPayload {
                product_id: product.id,
                quantity: 5.0,
                unit_cost_cents: 1_000,
            }],
        },
    )
    .unwrap();
    let invoice_item_id = get_purchase_return_context(&conn, purchase.id)
        .unwrap()
        .items[0]
        .purchase_invoice_item_id;
    let error = create_purchase_return(
        &conn,
        user,
        PurchaseReturnPayload {
            purchase_invoice_id: purchase.id,
            return_date: today_date(),
            reason: None,
            notes: None,
            idempotency_key: "return-too-large-001".to_string(),
            items: vec![PurchaseReturnItemPayload {
                purchase_invoice_item_id: invoice_item_id,
                quantity: 5.001,
            }],
        },
    )
    .unwrap_err();
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(error.message.contains("returnable quantity"));
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM purchase_returns", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        5.0
    );

    adjust_stock(
        &conn,
        user,
        crate::models::StockAdjustmentPayload {
            product_id: product.id,
            transaction_type: "damaged_stock".to_string(),
            quantity: 4.0,
            unit_cost_cents: Some(1_000),
            notes: Some("Only one unit remains physically available".to_string()),
        },
    )
    .unwrap();
    let insufficient_stock = create_purchase_return(
        &conn,
        user,
        PurchaseReturnPayload {
            purchase_invoice_id: purchase.id,
            return_date: today_date(),
            reason: None,
            notes: None,
            idempotency_key: "return-no-stock-001".to_string(),
            items: vec![PurchaseReturnItemPayload {
                purchase_invoice_item_id: invoice_item_id,
                quantity: 2.0,
            }],
        },
    )
    .unwrap_err();
    assert_eq!(insufficient_stock.code, "INSUFFICIENT_STOCK");
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM purchase_returns", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap(),
        0,
        "the return record and financial effect must roll back with the stock failure"
    );
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        1.0
    );
}

#[test]
fn duplicate_purchase_return_requests_are_idempotent() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier = make_supplier(&conn, user, "Idempotent Supplier");
    let product = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier), "Idempotent Pipe", 700, 1_100),
    )
    .unwrap();
    let purchase = create_purchase_invoice(
        &conn,
        user,
        PurchaseInvoicePayload {
            supplier_id: supplier,
            invoice_number: Some("PI-IDEMPOTENT".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            shipping_cents: 0,
            paid_cents: 0,
            notes: None,
            items: vec![PurchaseItemPayload {
                product_id: product.id,
                quantity: 8.0,
                unit_cost_cents: 700,
            }],
        },
    )
    .unwrap();
    let invoice_item_id = get_purchase_return_context(&conn, purchase.id)
        .unwrap()
        .items[0]
        .purchase_invoice_item_id;
    let payload = PurchaseReturnPayload {
        purchase_invoice_id: purchase.id,
        return_date: today_date(),
        reason: Some("Duplicate network retry".to_string()),
        notes: None,
        idempotency_key: "same-return-request-001".to_string(),
        items: vec![PurchaseReturnItemPayload {
            purchase_invoice_item_id: invoice_item_id,
            quantity: 2.0,
        }],
    };
    let first = create_purchase_return(&conn, user, payload.clone()).unwrap();
    let retry = create_purchase_return(&conn, user, payload).unwrap();
    assert_eq!(first.return_record.id, retry.return_record.id);
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM purchase_returns", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap(),
        1
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM inventory_transactions
             WHERE purchase_return_id = ?1",
            [first.return_record.id],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        6.0
    );
}

#[test]
fn cancelling_and_restoring_purchase_return_reverses_effects_once() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier = make_supplier(&conn, user, "Reversal Supplier");
    let product = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier), "Reversal Pipe", 1_000, 1_500),
    )
    .unwrap();
    let purchase = create_purchase_invoice(
        &conn,
        user,
        PurchaseInvoicePayload {
            supplier_id: supplier,
            invoice_number: Some("PI-REVERSAL".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            shipping_cents: 0,
            paid_cents: 0,
            notes: None,
            items: vec![PurchaseItemPayload {
                product_id: product.id,
                quantity: 10.0,
                unit_cost_cents: 1_000,
            }],
        },
    )
    .unwrap();
    let invoice_item_id = get_purchase_return_context(&conn, purchase.id)
        .unwrap()
        .items[0]
        .purchase_invoice_item_id;
    let purchase_return = create_purchase_return(
        &conn,
        user,
        PurchaseReturnPayload {
            purchase_invoice_id: purchase.id,
            return_date: today_date(),
            reason: None,
            notes: None,
            idempotency_key: "return-reversal-001".to_string(),
            items: vec![PurchaseReturnItemPayload {
                purchase_invoice_item_id: invoice_item_id,
                quantity: 3.0,
            }],
        },
    )
    .unwrap();
    assert_eq!(
        party_balance(&conn, PartyKind::Supplier, supplier).unwrap(),
        7_000
    );

    cancel_purchase_return(&conn, user, purchase_return.return_record.id).unwrap();
    cancel_purchase_return(&conn, user, purchase_return.return_record.id).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        10.0
    );
    assert_eq!(
        party_balance(&conn, PartyKind::Supplier, supplier).unwrap(),
        10_000
    );

    restore_purchase_return(&conn, user, purchase_return.return_record.id).unwrap();
    restore_purchase_return(&conn, user, purchase_return.return_record.id).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        7.0
    );
    assert_eq!(
        party_balance(&conn, PartyKind::Supplier, supplier).unwrap(),
        7_000
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM inventory_transactions
             WHERE purchase_return_id = ?1 AND status = 'active'",
            [purchase_return.return_record.id],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    let cancel_invoice_error = cancel_purchase_invoice(&conn, user, purchase.id).unwrap_err();
    assert!(cancel_invoice_error
        .message
        .contains("active purchase returns"));
}

#[test]
fn editing_purchase_return_replaces_effects_and_preserves_old_ledger_revision() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier = make_supplier(&conn, user, "Edit Return Supplier");
    let product = create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier), "Edit Return Pipe", 1_000, 1_500),
    )
    .unwrap();
    let purchase = create_purchase_invoice(
        &conn,
        user,
        PurchaseInvoicePayload {
            supplier_id: supplier,
            invoice_number: Some("PI-EDIT-RETURN".to_string()),
            invoice_date: today_date(),
            discount_cents: 0,
            tax_cents: 0,
            shipping_cents: 0,
            paid_cents: 0,
            notes: None,
            items: vec![PurchaseItemPayload {
                product_id: product.id,
                quantity: 10.0,
                unit_cost_cents: 1_000,
            }],
        },
    )
    .unwrap();
    let invoice_item_id = get_purchase_return_context(&conn, purchase.id)
        .unwrap()
        .items[0]
        .purchase_invoice_item_id;
    let purchase_return = create_purchase_return(
        &conn,
        user,
        PurchaseReturnPayload {
            purchase_invoice_id: purchase.id,
            return_date: today_date(),
            reason: None,
            notes: None,
            idempotency_key: "return-edit-001".to_string(),
            items: vec![PurchaseReturnItemPayload {
                purchase_invoice_item_id: invoice_item_id,
                quantity: 2.0,
            }],
        },
    )
    .unwrap();
    update_purchase_return(
        &conn,
        user,
        purchase_return.return_record.id,
        PurchaseReturnUpdatePayload {
            return_date: today_date(),
            reason: Some("Inspection found one more damaged piece".to_string()),
            notes: None,
            items: vec![PurchaseReturnItemPayload {
                purchase_invoice_item_id: invoice_item_id,
                quantity: 3.0,
            }],
        },
    )
    .unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        7.0
    );
    assert_eq!(
        party_balance(&conn, PartyKind::Supplier, supplier).unwrap(),
        7_000
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM inventory_transactions
             WHERE purchase_return_id = ?1 AND status = 'cancelled'",
            [purchase_return.return_record.id],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1,
        "the superseded inventory effect remains as cancelled audit history"
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM inventory_transactions
             WHERE purchase_return_id = ?1 AND status = 'active'",
            [purchase_return.return_record.id],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM purchase_return_items
             WHERE purchase_return_id = ?1 AND status = 'superseded'",
            [purchase_return.return_record.id],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    cancel_purchase_return(&conn, user, purchase_return.return_record.id).unwrap();
    restore_purchase_return(&conn, user, purchase_return.return_record.id).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT current_quantity FROM stock_levels WHERE product_id = ?1",
            [product.id],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        7.0,
        "restoring an edited return must not reactivate its superseded revision"
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM inventory_transactions
             WHERE purchase_return_id = ?1 AND status = 'active'",
            [purchase_return.return_record.id],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
}

#[test]
fn stock_reports_expose_name_thickness_price_and_render_them_in_that_order() {
    let conn = test_conn();
    let user = make_admin(&conn);
    let supplier = make_supplier(&conn, user, "Report Order Supplier");
    create_product(
        &conn,
        user,
        round_pipe_payload(Some(supplier), "Ordered Pipe", 1_000, 1_750),
    )
    .unwrap();
    let count_rows = stock_count_report(&conn, ReportFilters::default()).unwrap();
    let row = count_rows
        .iter()
        .find(|row| row["product_name"] == "Ordered Pipe")
        .unwrap();
    assert_eq!(row["thickness_mm"], 2.0);
    assert_eq!(row["selling_price_cents"], 1_750);

    let report_source = include_str!("../../src/features/reports/ReportsPage.tsx");
    let order_position = report_source
        .find(r#"["product_name", "thickness_mm", "selling_price_cents"]"#)
        .expect("stock report column order must be explicit");
    let print_position = report_source
        .find("<th>Name</th><th>Thickness</th><th>Price</th>")
        .expect("stock count print order must match the table/export order");
    assert!(order_position < print_position);
}
