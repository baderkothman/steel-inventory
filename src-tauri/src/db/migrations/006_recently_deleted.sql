ALTER TABLE company_settings
ADD COLUMN deleted_retention_days INTEGER NOT NULL DEFAULT 30
CHECK (deleted_retention_days BETWEEN 1 AND 365);

ALTER TABLE categories ADD COLUMN deleted_at TEXT NULL;
ALTER TABLE products ADD COLUMN deleted_at TEXT NULL;
ALTER TABLE suppliers ADD COLUMN deleted_at TEXT NULL;
ALTER TABLE customers ADD COLUMN deleted_at TEXT NULL;
ALTER TABLE purchase_invoices ADD COLUMN deleted_at TEXT NULL;
ALTER TABLE sales_invoices ADD COLUMN deleted_at TEXT NULL;
ALTER TABLE payments ADD COLUMN deleted_at TEXT NULL;
ALTER TABLE payments
ADD COLUMN cancelled_by_invoice INTEGER NOT NULL DEFAULT 0
CHECK (cancelled_by_invoice IN (0, 1));
ALTER TABLE expenses ADD COLUMN deleted_at TEXT NULL;
ALTER TABLE inventory_transactions ADD COLUMN deleted_at TEXT NULL;
ALTER TABLE supplier_settlement_payments ADD COLUMN deleted_at TEXT NULL;

UPDATE categories
SET deleted_at = updated_at
WHERE is_active = 0 AND deleted_at IS NULL;

UPDATE products
SET deleted_at = updated_at
WHERE is_active = 0 AND deleted_at IS NULL;

UPDATE suppliers
SET deleted_at = updated_at
WHERE is_active = 0 AND deleted_at IS NULL;

UPDATE customers
SET deleted_at = updated_at
WHERE is_active = 0 AND deleted_at IS NULL;

UPDATE purchase_invoices
SET deleted_at = updated_at
WHERE status = 'cancelled' AND deleted_at IS NULL;

UPDATE sales_invoices
SET deleted_at = updated_at
WHERE sales_status IN ('cancelled', 'returned') AND deleted_at IS NULL;

UPDATE payments
SET deleted_at = created_at
WHERE status = 'cancelled' AND deleted_at IS NULL;

UPDATE expenses
SET deleted_at = updated_at
WHERE status = 'cancelled' AND deleted_at IS NULL;

UPDATE inventory_transactions
SET deleted_at = created_at
WHERE status = 'cancelled' AND deleted_at IS NULL;

UPDATE supplier_settlement_payments
SET deleted_at = created_at
WHERE lifecycle_status = 'cancelled' AND deleted_at IS NULL;

CREATE INDEX idx_categories_deleted_at ON categories(deleted_at);
CREATE INDEX idx_products_deleted_at ON products(deleted_at);
CREATE INDEX idx_suppliers_deleted_at ON suppliers(deleted_at);
CREATE INDEX idx_customers_deleted_at ON customers(deleted_at);
CREATE INDEX idx_purchase_invoices_deleted_at ON purchase_invoices(deleted_at);
CREATE INDEX idx_sales_invoices_deleted_at ON sales_invoices(deleted_at);
CREATE INDEX idx_payments_deleted_at ON payments(deleted_at);
CREATE INDEX idx_expenses_deleted_at ON expenses(deleted_at);
CREATE INDEX idx_inventory_transactions_deleted_at ON inventory_transactions(deleted_at);
CREATE INDEX idx_supplier_settlement_deleted_at
ON supplier_settlement_payments(deleted_at);
