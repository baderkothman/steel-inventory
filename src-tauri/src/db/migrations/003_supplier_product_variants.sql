-- Supplier-specific product variants, spec grouping, storage location,
-- and supplier settlement payments (period payables paid against sold goods).

-- 1. Extend products with supplier ownership, a shared-specification key, and storage location.
ALTER TABLE products ADD COLUMN supplier_id INTEGER NULL REFERENCES suppliers(id) ON DELETE RESTRICT;
ALTER TABLE products ADD COLUMN spec_key TEXT NOT NULL DEFAULT '';
ALTER TABLE products ADD COLUMN location TEXT NULL;

-- 2. Fallback supplier so existing supplier-less products remain valid and reportable.
--    Looked up by name only here; no supplier name is hard-coded anywhere in application code.
INSERT INTO suppliers (name, company_name, opening_balance_cents, notes, is_active, created_at, updated_at)
SELECT 'Unknown Supplier', NULL, 0, 'Auto-created for products imported before supplier tracking.', 1,
       datetime('now'), datetime('now')
WHERE NOT EXISTS (SELECT 1 FROM suppliers WHERE name = 'Unknown Supplier');

-- 3. Backfill existing products to the fallback supplier.
UPDATE products
SET supplier_id = (SELECT id FROM suppliers WHERE name = 'Unknown Supplier' ORDER BY id LIMIT 1)
WHERE supplier_id IS NULL;

-- 4. Backfill spec_key from the steel specification (shared across suppliers, supplier-independent).
--    Mirrors utils::sku::spec_key_from_parts: TYPE|MATERIAL|SHAPE|FINISH|SIZE|THICKNESS.
UPDATE products
SET spec_key = upper(
        trim(product_type) || '|' || trim(material) || '|' || trim(shape) || '|' ||
        trim(finish) || '|' || COALESCE(trim(size_label), '') || '|' ||
        COALESCE(CAST(thickness_mm AS TEXT), '')
    )
WHERE spec_key = '';

-- 5. Supplier settlement payments: money paid to a supplier for a settlement period
--    (driven by sales of that supplier's goods). Status + reference/notes per payment.
CREATE TABLE supplier_settlement_payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    supplier_id INTEGER NOT NULL REFERENCES suppliers(id) ON DELETE RESTRICT,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
    currency TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'paid' CHECK (status IN ('unpaid', 'partial', 'paid')),
    payment_date TEXT NOT NULL,
    reference TEXT,
    notes TEXT,
    created_by INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_products_supplier_id ON products(supplier_id);
CREATE INDEX idx_products_spec_key ON products(spec_key);
CREATE INDEX idx_settlement_payments_supplier ON supplier_settlement_payments(supplier_id);
CREATE INDEX idx_settlement_payments_period ON supplier_settlement_payments(period_start, period_end);
