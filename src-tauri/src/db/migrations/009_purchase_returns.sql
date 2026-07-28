ALTER TABLE purchase_invoices
ADD COLUMN returned_cents INTEGER NOT NULL DEFAULT 0
CHECK (returned_cents >= 0);

CREATE TABLE purchase_returns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    purchase_invoice_id INTEGER NOT NULL REFERENCES purchase_invoices(id) ON DELETE RESTRICT,
    supplier_id INTEGER NOT NULL REFERENCES suppliers(id) ON DELETE RESTRICT,
    return_number TEXT NOT NULL UNIQUE,
    return_date TEXT NOT NULL,
    reason TEXT NULL,
    notes TEXT NULL,
    subtotal_cents INTEGER NOT NULL CHECK (subtotal_cents >= 0),
    discount_cents INTEGER NOT NULL DEFAULT 0 CHECK (discount_cents >= 0),
    tax_cents INTEGER NOT NULL DEFAULT 0 CHECK (tax_cents >= 0),
    shipping_cents INTEGER NOT NULL DEFAULT 0 CHECK (shipping_cents >= 0),
    total_cents INTEGER NOT NULL CHECK (total_cents >= 0),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'cancelled')),
    idempotency_key TEXT NOT NULL UNIQUE CHECK (length(trim(idempotency_key)) >= 8),
    request_payload_json TEXT NOT NULL,
    current_revision INTEGER NOT NULL DEFAULT 1 CHECK (current_revision > 0),
    created_by INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    cancelled_by INTEGER NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    cancelled_at TEXT NULL
);

CREATE TABLE purchase_return_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    purchase_return_id INTEGER NOT NULL REFERENCES purchase_returns(id) ON DELETE RESTRICT,
    purchase_invoice_item_id INTEGER NOT NULL REFERENCES purchase_invoice_items(id) ON DELETE RESTRICT,
    product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    quantity REAL NOT NULL CHECK (quantity > 0),
    unit_cost_cents INTEGER NOT NULL CHECK (unit_cost_cents >= 0),
    total_cost_cents INTEGER NOT NULL CHECK (total_cost_cents >= 0),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision > 0),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'superseded')),
    created_at TEXT NOT NULL,
    superseded_at TEXT NULL,
    UNIQUE (purchase_return_id, purchase_invoice_item_id, revision)
);

ALTER TABLE inventory_transactions
ADD COLUMN purchase_return_id INTEGER NULL REFERENCES purchase_returns(id) ON DELETE RESTRICT;

ALTER TABLE inventory_transactions
ADD COLUMN purchase_return_item_id INTEGER NULL REFERENCES purchase_return_items(id) ON DELETE RESTRICT;

CREATE UNIQUE INDEX idx_inventory_purchase_return_item
ON inventory_transactions(purchase_return_item_id)
WHERE purchase_return_item_id IS NOT NULL;

CREATE INDEX idx_purchase_returns_invoice_status
ON purchase_returns(purchase_invoice_id, status, return_date);

CREATE INDEX idx_purchase_returns_supplier_status
ON purchase_returns(supplier_id, status, return_date);

CREATE INDEX idx_purchase_return_items_invoice_item
ON purchase_return_items(purchase_invoice_item_id, status, purchase_return_id);

CREATE TRIGGER validate_purchase_return_supplier_before_insert
BEFORE INSERT ON purchase_returns
BEGIN
    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1
            FROM purchase_invoices pi
            WHERE pi.id = NEW.purchase_invoice_id
              AND pi.supplier_id = NEW.supplier_id
        )
        THEN RAISE(ABORT, 'purchase return supplier does not match invoice')
    END;
END;

CREATE TRIGGER validate_purchase_return_item_before_insert
BEFORE INSERT ON purchase_return_items
BEGIN
    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1
            FROM purchase_invoice_items pii
            JOIN purchase_returns pr
              ON pr.id = NEW.purchase_return_id
             AND pr.purchase_invoice_id = pii.purchase_invoice_id
            WHERE pii.id = NEW.purchase_invoice_item_id
              AND pii.product_id = NEW.product_id
              AND pii.unit_cost_cents = NEW.unit_cost_cents
        )
        THEN RAISE(ABORT, 'purchase return item does not match original invoice')
    END;
END;
