export type QuotationStatus =
  | "draft"
  | "sent"
  | "accepted"
  | "rejected"
  | "expired"
  | "converted";

export type QuotationItemPayload = {
  product_id: number;
  quantity: number;
  unit_price_cents: number;
};

export type QuotationPayload = {
  customer_id: number;
  quotation_number?: string | null;
  quotation_date: string;
  valid_until: string;
  discount_cents: number;
  tax_cents: number;
  notes?: string | null;
  items: QuotationItemPayload[];
};

export type QuotationFilters = {
  search?: string | null;
  status?: QuotationStatus | null;
  date_from?: string | null;
  date_to?: string | null;
};

export type QuotationListRow = {
  id: number;
  customer_id?: number | null;
  quotation_number: string;
  quotation_date: string;
  valid_until: string;
  customer_name: string;
  subtotal_cents: number;
  discount_cents: number;
  tax_cents: number;
  total_cents: number;
  status: QuotationStatus;
  converted_sales_invoice_id?: number | null;
  notes?: string | null;
  created_at: string;
  updated_at: string;
};

export type QuotationItemRow = {
  id: number;
  product_id?: number | null;
  sku: string;
  product_name: string;
  quantity: number;
  unit_price_cents: number;
  line_total_cents: number;
};

export type QuotationDetail = {
  quotation: QuotationListRow;
  customer_company_name?: string | null;
  customer_phone?: string | null;
  customer_email?: string | null;
  customer_address?: string | null;
  customer_tax_number?: string | null;
  items: QuotationItemRow[];
};

export type QuotationConversionPayload = {
  invoice_number?: string | null;
  invoice_date: string;
  delivery_cents: number;
  paid_cents: number;
};
