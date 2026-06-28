export type InvoiceListRow = {
  id: number;
  invoice_number: string;
  invoice_date: string;
  party_name: string;
  subtotal_cents: number;
  discount_cents: number;
  tax_cents: number;
  extra_cents: number;
  total_cents: number;
  paid_cents: number;
  remaining_cents: number;
  payment_status: string;
  status: string;
  notes?: string | null;
  created_at: string;
};

export type InvoiceSaveResult = {
  id: number;
  invoice_number: string;
};

export type PurchaseInvoicePayload = {
  supplier_id: number;
  invoice_number?: string | null;
  invoice_date: string;
  discount_cents: number;
  tax_cents: number;
  shipping_cents: number;
  paid_cents: number;
  notes?: string | null;
  items: Array<{
    product_id: number;
    quantity: number;
    unit_cost_cents: number;
  }>;
};

export type SalesInvoicePayload = {
  customer_id?: number | null;
  invoice_number?: string | null;
  invoice_date: string;
  discount_cents: number;
  tax_cents: number;
  delivery_cents: number;
  paid_cents: number;
  notes?: string | null;
  items: Array<{
    product_id: number;
    quantity: number;
    unit_price_cents: number;
  }>;
};
