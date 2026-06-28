export type AdminUser = {
  id: number;
  full_name: string;
  email: string;
  role: string;
};

export type Category = {
  id: number;
  name: string;
  parent_id?: number | null;
  description?: string | null;
  is_active: boolean;
  created_at: string;
  updated_at: string;
};

export type DateRangeFilters = {
  date_from?: string | null;
  date_to?: string | null;
};

export type CompanySettings = {
  id: number;
  company_name: string;
  phone?: string | null;
  email?: string | null;
  address?: string | null;
  tax_number?: string | null;
  default_currency: string;
  invoice_prefix_sales: string;
  invoice_prefix_purchase: string;
  allow_negative_stock: boolean;
  backup_path?: string | null;
  default_tax_rate: number;
  default_profit_method: string;
  created_at: string;
  updated_at: string;
};

export type BackupRow = {
  id: number;
  backup_path: string;
  backup_type: string;
  status: string;
  notes?: string | null;
  created_at: string;
};
