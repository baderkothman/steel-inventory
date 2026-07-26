-- Give every operational ledger a lifecycle state. Cancelled rows remain available
-- for audit/history, but are excluded from every business calculation.
ALTER TABLE inventory_transactions
ADD COLUMN status TEXT NOT NULL DEFAULT 'active'
CHECK (status IN ('active', 'cancelled'));

ALTER TABLE payments
ADD COLUMN status TEXT NOT NULL DEFAULT 'active'
CHECK (status IN ('active', 'cancelled'));

ALTER TABLE expenses
ADD COLUMN status TEXT NOT NULL DEFAULT 'active'
CHECK (status IN ('active', 'cancelled'));

ALTER TABLE supplier_settlement_payments
ADD COLUMN lifecycle_status TEXT NOT NULL DEFAULT 'active'
CHECK (lifecycle_status IN ('active', 'cancelled'));

-- Purchase invoices can change the latest product cost. Track those price snapshots
-- so cancelling a purchase also removes its price effect.
ALTER TABLE product_prices ADD COLUMN reference_type TEXT NULL
CHECK (reference_type IS NULL OR reference_type = 'purchase_invoice');
ALTER TABLE product_prices ADD COLUMN reference_id INTEGER NULL;
ALTER TABLE product_prices
ADD COLUMN status TEXT NOT NULL DEFAULT 'active'
CHECK (status IN ('active', 'cancelled'));

-- Link price snapshots produced by older purchase invoices. Purchase-created prices
-- use the same timestamp as their invoice and line items.
UPDATE product_prices
SET reference_type = 'purchase_invoice',
    reference_id = (
        SELECT pi.id
        FROM purchase_invoice_items pii
        JOIN purchase_invoices pi ON pi.id = pii.purchase_invoice_id
        WHERE pii.product_id = product_prices.product_id
          AND pii.unit_cost_cents = product_prices.cost_price_cents
          AND pi.created_at = product_prices.created_at
        ORDER BY pi.id DESC
        LIMIT 1
    )
WHERE EXISTS (
    SELECT 1
    FROM purchase_invoice_items pii
    JOIN purchase_invoices pi ON pi.id = pii.purchase_invoice_id
    WHERE pii.product_id = product_prices.product_id
      AND pii.unit_cost_cents = product_prices.cost_price_cents
      AND pi.created_at = product_prices.created_at
);

-- Older cancellation code wrote compensating inventory movements. Mark both the
-- original and compensating rows cancelled, leaving a zero operational effect.
UPDATE inventory_transactions
SET status = 'cancelled'
WHERE reference_type = 'purchase_invoice'
  AND EXISTS (
      SELECT 1
      FROM purchase_invoices pi
      WHERE pi.id = inventory_transactions.reference_id
        AND pi.status = 'cancelled'
  );

UPDATE inventory_transactions
SET status = 'cancelled'
WHERE reference_type = 'sales_invoice'
  AND EXISTS (
      SELECT 1
      FROM sales_invoices si
      WHERE si.id = inventory_transactions.reference_id
        AND si.sales_status IN ('cancelled', 'returned')
  );

UPDATE product_prices
SET status = 'cancelled'
WHERE reference_type = 'purchase_invoice'
  AND EXISTS (
      SELECT 1
      FROM purchase_invoices pi
      WHERE pi.id = product_prices.reference_id
        AND pi.status = 'cancelled'
  );

-- Rebuild the stock cache from the active inventory ledger so legacy drift is
-- repaired during migration.
UPDATE stock_levels
SET current_quantity = COALESCE((
        SELECT SUM(it.quantity_in - it.quantity_out)
        FROM inventory_transactions it
        WHERE it.product_id = stock_levels.product_id
          AND it.status = 'active'
    ), 0),
    updated_at = datetime('now');

CREATE INDEX idx_inventory_transactions_status
ON inventory_transactions(status, reference_type, reference_id);
CREATE INDEX idx_payments_status ON payments(status, payment_date);
CREATE INDEX idx_expenses_status ON expenses(status, expense_date);
CREATE INDEX idx_settlement_payments_lifecycle
ON supplier_settlement_payments(lifecycle_status, payment_date);
CREATE INDEX idx_product_prices_status
ON product_prices(product_id, status, effective_from);
