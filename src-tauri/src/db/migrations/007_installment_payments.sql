ALTER TABLE expenses ADD COLUMN paid_cents INTEGER NOT NULL DEFAULT 0 CHECK (paid_cents >= 0);
ALTER TABLE expenses ADD COLUMN remaining_cents INTEGER NOT NULL DEFAULT 0 CHECK (remaining_cents >= 0);
ALTER TABLE expenses
ADD COLUMN payment_status TEXT NOT NULL DEFAULT 'unpaid'
CHECK (payment_status IN ('paid', 'partial', 'unpaid'));

UPDATE expenses
SET paid_cents = amount_cents,
    remaining_cents = 0,
    payment_status = 'paid';

CREATE TABLE expense_payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    expense_id INTEGER NOT NULL,
    amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
    currency TEXT NOT NULL,
    payment_method TEXT NOT NULL CHECK (payment_method IN ('cash', 'bank', 'card', 'other')),
    payment_date TEXT NOT NULL,
    notes TEXT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'cancelled')),
    cancelled_by_expense INTEGER NOT NULL DEFAULT 0 CHECK (cancelled_by_expense IN (0, 1)),
    created_by INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    deleted_at TEXT NULL,
    FOREIGN KEY (expense_id) REFERENCES expenses(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

INSERT INTO expense_payments
    (expense_id, amount_cents, currency, payment_method, payment_date, notes,
     status, cancelled_by_expense, created_by, created_at, deleted_at)
SELECT id, amount_cents, currency, payment_method, expense_date,
       'Opening payment migrated from the expense record',
       CASE WHEN status = 'active' THEN 'active' ELSE 'cancelled' END,
       CASE WHEN status = 'active' THEN 0 ELSE 1 END,
       created_by, created_at,
       CASE WHEN status = 'active' THEN NULL ELSE COALESCE(deleted_at, updated_at) END
FROM expenses
WHERE amount_cents > 0;

CREATE INDEX idx_expense_payments_expense
ON expense_payments(expense_id, status, payment_date);

CREATE TABLE walk_in_sales_payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sales_invoice_id INTEGER NOT NULL,
    amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
    currency TEXT NOT NULL,
    payment_method TEXT NOT NULL CHECK (payment_method IN ('cash', 'bank', 'card', 'other')),
    payment_date TEXT NOT NULL,
    notes TEXT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'cancelled')),
    cancelled_by_invoice INTEGER NOT NULL DEFAULT 0 CHECK (cancelled_by_invoice IN (0, 1)),
    created_by INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    deleted_at TEXT NULL,
    FOREIGN KEY (sales_invoice_id) REFERENCES sales_invoices(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

INSERT INTO walk_in_sales_payments
    (sales_invoice_id, amount_cents, currency, payment_method, payment_date, notes,
     status, cancelled_by_invoice, created_by, created_at, deleted_at)
SELECT si.id, si.paid_cents, cs.default_currency, 'cash', si.invoice_date,
       'Opening payment migrated from the sales invoice',
       CASE WHEN si.sales_status = 'completed' THEN 'active' ELSE 'cancelled' END,
       CASE WHEN si.sales_status = 'completed' THEN 0 ELSE 1 END,
       si.created_by, si.created_at,
       CASE WHEN si.sales_status = 'completed' THEN NULL ELSE COALESCE(si.deleted_at, si.updated_at) END
FROM sales_invoices si
CROSS JOIN company_settings cs
WHERE si.customer_id IS NULL
  AND si.paid_cents > 0;

CREATE INDEX idx_walk_in_sales_payments_invoice
ON walk_in_sales_payments(sales_invoice_id, status, payment_date);
