export type Party = {
  id: number;
  name: string;
  company_name?: string | null;
  phone?: string | null;
  email?: string | null;
  address?: string | null;
  tax_number?: string | null;
  opening_balance_cents: number;
  notes?: string | null;
  is_active: boolean;
  balance_cents: number;
  created_at: string;
  updated_at: string;
};

export type PartyPayload = {
  name: string;
  company_name?: string | null;
  phone?: string | null;
  email?: string | null;
  address?: string | null;
  tax_number?: string | null;
  opening_balance_cents: number;
  notes?: string | null;
};

export type StatementRow = {
  date: string;
  entry_type: string;
  reference: string;
  debit_cents: number;
  credit_cents: number;
  balance_cents: number;
};
