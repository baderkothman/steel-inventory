-- Demo records were created only by the retired Dashboard seeder. Remove the
-- complete seeded graph in dependency order while leaving reference categories,
-- company settings, and administrator accounts intact.

CREATE TEMP TABLE demo_products_to_remove (
    id INTEGER PRIMARY KEY
);

-- Preserve a demo-prefixed product if it was later used on a real, non-demo
-- invoice. This avoids altering user-created invoice history during an upgrade.
INSERT INTO demo_products_to_remove (id)
SELECT p.id
FROM products p
WHERE p.sku LIKE 'DEMO-%'
  AND NOT EXISTS (
      SELECT 1
      FROM purchase_invoice_items pii
      JOIN purchase_invoices pi ON pi.id = pii.purchase_invoice_id
      WHERE pii.product_id = p.id
        AND pi.invoice_number NOT LIKE 'PI-DEMO-%'
  )
  AND NOT EXISTS (
      SELECT 1
      FROM sales_invoice_items sii
      JOIN sales_invoices si ON si.id = sii.sales_invoice_id
      WHERE sii.product_id = p.id
        AND si.invoice_number NOT LIKE 'SI-DEMO-%'
  );

DELETE FROM audit_logs
WHERE table_name = 'demo_seed'
   OR (table_name = 'products' AND record_id IN (
       SELECT id FROM demo_products_to_remove
   ))
   OR (table_name = 'suppliers' AND record_id IN (
       SELECT id FROM suppliers
       WHERE name IN ('Demo Metal Supply', 'Demo Tools Import', 'Demo Transport Supplier')
   ))
   OR (table_name = 'customers' AND record_id IN (
       SELECT id FROM customers
       WHERE name IN ('Demo Builders', 'Demo Workshop', 'Demo Cash Customer')
   ))
   OR (table_name = 'purchase_invoices' AND record_id IN (
       SELECT id FROM purchase_invoices WHERE invoice_number LIKE 'PI-DEMO-%'
   ))
   OR (table_name = 'sales_invoices' AND record_id IN (
       SELECT id FROM sales_invoices WHERE invoice_number LIKE 'SI-DEMO-%'
   ))
   OR (table_name = 'expenses' AND record_id IN (
       SELECT id FROM expenses
       WHERE title IN (
           'Demo shop rent',
           'Demo electricity bill',
           'Demo local delivery',
           'Demo machine maintenance',
           'Demo packaging material'
       )
   ));

DELETE FROM expense_payments
WHERE expense_id IN (
    SELECT id FROM expenses
    WHERE title IN (
        'Demo shop rent',
        'Demo electricity bill',
        'Demo local delivery',
        'Demo machine maintenance',
        'Demo packaging material'
    )
);

DELETE FROM walk_in_sales_payments
WHERE sales_invoice_id IN (
    SELECT id FROM sales_invoices WHERE invoice_number LIKE 'SI-DEMO-%'
);

DELETE FROM payments
WHERE (reference_type = 'purchase_invoice' AND reference_id IN (
           SELECT id FROM purchase_invoices WHERE invoice_number LIKE 'PI-DEMO-%'
       ))
       OR (reference_type = 'sales_invoice' AND reference_id IN (
           SELECT id FROM sales_invoices WHERE invoice_number LIKE 'SI-DEMO-%'
       ))
   OR notes IN (
       'Extra payment against old balance.',
       'General payment to supplier.'
   )
   OR notes LIKE 'Demo payment with PI-DEMO-%'
   OR notes LIKE 'Demo payment with SI-DEMO-%';

DELETE FROM inventory_transactions
WHERE product_id IN (
          SELECT id FROM demo_products_to_remove
      )
   OR (reference_type = 'purchase_invoice' AND reference_id IN (
          SELECT id FROM purchase_invoices WHERE invoice_number LIKE 'PI-DEMO-%'
      ))
   OR (reference_type = 'sales_invoice' AND reference_id IN (
          SELECT id FROM sales_invoices WHERE invoice_number LIKE 'SI-DEMO-%'
      ));

DELETE FROM purchase_invoice_items
WHERE purchase_invoice_id IN (
          SELECT id FROM purchase_invoices WHERE invoice_number LIKE 'PI-DEMO-%'
      );

DELETE FROM sales_invoice_items
WHERE sales_invoice_id IN (
          SELECT id FROM sales_invoices WHERE invoice_number LIKE 'SI-DEMO-%'
      );

DELETE FROM stock_levels
WHERE product_id IN (
    SELECT id FROM demo_products_to_remove
);

DELETE FROM product_prices
WHERE product_id IN (
    SELECT id FROM demo_products_to_remove
);

DELETE FROM expenses
WHERE title IN (
    'Demo shop rent',
    'Demo electricity bill',
    'Demo local delivery',
    'Demo machine maintenance',
    'Demo packaging material'
);

DELETE FROM sales_invoices
WHERE invoice_number LIKE 'SI-DEMO-%';

DELETE FROM purchase_invoices
WHERE invoice_number LIKE 'PI-DEMO-%';

DELETE FROM products
WHERE id IN (SELECT id FROM demo_products_to_remove);

DELETE FROM customers
WHERE name IN ('Demo Builders', 'Demo Workshop', 'Demo Cash Customer')
  AND NOT EXISTS (
      SELECT 1 FROM sales_invoices WHERE customer_id = customers.id
  )
  AND NOT EXISTS (
      SELECT 1
      FROM payments
      WHERE party_type = 'customer' AND party_id = customers.id
  );

DELETE FROM suppliers
WHERE name IN ('Demo Metal Supply', 'Demo Tools Import', 'Demo Transport Supplier')
  AND NOT EXISTS (
      SELECT 1 FROM purchase_invoices WHERE supplier_id = suppliers.id
  )
  AND NOT EXISTS (
      SELECT 1
      FROM payments
      WHERE party_type = 'supplier' AND party_id = suppliers.id
  )
  AND NOT EXISTS (
      SELECT 1
      FROM supplier_settlement_payments
      WHERE supplier_id = suppliers.id
  );

DELETE FROM backups
WHERE notes = 'Demo backup log row only; no file is created by demo seed.'
   OR backup_path LIKE '%steel_inventory_backup_demo.db';

DROP TABLE demo_products_to_remove;
