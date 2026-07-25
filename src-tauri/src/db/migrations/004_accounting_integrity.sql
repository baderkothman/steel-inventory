-- Remove legacy polymorphic payment rows that no longer point to a real party
-- or to the active invoice they claim to settle.
DELETE FROM payments
WHERE (party_type = 'customer' AND NOT EXISTS (
           SELECT 1 FROM customers c WHERE c.id = payments.party_id
       ))
   OR (party_type = 'supplier' AND NOT EXISTS (
           SELECT 1 FROM suppliers s WHERE s.id = payments.party_id
       ))
   OR (party_type = 'customer' AND payment_direction <> 'in')
   OR (party_type = 'supplier' AND payment_direction <> 'out')
   OR (reference_type IS NULL AND reference_id IS NOT NULL)
   OR (reference_type = 'sales_invoice' AND NOT EXISTS (
           SELECT 1
           FROM sales_invoices si
           WHERE si.id = payments.reference_id
             AND si.customer_id = payments.party_id
             AND payments.party_type = 'customer'
             AND si.sales_status = 'completed'
       ))
   OR (reference_type = 'purchase_invoice' AND NOT EXISTS (
           SELECT 1
           FROM purchase_invoices pi
           WHERE pi.id = payments.reference_id
             AND pi.supplier_id = payments.party_id
             AND payments.party_type = 'supplier'
             AND pi.status = 'active'
       ));

-- Bring invoice summary fields back in line with the surviving accounting rows.
-- Walk-in sales have no customer payment ledger, so their recorded cash values
-- intentionally remain untouched.
UPDATE sales_invoices
SET paid_cents = MIN(total_cents, COALESCE((
        SELECT SUM(p.amount_cents)
        FROM payments p
        WHERE p.reference_type = 'sales_invoice'
          AND p.reference_id = sales_invoices.id
          AND p.party_type = 'customer'
          AND p.party_id = sales_invoices.customer_id
    ), 0)),
    remaining_cents = total_cents - MIN(total_cents, COALESCE((
        SELECT SUM(p.amount_cents)
        FROM payments p
        WHERE p.reference_type = 'sales_invoice'
          AND p.reference_id = sales_invoices.id
          AND p.party_type = 'customer'
          AND p.party_id = sales_invoices.customer_id
    ), 0)),
    payment_status = CASE
        WHEN COALESCE((
            SELECT SUM(p.amount_cents)
            FROM payments p
            WHERE p.reference_type = 'sales_invoice'
              AND p.reference_id = sales_invoices.id
              AND p.party_type = 'customer'
              AND p.party_id = sales_invoices.customer_id
        ), 0) <= 0 THEN 'unpaid'
        WHEN COALESCE((
            SELECT SUM(p.amount_cents)
            FROM payments p
            WHERE p.reference_type = 'sales_invoice'
              AND p.reference_id = sales_invoices.id
              AND p.party_type = 'customer'
              AND p.party_id = sales_invoices.customer_id
        ), 0) >= total_cents THEN 'paid'
        ELSE 'partial'
    END
WHERE customer_id IS NOT NULL
  AND sales_status = 'completed';

UPDATE purchase_invoices
SET paid_cents = MIN(total_cents, COALESCE((
        SELECT SUM(p.amount_cents)
        FROM payments p
        WHERE p.reference_type = 'purchase_invoice'
          AND p.reference_id = purchase_invoices.id
          AND p.party_type = 'supplier'
          AND p.party_id = purchase_invoices.supplier_id
    ), 0)),
    remaining_cents = total_cents - MIN(total_cents, COALESCE((
        SELECT SUM(p.amount_cents)
        FROM payments p
        WHERE p.reference_type = 'purchase_invoice'
          AND p.reference_id = purchase_invoices.id
          AND p.party_type = 'supplier'
          AND p.party_id = purchase_invoices.supplier_id
    ), 0)),
    payment_status = CASE
        WHEN COALESCE((
            SELECT SUM(p.amount_cents)
            FROM payments p
            WHERE p.reference_type = 'purchase_invoice'
              AND p.reference_id = purchase_invoices.id
              AND p.party_type = 'supplier'
              AND p.party_id = purchase_invoices.supplier_id
        ), 0) <= 0 THEN 'unpaid'
        WHEN COALESCE((
            SELECT SUM(p.amount_cents)
            FROM payments p
            WHERE p.reference_type = 'purchase_invoice'
              AND p.reference_id = purchase_invoices.id
              AND p.party_type = 'supplier'
              AND p.party_id = purchase_invoices.supplier_id
        ), 0) >= total_cents THEN 'paid'
        ELSE 'partial'
    END
