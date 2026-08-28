ALTER TABLE company_settings
ADD COLUMN quotation_prefix TEXT NOT NULL DEFAULT 'QT';

ALTER TABLE company_settings
ADD COLUMN logo_path TEXT NULL;

ALTER TABLE suppliers
ADD COLUMN logo_path TEXT NULL;

CREATE TABLE quotations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NULL REFERENCES customers(id) ON DELETE SET NULL,
    quotation_number TEXT NOT NULL UNIQUE,
    quotation_date TEXT NOT NULL,
    valid_until TEXT NOT NULL,
    customer_name TEXT NOT NULL,
    customer_company_name TEXT,
    customer_phone TEXT,
    customer_email TEXT,
    customer_address TEXT,
    customer_tax_number TEXT,
    subtotal_cents INTEGER NOT NULL CHECK (subtotal_cents >= 0),
    discount_cents INTEGER NOT NULL DEFAULT 0 CHECK (discount_cents >= 0),
    tax_cents INTEGER NOT NULL DEFAULT 0 CHECK (tax_cents >= 0),
    total_cents INTEGER NOT NULL CHECK (total_cents >= 0),
    notes TEXT,
    terms TEXT,
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'sent', 'accepted', 'rejected', 'expired', 'converted')),
    converted_sales_invoice_id INTEGER NULL UNIQUE
        REFERENCES sales_invoices(id) ON DELETE RESTRICT,
    created_by INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE quotation_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    quotation_id INTEGER NOT NULL REFERENCES quotations(id) ON DELETE CASCADE,
    product_id INTEGER NULL REFERENCES products(id) ON DELETE SET NULL,
    sku TEXT NOT NULL,
    product_name TEXT NOT NULL,
    quantity REAL NOT NULL CHECK (quantity > 0),
    unit_price_cents INTEGER NOT NULL CHECK (unit_price_cents >= 0),
    line_total_cents INTEGER NOT NULL CHECK (line_total_cents >= 0),
    created_at TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_quotations_number ON quotations(quotation_number);
CREATE INDEX idx_quotations_customer ON quotations(customer_id);
CREATE INDEX idx_quotations_date ON quotations(quotation_date);
CREATE INDEX idx_quotations_valid_until ON quotations(valid_until);
CREATE INDEX idx_quotations_status ON quotations(status);
CREATE INDEX idx_quotation_items_quotation ON quotation_items(quotation_id);
CREATE INDEX idx_quotation_items_product ON quotation_items(product_id);
