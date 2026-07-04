use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: i64,
    pub full_name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct SetupAdminPayload {
    pub full_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordPayload {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CategoryPayload {
    pub name: String,
    pub parent_id: Option<i64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRow {
    pub id: i64,
    pub sku: String,
    pub category_id: i64,
    pub category_name: String,
    pub supplier_id: Option<i64>,
    pub supplier_name: String,
    pub spec_key: String,
    pub location: Option<String>,
    pub name: String,
    pub product_type: String,
    pub material: String,
    pub shape: String,
    pub finish: String,
    pub size_label: Option<String>,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub diameter_mm: Option<f64>,
    pub thickness_mm: Option<f64>,
    pub length_mm: Option<f64>,
    pub unit: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub cost_price_cents: i64,
    pub selling_price_cents: i64,
    pub wholesale_price_cents: i64,
    pub current_quantity: f64,
    pub minimum_quantity: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProductPayload {
    pub sku: Option<String>,
    pub category_id: i64,
    pub supplier_id: Option<i64>,
    pub location: Option<String>,
    pub name: String,
    pub product_type: String,
    pub material: String,
    pub shape: String,
    pub finish: String,
    pub size_label: String,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub diameter_mm: Option<f64>,
    pub thickness_mm: Option<f64>,
    pub length_mm: Option<f64>,
    pub unit: String,
    pub description: Option<String>,
    pub cost_price_cents: i64,
    pub selling_price_cents: i64,
    pub wholesale_price_cents: Option<i64>,
    pub minimum_quantity: f64,
    pub initial_quantity: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProductFilters {
    pub search: Option<String>,
    pub category_id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub active_only: Option<bool>,
}

/// One supplier's variant of a shared product specification, for cheapest comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierVariantRow {
    pub spec_key: String,
    pub product_id: i64,
    pub sku: String,
    pub name: String,
    pub supplier_id: Option<i64>,
    pub supplier_name: String,
    pub category_name: String,
    pub unit: String,
    pub location: Option<String>,
    pub cost_price_cents: i64,
    pub selling_price_cents: i64,
    pub current_quantity: f64,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct VariantFilters {
    pub search: Option<String>,
    pub category_id: Option<i64>,
    pub in_stock_only: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SettlementPaymentPayload {
    pub supplier_id: i64,
    pub period_start: String,
    pub period_end: String,
    pub amount_cents: i64,
    pub status: String,
    pub payment_date: String,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementPaymentRow {
    pub id: i64,
    pub supplier_id: i64,
    pub supplier_name: String,
    pub period_start: String,
    pub period_end: String,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub payment_date: String,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct SettlementFilters {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub supplier_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryTransactionRow {
    pub id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub sku: String,
    pub transaction_type: String,
    pub reference_type: String,
    pub reference_id: Option<i64>,
    pub quantity_in: f64,
    pub quantity_out: f64,
    pub unit_cost_cents: Option<i64>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct MovementFilters {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StockAdjustmentPayload {
    pub product_id: i64,
    pub transaction_type: String,
    pub quantity: f64,
    pub unit_cost_cents: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyRow {
    pub id: i64,
    pub name: String,
    pub company_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub tax_number: Option<String>,
    pub opening_balance_cents: i64,
    pub notes: Option<String>,
    pub is_active: bool,
    pub balance_cents: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PartyPayload {
    pub name: String,
    pub company_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub tax_number: Option<String>,
    pub opening_balance_cents: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PartyFilters {
    pub search: Option<String>,
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct DateRangeFilters {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementRow {
    pub date: String,
    pub entry_type: String,
    pub reference: String,
    pub debit_cents: i64,
    pub credit_cents: i64,
    pub balance_cents: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PurchaseItemPayload {
    pub product_id: i64,
    pub quantity: f64,
    pub unit_cost_cents: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PurchaseInvoicePayload {
    pub supplier_id: i64,
    pub invoice_number: Option<String>,
    pub invoice_date: String,
    pub discount_cents: i64,
    pub tax_cents: i64,
    pub shipping_cents: i64,
    pub paid_cents: i64,
    pub notes: Option<String>,
    pub items: Vec<PurchaseItemPayload>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SalesItemPayload {
    pub product_id: i64,
    pub quantity: f64,
    pub unit_price_cents: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SalesInvoicePayload {
    pub customer_id: Option<i64>,
    pub invoice_number: Option<String>,
    pub invoice_date: String,
    pub discount_cents: i64,
    pub tax_cents: i64,
    pub delivery_cents: i64,
    pub paid_cents: i64,
    pub notes: Option<String>,
    pub items: Vec<SalesItemPayload>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvoiceSaveResult {
    pub id: i64,
    pub invoice_number: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct InvoiceFilters {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub party_id: Option<i64>,
    pub payment_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvoiceListRow {
    pub id: i64,
    pub invoice_number: String,
    pub invoice_date: String,
    pub party_name: String,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub tax_cents: i64,
    pub extra_cents: i64,
    pub total_cents: i64,
    pub paid_cents: i64,
    pub remaining_cents: i64,
    pub payment_status: String,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvoiceItemRow {
    pub id: i64,
    pub product_id: i64,
    pub sku: String,
    pub product_name: String,
    pub quantity: f64,
    pub unit_cost_cents: i64,
    pub unit_price_cents: i64,
    pub row_total_cents: i64,
    pub profit_cents: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvoiceDetail {
    pub invoice: InvoiceListRow,
    pub items: Vec<InvoiceItemRow>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExpenseCategory {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExpensePayload {
    pub expense_category_id: i64,
    pub title: String,
    pub amount_cents: i64,
    pub currency: String,
    pub expense_date: String,
    pub payment_method: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ExpenseFilters {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub expense_category_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExpenseRow {
    pub id: i64,
    pub expense_category_id: i64,
    pub category_name: String,
    pub title: String,
    pub amount_cents: i64,
    pub currency: String,
    pub expense_date: String,
    pub payment_method: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PaymentPayload {
    pub party_type: String,
    pub party_id: i64,
    pub amount_cents: i64,
    pub currency: String,
    pub payment_method: String,
    pub payment_date: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PaymentFilters {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub party_type: Option<String>,
    pub party_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentRow {
    pub id: i64,
    pub party_type: String,
    pub party_id: i64,
    pub party_name: String,
    pub payment_direction: String,
    pub amount_cents: i64,
    pub currency: String,
    pub payment_method: String,
    pub payment_date: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<i64>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanySettings {
    pub id: i64,
    pub company_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub tax_number: Option<String>,
    pub default_currency: String,
    pub invoice_prefix_sales: String,
    pub invoice_prefix_purchase: String,
    pub allow_negative_stock: bool,
    pub backup_path: Option<String>,
    pub default_tax_rate: f64,
    pub default_profit_method: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CompanySettingsPayload {
    pub company_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub tax_number: Option<String>,
    pub default_currency: String,
    pub invoice_prefix_sales: String,
    pub invoice_prefix_purchase: String,
    pub allow_negative_stock: bool,
    pub backup_path: Option<String>,
    pub default_tax_rate: f64,
    pub default_profit_method: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardSummary {
    pub date: String,
    pub today_sales_cents: i64,
    pub today_paid_cents: i64,
    pub today_remaining_cents: i64,
    pub today_profit_cents: i64,
    pub today_expenses_cents: i64,
    pub net_profit_cents: i64,
    pub total_customer_debts_cents: i64,
    pub total_supplier_debts_cents: i64,
    pub low_stock_count: i64,
    pub current_stock_value_cents: i64,
    pub low_stock_products: Vec<ProductRow>,
    pub recent_sales_invoices: Vec<InvoiceListRow>,
    pub recent_purchase_invoices: Vec<InvoiceListRow>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupRow {
    pub id: i64,
    pub backup_path: String,
    pub backup_type: String,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DemoSeedResult {
    pub inserted: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ReportFilters {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub product_id: Option<i64>,
    pub category_id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub payment_status: Option<String>,
}
