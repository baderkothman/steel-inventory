import { useState } from "react";
import { Alert, Button, Card, CardContent, Grid, Snackbar, Stack, Table, TableBody, TableCell, TableHead, TableRow, Typography } from "@mui/material";
import DataObjectIcon from "@mui/icons-material/DataObject";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { PageHeader } from "../../components/PageHeader";
import { EmptyState, LoadingState } from "../../components/feedback/PageState";
import { reportApi, seedApi } from "../../lib/api";
import { quantity } from "../../lib/formatters";

const cards = [
  ["today_sales_cents", "Today's sales"],
  ["today_profit_cents", "Today's profit"],
  ["today_expenses_cents", "Today's expenses"],
  ["net_profit_cents", "Net profit"],
  ["total_customer_debts_cents", "Customer debts"],
  ["total_supplier_debts_cents", "Supplier debts"],
  ["current_stock_value_cents", "Current stock value"],
  ["low_stock_count", "Low-stock products"]
] as const;

export function DashboardPage() {
  const queryClient = useQueryClient();
  const [toast, setToast] = useState<string | null>(null);
  const { data, isLoading } = useQuery({
    queryKey: ["dashboard"],
    queryFn: () => reportApi.dashboard()
  });
  const seedMutation = useMutation({
    mutationFn: seedApi.demoData,
    onSuccess: async (result) => {
      setToast(result.message);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["dashboard"] }),
        queryClient.invalidateQueries({ queryKey: ["products"] }),
        queryClient.invalidateQueries({ queryKey: ["categories"] }),
        queryClient.invalidateQueries({ queryKey: ["supplier"] }),
        queryClient.invalidateQueries({ queryKey: ["customer"] }),
        queryClient.invalidateQueries({ queryKey: ["purchase"] }),
        queryClient.invalidateQueries({ queryKey: ["sales"] }),
        queryClient.invalidateQueries({ queryKey: ["expenses"] }),
        queryClient.invalidateQueries({ queryKey: ["payments"] }),
        queryClient.invalidateQueries({ queryKey: ["backups"] })
      ]);
    },
    onError: (error) => setToast(error instanceof Error ? error.message : "Could not seed demo data.")
  });

  if (isLoading || !data) {
    return <LoadingState label="Loading dashboard" />;
  }

  return (
    <Stack spacing={3}>
      <PageHeader title="Dashboard" description="Daily sales, profit, debts, and stock alerts." />
      <Alert
        severity="info"
        action={
          <Button
            color="inherit"
            size="small"
            startIcon={<DataObjectIcon />}
            disabled={seedMutation.isPending}
            onClick={() => seedMutation.mutate()}
          >
            Seed demo data
          </Button>
        }
      >
        Populate demo rows across products, parties, purchases, sales, expenses, payments, reports, and backup logs.
      </Alert>

      <Grid container spacing={2}>
        {cards.map(([key, label]) => (
          <Grid key={key} size={{ xs: 12, sm: 6, md: 3 }}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="body2" color="text.secondary">
                  {label}
                </Typography>
                <Typography variant="h5" sx={{ mt: 1, fontVariantNumeric: "tabular-nums" }}>
                  {key === "low_stock_count" ? data.low_stock_count : <MoneyText value={data[key]} />}
                </Typography>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>

      <Grid container spacing={2}>
        <Grid size={{ xs: 12, md: 6 }}>
          <Card variant="outlined">
            <CardContent>
              <Typography variant="h6" sx={{ mb: 1.5 }}>
                Recent sales invoices
              </Typography>
              <InvoiceTable rows={data.recent_sales_invoices} />
            </CardContent>
          </Card>
        </Grid>
        <Grid size={{ xs: 12, md: 6 }}>
          <Card variant="outlined">
            <CardContent>
              <Typography variant="h6" sx={{ mb: 1.5 }}>
                Recent purchase invoices
              </Typography>
              <InvoiceTable rows={data.recent_purchase_invoices} />
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      <Card variant="outlined">
        <CardContent>
          <Typography variant="h6" sx={{ mb: 1.5 }}>
            Low-stock products
          </Typography>
          {data.low_stock_products.length === 0 ? (
            <EmptyState label="No products are currently at or below minimum stock." />
          ) : (
            <Table size="small">
              <TableHead>
                <TableRow>
                  <TableCell>SKU</TableCell>
                  <TableCell>Product</TableCell>
                  <TableCell align="right">Current</TableCell>
                  <TableCell align="right">Minimum</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {data.low_stock_products.map((product) => (
                  <TableRow key={product.id}>
                    <TableCell>{product.sku}</TableCell>
                    <TableCell>{product.name}</TableCell>
                    <TableCell align="right">{quantity(product.current_quantity)}</TableCell>
                    <TableCell align="right">{quantity(product.minimum_quantity)}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
      <Snackbar open={Boolean(toast)} autoHideDuration={5000} onClose={() => setToast(null)} message={toast} />
    </Stack>
  );
}

function InvoiceTable({ rows }: { rows: Array<{ id: number; invoice_number: string; invoice_date: string; party_name: string; total_cents: number; payment_status: string }> }) {
  if (rows.length === 0) {
    return <EmptyState label="No invoices recorded yet." />;
  }

  return (
    <Table size="small">
      <TableHead>
        <TableRow>
          <TableCell>Invoice</TableCell>
          <TableCell>Date</TableCell>
          <TableCell>Party</TableCell>
          <TableCell align="right">Total</TableCell>
          <TableCell>Status</TableCell>
        </TableRow>
      </TableHead>
      <TableBody>
        {rows.map((row) => (
          <TableRow key={row.id}>
            <TableCell>{row.invoice_number}</TableCell>
            <TableCell>{row.invoice_date}</TableCell>
            <TableCell>{row.party_name}</TableCell>
            <TableCell align="right">
              <MoneyText value={row.total_cents} />
            </TableCell>
            <TableCell>{row.payment_status}</TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
