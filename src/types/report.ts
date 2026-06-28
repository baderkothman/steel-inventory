import type { InvoiceListRow } from "./invoice";
import type { Product } from "./product";

export type DashboardSummary = {
  date: string;
  today_sales_cents: number;
  today_paid_cents: number;
  today_remaining_cents: number;
  today_profit_cents: number;
  today_expenses_cents: number;
  net_profit_cents: number;
  total_customer_debts_cents: number;
  total_supplier_debts_cents: number;
  low_stock_count: number;
  current_stock_value_cents: number;
  low_stock_products: Product[];
  recent_sales_invoices: InvoiceListRow[];
  recent_purchase_invoices: InvoiceListRow[];
};

export type ReportFilters = {
  date_from?: string | null;
  date_to?: string | null;
  product_id?: number | null;
  category_id?: number | null;
  supplier_id?: number | null;
  customer_id?: number | null;
  payment_status?: string | null;
};

export type ReportRow = Record<string, unknown>;
