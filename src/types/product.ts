export type Product = {
  id: number;
  sku: string;
  category_id: number;
  category_name: string;
  supplier_id?: number | null;
  supplier_name: string;
  spec_key: string;
  location?: string | null;
  name: string;
  product_type: string;
  material: string;
  shape: string;
  finish: string;
  size_label?: string | null;
  width_mm?: number | null;
  height_mm?: number | null;
  diameter_mm?: number | null;
  thickness_mm?: number | null;
  length_mm?: number | null;
  unit: string;
  description?: string | null;
  is_active: boolean;
  cost_price_cents: number;
  selling_price_cents: number;
  wholesale_price_cents: number;
  current_quantity: number;
  minimum_quantity: number;
  created_at: string;
  updated_at: string;
};

export type ProductPayload = {
  sku?: string | null;
  category_id: number;
  supplier_id?: number | null;
  location?: string | null;
  name: string;
  product_type: string;
  material: string;
  shape: string;
  finish: string;
  size_label: string;
  width_mm?: number | null;
  height_mm?: number | null;
  diameter_mm?: number | null;
  thickness_mm?: number | null;
  length_mm?: number | null;
  unit: string;
  description?: string | null;
  cost_price_cents: number;
  selling_price_cents: number;
  wholesale_price_cents?: number | null;
  minimum_quantity: number;
  initial_quantity?: number | null;
};

export type SupplierVariant = {
  spec_key: string;
  product_id: number;
  sku: string;
  name: string;
  supplier_id?: number | null;
  supplier_name: string;
  category_name: string;
  unit: string;
  location?: string | null;
  cost_price_cents: number;
  selling_price_cents: number;
  current_quantity: number;
  is_active: boolean;
};

export type VariantFilters = {
  search?: string | null;
  category_id?: number | null;
  in_stock_only?: boolean | null;
};

export type SettlementPaymentPayload = {
  supplier_id: number;
  period_start: string;
  period_end: string;
  amount_cents: number;
  status: string;
  payment_date: string;
  reference?: string | null;
  notes?: string | null;
};

export type SettlementPayment = {
  id: number;
  supplier_id: number;
  supplier_name: string;
  period_start: string;
  period_end: string;
  amount_cents: number;
  currency: string;
  status: string;
  payment_date: string;
  reference?: string | null;
  notes?: string | null;
  created_at: string;
};

export type SettlementFilters = {
  date_from?: string | null;
  date_to?: string | null;
  supplier_id?: number | null;
};

export type InventoryTransaction = {
  id: number;
  product_id: number;
  product_name: string;
  sku: string;
  transaction_type: string;
  reference_type: string;
  reference_id?: number | null;
  quantity_in: number;
  quantity_out: number;
  unit_cost_cents?: number | null;
  notes?: string | null;
  created_at: string;
};
