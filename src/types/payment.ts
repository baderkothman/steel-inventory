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
  currency: string;
  expense_date: string;
  payment_method: string;
  notes?: string | null;
  created_at: string;
  updated_at: string;
};

export type ExpensePayload = {
  expense_category_id: number;
  title: string;
  amount_cents: number;
  currency: string;
  expense_date: string;
  payment_method: string;
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
  created_at: string;
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
