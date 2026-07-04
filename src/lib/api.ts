import { call } from "./tauri";
import type { AdminUser, BackupRow, Category, CompanySettings, DateRangeFilters } from "../types/common";
import type { InvoiceListRow, InvoiceSaveResult, PurchaseInvoicePayload, SalesInvoicePayload } from "../types/invoice";
import type { ExpenseCategory, ExpensePayload, ExpenseRow, PaymentPayload, PaymentRow } from "../types/payment";
import type { Party, PartyPayload, StatementRow } from "../types/party";
import type {
  InventoryTransaction,
  Product,
  ProductPayload,
  SettlementFilters,
  SettlementPayment,
  SettlementPaymentPayload,
  SupplierVariant,
  VariantFilters
} from "../types/product";
import type { DashboardSummary, ReportFilters, ReportRow } from "../types/report";

export const authApi = {
  hasAdmin: () => call<boolean>("has_admin"),
  setup: (payload: { full_name: string; email: string; password: string }) =>
    call<AdminUser>("setup_admin", { payload }),
  login: (payload: { email: string; password: string }) => call<AdminUser>("login_admin", { payload }),
  logout: () => call<void>("logout_admin"),
  current: () => call<AdminUser | null>("get_current_admin"),
  changePassword: (payload: { current_password: string; new_password: string }) =>
    call<void>("change_admin_password", { payload })
};

export const categoryApi = {
  list: () => call<Category[]>("list_categories"),
  create: (payload: { name: string; parent_id?: number | null; description?: string | null }) =>
    call<Category>("create_category", { payload }),
  update: (id: number, payload: { name: string; parent_id?: number | null; description?: string | null }) =>
    call<Category>("update_category", { id, payload }),
  archive: (id: number) => call<void>("archive_category", { id })
};

export const productApi = {
  list: (filters = {}) => call<Product[]>("list_products", { filters }),
  get: (id: number) => call<Product>("get_product", { id }),
  create: (payload: ProductPayload) => call<Product>("create_product", { payload }),
  update: (id: number, payload: ProductPayload) => call<Product>("update_product", { id, payload }),
  archive: (id: number) => call<void>("archive_product", { id }),
  movement: (product_id: number, filters: DateRangeFilters = {}) =>
    call<InventoryTransaction[]>("get_product_movement", { product_id, filters }),
  adjustStock: (payload: {
    product_id: number;
    transaction_type: string;
    quantity: number;
    unit_cost_cents?: number | null;
    notes?: string | null;
  }) => call<void>("adjust_stock", { payload }),
  generateSku: (payload: ProductPayload) => call<string>("generate_product_sku", { payload }),
  supplierVariants: (filters: VariantFilters = {}) =>
    call<SupplierVariant[]>("list_supplier_variants", { filters })
};

export const settlementApi = {
  list: (filters: SettlementFilters = {}) => call<SettlementPayment[]>("list_settlement_payments", { filters }),
  create: (payload: SettlementPaymentPayload) => call<SettlementPayment>("create_settlement_payment", { payload }),
  delete: (id: number) => call<void>("delete_settlement_payment", { id })
};

function partyApi(kind: "supplier" | "customer") {
  return {
    list: (filters = {}) => call<Party[]>(`list_${kind}s`, { filters }),
    get: (id: number) => call<Party>(`get_${kind}`, { id }),
    create: (payload: PartyPayload) => call<Party>(`create_${kind}`, { payload }),
    update: (id: number, payload: PartyPayload) => call<Party>(`update_${kind}`, { id, payload }),
    archive: (id: number) => call<void>(`archive_${kind}`, { id }),
    statement: (partyId: number, filters: DateRangeFilters = {}) =>
      call<StatementRow[]>(`get_${kind}_statement`, { [`${kind}_id`]: partyId, filters })
  };
}

export const supplierApi = partyApi("supplier");
export const customerApi = partyApi("customer");

