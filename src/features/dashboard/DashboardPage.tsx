import { Box, Card, CardContent, Grid, Stack, Typography } from "@mui/material";
import { useQuery } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { PageHeader } from "../../components/PageHeader";
import { LoadingState } from "../../components/feedback/PageState";
import { PrintButton } from "../../components/print/PrintButton";
import { EnterpriseTable, type TableColumn } from "../../components/table/EnterpriseTable";
import { StatusBadge } from "../../components/table/StatusBadge";
import { reportApi } from "../../lib/api";
import { money, quantity } from "../../lib/formatters";
import type { InvoiceListRow } from "../../types/invoice";
import type { Product } from "../../types/product";

const cards = [
  ["today_sales_count", "Today's sales count"],
  ["today_purchase_count", "Today's purchase count"],
  ["today_sales_cents", "Today's sales"],
  ["today_profit_cents", "Today's profit"],
  ["today_expenses_cents", "Today's expenses"],
  ["net_profit_cents", "Net profit"],
  ["total_customer_debts_cents", "Customer debts"],
  ["total_supplier_debts_cents", "Supplier debts"],
  ["current_stock_value_cents", "Current stock value"],
  ["low_stock_count", "Low-stock products"]
] as const;

const countCards = new Set(["today_sales_count", "today_purchase_count", "low_stock_count"]);
const invoiceColumns: TableColumn<InvoiceListRow>[] = [
  { id: "invoice", label: "Invoice", value: (row) => row.invoice_number, minWidth: 120 },
  { id: "date", label: "Date", value: (row) => row.invoice_date, width: 110 },
  { id: "party", label: "Party", value: (row) => row.party_name, minWidth: 150 },
  { id: "total", label: "Total", value: (row) => money(row.total_cents), render: (row) => <MoneyText value={row.total_cents} />, align: "right", minWidth: 110 },
  { id: "status", label: "Status", value: (row) => row.payment_status, render: (row) => <StatusBadge value={row.payment_status} />, width: 110 }
];
const lowStockColumns: TableColumn<Product>[] = [
  { id: "sku", label: "SKU", value: (row) => row.sku, minWidth: 130 },
  { id: "product", label: "Product", value: (row) => row.name, minWidth: 180 },
  { id: "current", label: "Current", value: (row) => quantity(row.current_quantity), align: "right", width: 100 },
  { id: "minimum", label: "Minimum", value: (row) => quantity(row.minimum_quantity), align: "right", width: 100 }
];

export function DashboardPage() {
  const { data, isLoading } = useQuery({
    queryKey: ["dashboard"],
    queryFn: () => reportApi.dashboard()
  });

  if (isLoading || !data) {
    return <LoadingState label="Loading dashboard" />;
  }

  return (
    <Stack spacing={3}>
      <PageHeader
        title="Dashboard"
        description="Daily sales, profit, debts, and stock alerts."
        actions={<PrintButton targetId="dashboard-print" title="Inventory Dashboard" />}
      />

      <Stack id="dashboard-print" spacing={2}>
        <Grid container spacing={2}>
          {cards.map(([key, label]) => (
            <Grid key={key} size={{ xs: 12, sm: 6, md: 3 }}>
              <Card variant="outlined">
                <CardContent>
                  <Typography variant="body2" color="text.secondary">
                    {label}
                  </Typography>
                  <Typography variant="h5" sx={{ mt: 1, fontVariantNumeric: "tabular-nums" }}>
                    {countCards.has(key) ? data[key] : <MoneyText value={data[key]} />}
                  </Typography>
                </CardContent>
              </Card>
            </Grid>
          ))}
        </Grid>

      <Grid container spacing={2}>
        <Grid size={{ xs: 12, md: 6 }} sx={{ display: "flex" }}>
          <Stack spacing={1} sx={{ width: "100%", minWidth: 0 }}>
            <Typography variant="h6">Recent Sales Invoices</Typography>
            <Box sx={{ flex: 1, minHeight: 0 }}>
              <EnterpriseTable
                title="Recent Sales Invoices"
                rows={data.recent_sales_invoices}
                columns={invoiceColumns}
                rowId={(row) => row.id}
                compact
                pageSize={10}
                fillHeight
                emptyTitle="No recent sales invoices"
                emptyDescription="Active sales invoices will appear here."
              />
            </Box>
          </Stack>
        </Grid>
        <Grid size={{ xs: 12, md: 6 }} sx={{ display: "flex" }}>
          <Stack spacing={1} sx={{ width: "100%", minWidth: 0 }}>
            <Typography variant="h6">Recent Purchase Invoices</Typography>
            <Box sx={{ flex: 1, minHeight: 0 }}>
              <EnterpriseTable
                title="Recent Purchase Invoices"
                rows={data.recent_purchase_invoices}
                columns={invoiceColumns}
                rowId={(row) => row.id}
                compact
                pageSize={10}
                fillHeight
                emptyTitle="No recent purchase invoices"
                emptyDescription="Active purchase invoices will appear here."
              />
            </Box>
          </Stack>
        </Grid>
      </Grid>

        <Box>
          <EnterpriseTable
            title="Low-stock Products"
            rows={data.low_stock_products}
            columns={lowStockColumns}
            rowId={(row) => row.id}
            compact
            emptyTitle="Stock levels are healthy"
            emptyDescription="No products are currently at or below minimum stock."
          />
        </Box>
      </Stack>
    </Stack>
  );
}