WHERE status = 'active';

-- payments.party_id is polymorphic, so ordinary foreign keys cannot express
-- these relationships. Triggers keep future imports and direct writes honest.
CREATE TRIGGER validate_payment_integrity_before_insert
BEFORE INSERT ON payments
BEGIN
    SELECT CASE
        WHEN NEW.party_type = 'customer'
         AND NOT EXISTS (SELECT 1 FROM customers WHERE id = NEW.party_id)
        THEN RAISE(ABORT, 'payment customer not found')
        WHEN NEW.party_type = 'supplier'
         AND NOT EXISTS (SELECT 1 FROM suppliers WHERE id = NEW.party_id)
        THEN RAISE(ABORT, 'payment supplier not found')
        WHEN NEW.party_type = 'customer' AND NEW.payment_direction <> 'in'
        THEN RAISE(ABORT, 'invalid customer payment direction')
        WHEN NEW.party_type = 'supplier' AND NEW.payment_direction <> 'out'
        THEN RAISE(ABORT, 'invalid supplier payment direction')
        WHEN NEW.reference_type IS NULL AND NEW.reference_id IS NOT NULL
        THEN RAISE(ABORT, 'payment reference id requires a reference type')
        WHEN NEW.reference_type = 'sales_invoice'
         AND NOT EXISTS (
             SELECT 1 FROM sales_invoices
             WHERE id = NEW.reference_id
               AND customer_id = NEW.party_id
               AND NEW.party_type = 'customer'
               AND sales_status = 'completed'
         )
        THEN RAISE(ABORT, 'invalid sales invoice payment reference')
        WHEN NEW.reference_type = 'purchase_invoice'
         AND NOT EXISTS (
             SELECT 1 FROM purchase_invoices
             WHERE id = NEW.reference_id
               AND supplier_id = NEW.party_id
               AND NEW.party_type = 'supplier'
               AND status = 'active'
         )
        THEN RAISE(ABORT, 'invalid purchase invoice payment reference')
    END;
END;

CREATE TRIGGER validate_payment_integrity_before_update
BEFORE UPDATE OF party_type, party_id, payment_direction, reference_type, reference_id ON payments
BEGIN
    SELECT CASE
        WHEN NEW.party_type = 'customer'
         AND NOT EXISTS (SELECT 1 FROM customers WHERE id = NEW.party_id)
        THEN RAISE(ABORT, 'payment customer not found')
        WHEN NEW.party_type = 'supplier'
         AND NOT EXISTS (SELECT 1 FROM suppliers WHERE id = NEW.party_id)
        THEN RAISE(ABORT, 'payment supplier not found')
        WHEN NEW.party_type = 'customer' AND NEW.payment_direction <> 'in'
        THEN RAISE(ABORT, 'invalid customer payment direction')
        WHEN NEW.party_type = 'supplier' AND NEW.payment_direction <> 'out'
        THEN RAISE(ABORT, 'invalid supplier payment direction')
        WHEN NEW.reference_type IS NULL AND NEW.reference_id IS NOT NULL
        THEN RAISE(ABORT, 'payment reference id requires a reference type')
        WHEN NEW.reference_type = 'sales_invoice'
         AND NOT EXISTS (
             SELECT 1 FROM sales_invoices
             WHERE id = NEW.reference_id
               AND customer_id = NEW.party_id
               AND NEW.party_type = 'customer'
               AND sales_status = 'completed'
         )
        THEN RAISE(ABORT, 'invalid sales invoice payment reference')
        WHEN NEW.reference_type = 'purchase_invoice'
         AND NOT EXISTS (
             SELECT 1 FROM purchase_invoices
             WHERE id = NEW.reference_id
               AND supplier_id = NEW.party_id
               AND NEW.party_type = 'supplier'
               AND status = 'active'
         )
        THEN RAISE(ABORT, 'invalid purchase invoice payment reference')
    END;
END;
