export const paymentMethods = ["cash", "bank", "card", "other"] as const;

export const productTypes = ["pipe", "sheet", "bar", "beam", "equipment", "accessory"] as const;
export const materials = ["steel", "galvanized steel", "stainless steel"] as const;
export const shapes = ["square", "rectangular", "round", "flat", "angle", "channel", "beam"] as const;
export const finishes = ["galvanized", "black", "painted", "stainless"] as const;
export const units = ["piece", "meter", "kg", "sheet"] as const;

export const reportOptions = [
  { value: "daily_sales", label: "Daily sales" },
  { value: "daily_profit", label: "Daily profit" },
  { value: "monthly_profit", label: "Monthly profit" },
  { value: "stock", label: "Stock remaining" },
  { value: "stock_movement", label: "Stock movement" },
  { value: "low_stock", label: "Low stock" },
  { value: "purchase", label: "Purchase report" },
  { value: "supplier_debt", label: "Supplier debt" },
  { value: "customer_debt", label: "Customer debt" },
  { value: "expense", label: "Expense report" },
  { value: "payment", label: "Payment report" },
  { value: "inventory_value", label: "Inventory value" },
  { value: "best_selling", label: "Best-selling products" }
] as const;
