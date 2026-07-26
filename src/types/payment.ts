export type ExpenseCategory = {
  id: number;
  name: string;
  description?: string | null;
  is_active: boolean;
};

export type ExpenseRow = {
  id: number;
  expense_category_id: number;
  category_name: string;
  title: string;
  amount_cents: number;
  paid_cents: number;
  remaining_cents: number;
  payment_status: string;
  currency: string;
  expense_date: string;
  payment_method: string;
  notes?: string | null;
  status: string;
  created_at: string;
  updated_at: string;
  deleted_at?: string | null;
};

export type ExpensePayload = {
  expense_category_id: number;
  title: string;
  amount_cents: number;
  paid_cents: number;
  currency: string;
  expense_date: string;
  payment_method: string;
  notes?: string | null;
};

export type InstallmentPaymentRow = {
  id: number;
  amount_cents: number;
  currency: string;
  payment_method: string;
  payment_date: string;
  notes?: string | null;
  status: string;
  created_at: string;
};

export type InstallmentPaymentPayload = {
  amount_cents: number;
  payment_method: string;
  payment_date: string;
  notes?: string | null;
};

export type PaymentRow = {
  id: number;
  party_type: string;
  party_id: number;
  party_name: string;
  payment_direction: string;
  amount_cents: number;
  currency: string;
  payment_method: string;
  payment_date: string;
  reference_type?: string | null;
  reference_id?: number | null;
  notes?: string | null;
  status: string;
  created_at: string;
  deleted_at?: string | null;
};

export type PaymentPayload = {
  party_type: string;
  party_id: number;
  amount_cents: number;
  currency: string;
  payment_method: string;
  payment_date: string;
  reference_type?: string | null;
  reference_id?: number | null;
  notes?: string | null;
};