export const purchaseApi = {
  list: (filters = {}) => call<InvoiceListRow[]>("list_purchase_invoices", { filters }),
  create: (payload: PurchaseInvoicePayload) => call<InvoiceSaveResult>("create_purchase_invoice", { payload }),
  cancel: (id: number) => call<void>("cancel_purchase_invoice", { id }),
  print: (id: number) => call<string>("print_purchase_invoice", { id })
};

export const salesApi = {
  list: (filters = {}) => call<InvoiceListRow[]>("list_sales_invoices", { filters }),
  create: (payload: SalesInvoicePayload) => call<InvoiceSaveResult>("create_sales_invoice", { payload }),
  cancel: (id: number) => call<void>("cancel_sales_invoice", { id }),
  print: (id: number) => call<string>("print_sales_invoice", { id })
};

export const expenseApi = {
  categories: () => call<ExpenseCategory[]>("list_expense_categories"),
  list: (filters = {}) => call<ExpenseRow[]>("list_expenses", { filters }),
  create: (payload: ExpensePayload) => call<ExpenseRow>("create_expense", { payload }),
  update: (id: number, payload: ExpensePayload) => call<ExpenseRow>("update_expense", { id, payload }),
  delete: (id: number) => call<void>("delete_expense", { id })
};

export const paymentApi = {
  list: (filters = {}) => call<PaymentRow[]>("list_payments", { filters }),
  create: (payload: PaymentPayload) => call<PaymentRow>("create_payment", { payload }),
  delete: (id: number) => call<void>("delete_payment", { id })
};

export const reportApi = {
  dashboard: (date?: string) => call<DashboardSummary>("get_dashboard_summary", { date: date ?? null }),
  dailySales: (filters: ReportFilters) => call<ReportRow[]>("get_daily_sales_report", { filters }),
  profit: (filters: ReportFilters) => call<ReportRow[]>("get_profit_report", { filters }),
  monthlyProfit: (filters: ReportFilters) => call<ReportRow[]>("get_monthly_profit_report", { filters }),
  stock: (filters: ReportFilters) => call<ReportRow[]>("get_stock_report", { filters }),
  stockMovement: (filters: ReportFilters) => call<ReportRow[]>("get_stock_movement_report", { filters }),
  lowStock: () => call<ReportRow[]>("get_low_stock_report"),
  purchase: (filters: ReportFilters) => call<ReportRow[]>("get_purchase_report", { filters }),
  supplierDebt: (filters: ReportFilters) => call<ReportRow[]>("get_supplier_debt_report", { filters }),
  customerDebt: (filters: ReportFilters) => call<ReportRow[]>("get_customer_debt_report", { filters }),
  expense: (filters: ReportFilters) => call<ReportRow[]>("get_expense_report", { filters }),
  payment: (filters: ReportFilters) => call<ReportRow[]>("get_payment_report", { filters }),
  inventoryValue: () => call<ReportRow[]>("get_inventory_value_report"),
  bestSelling: (filters: ReportFilters) => call<ReportRow[]>("get_best_selling_products_report", { filters }),
  stockCount: (filters: ReportFilters) => call<ReportRow[]>("get_stock_count_report", { filters }),
  cheapestSupplier: (filters: ReportFilters) => call<ReportRow[]>("get_cheapest_supplier_report", { filters }),
  supplierSettlement: (filters: ReportFilters) => call<ReportRow[]>("get_supplier_settlement_report", { filters }),
  supplierSettlementSummary: (filters: ReportFilters) =>
    call<ReportRow[]>("get_supplier_settlement_summary", { filters })
};

export const seedApi = {
  demoData: () => call<{ inserted: boolean; message: string }>("seed_demo_data")
};

export const settingsApi = {
  get: () => call<CompanySettings>("get_company_settings"),
  update: (payload: Omit<CompanySettings, "id" | "created_at" | "updated_at">) =>
    call<CompanySettings>("update_company_settings", { payload })
};

export const backupApi = {
  list: () => call<BackupRow[]>("list_backups"),
  create: () => call<BackupRow>("create_manual_backup"),
  restore: (path: string) => call<void>("restore_backup", { path })
};
