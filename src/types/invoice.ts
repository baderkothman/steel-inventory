export type InvoiceListRow = {
  id: number;
  party_id?: number | null;
  invoice_number: string;
  invoice_date: string;
  party_name: string;
  subtotal_cents: number;
  discount_cents: number;
  tax_cents: number;
  extra_cents: number;
  total_cents: number;
  returned_cents: number;
  net_total_cents: number;
  paid_cents: number;
  remaining_cents: number;
  payment_status: string;
  status: string;
  notes?: string | null;
  created_at: string;
  deleted_at?: string | null;
};

export type PurchaseReturnItemPayload = {
  purchase_invoice_item_id: number;
  quantity: number;
};

export type PurchaseReturnPayload = {
  purchase_invoice_id: number;
  return_date: string;
  reason?: string | null;
  notes?: string | null;
  idempotency_key: string;
  items: PurchaseReturnItemPayload[];
};

export type PurchaseReturnUpdatePayload = Omit<
  PurchaseReturnPayload,
  "purchase_invoice_id" | "idempotency_key"
>;

export type PurchaseReturnRow = {
  id: number;
  purchase_invoice_id: number;
  supplier_id: number;
  return_number: string;
  return_date: string;
  reason?: string | null;
  notes?: string | null;
  subtotal_cents: number;
  discount_cents: number;
  tax_cents: number;
  shipping_cents: number;
  total_cents: number;
  status: "active" | "cancelled";
  created_at: string;
  updated_at: string;
  cancelled_at?: string | null;
};

export type PurchaseReturnItemRow = {
  id: number;
  purchase_return_id: number;
  purchase_invoice_item_id: number;
  product_id: number;
  sku: string;
  product_name: string;
  quantity: number;
  unit_cost_cents: number;
  total_cost_cents: number;
};

export type PurchaseReturnDetail = {
  return_record: PurchaseReturnRow;
  items: PurchaseReturnItemRow[];
};

export type PurchaseReturnableItem = {
  purchase_invoice_item_id: number;
  product_id: number;
  sku: string;
  product_name: string;
  purchased_quantity: number;
  returned_quantity: number;
  returnable_quantity: number;
  unit_cost_cents: number;
};

export type PurchaseReturnContext = {
  invoice: InvoiceListRow;
  items: PurchaseReturnableItem[];
  returns: PurchaseReturnDetail[];
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
