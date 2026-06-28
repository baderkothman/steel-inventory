import { FormEvent, useMemo, useState } from "react";
import {
  Alert,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  MenuItem,
  Paper,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  TextField,
  Typography
} from "@mui/material";
import AddIcon from "@mui/icons-material/Add";
import CancelIcon from "@mui/icons-material/Cancel";
import PrintIcon from "@mui/icons-material/Print";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { PageHeader } from "../../components/PageHeader";
import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { EmptyState, LoadingState } from "../../components/feedback/PageState";
import { PrintDialog } from "../../components/print/PrintDialog";
import { customerApi, productApi, purchaseApi, salesApi, supplierApi } from "../../lib/api";
import { fromCents, quantity, toCents, today } from "../../lib/formatters";
import { normalizeError } from "../../lib/tauri";
import type { InvoiceListRow } from "../../types/invoice";
import type { Product } from "../../types/product";

type Kind = "purchase" | "sales";
type InvoiceItemForm = {
  product_id: number;
  quantity: string;
  unit_price: string;
};
type InvoiceForm = {
  party_id: number | "";
  invoice_number: string;
  invoice_date: string;
  discount: string;
  tax: string;
  extra: string;
  paid: string;
  notes: string;
  items: InvoiceItemForm[];
};

const emptyForm: InvoiceForm = {
  party_id: "",
  invoice_number: "",
  invoice_date: today(),
  discount: "0.00",
  tax: "0.00",
  extra: "0.00",
  paid: "0.00",
  notes: "",
  items: []
};

export function PurchasesPage() {
  return <InvoicePage kind="purchase" />;
}

export function SalesPage() {
  return <InvoicePage kind="sales" />;
}

function InvoicePage({ kind }: { kind: Kind }) {
  const queryClient = useQueryClient();
  const invoiceApi = kind === "purchase" ? purchaseApi : salesApi;
  const partyApi = kind === "purchase" ? supplierApi : customerApi;
  const [form, setForm] = useState<InvoiceForm | null>(null);
  const [cancelId, setCancelId] = useState<number | null>(null);
  const [printHtml, setPrintHtml] = useState("");
  const [error, setError] = useState<string | null>(null);

  const { data: invoices = [], isLoading } = useQuery({ queryKey: [kind, "invoices"], queryFn: () => invoiceApi.list() });
  const { data: parties = [] } = useQuery({ queryKey: [kind, "parties"], queryFn: () => partyApi.list({ active_only: true }) });
  const { data: products = [] } = useQuery({ queryKey: ["products", "active"], queryFn: () => productApi.list({ active_only: true }) });

  const saveMutation = useMutation({
    mutationFn: (value: InvoiceForm) => {
      if (kind === "purchase") {
        return purchaseApi.create({
          supplier_id: Number(value.party_id),
          invoice_number: value.invoice_number || null,
          invoice_date: value.invoice_date,
          discount_cents: toCents(value.discount),
          tax_cents: toCents(value.tax),
          shipping_cents: toCents(value.extra),
          paid_cents: toCents(value.paid),
          notes: value.notes || null,
          items: value.items.map((item) => ({
            product_id: item.product_id,
            quantity: Number(item.quantity),
            unit_cost_cents: toCents(item.unit_price)
          }))
        });
      }
      return salesApi.create({
        customer_id: value.party_id === "" ? null : Number(value.party_id),
        invoice_number: value.invoice_number || null,
        invoice_date: value.invoice_date,
        discount_cents: toCents(value.discount),
        tax_cents: toCents(value.tax),
        delivery_cents: toCents(value.extra),
        paid_cents: toCents(value.paid),
        notes: value.notes || null,
        items: value.items.map((item) => ({
          product_id: item.product_id,
          quantity: Number(item.quantity),
          unit_price_cents: toCents(item.unit_price)
        }))
      });
    },
    onSuccess: async () => {
      setForm(null);
      setError(null);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: [kind, "invoices"] }),
        queryClient.invalidateQueries({ queryKey: ["products"] }),
        queryClient.invalidateQueries({ queryKey: ["dashboard"] })
      ]);
    },
    onError: (err) => setError(normalizeError(err).message)
  });

  const cancelMutation = useMutation({
    mutationFn: (id: number) => invoiceApi.cancel(id),
    onSuccess: async () => {
      setCancelId(null);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: [kind, "invoices"] }),
        queryClient.invalidateQueries({ queryKey: ["products"] }),
        queryClient.invalidateQueries({ queryKey: ["dashboard"] })
      ]);
    }
  });

  async function printInvoice(id: number) {
    setPrintHtml(await invoiceApi.print(id));
  }

  if (isLoading) return <LoadingState label={`Loading ${kind} invoices`} />;

  const title = kind === "purchase" ? "Purchases / Stock In" : "Sales Invoices";
  const partyLabel = kind === "purchase" ? "Supplier" : "Customer";

  return (
    <Stack spacing={2}>
      <PageHeader
        title={title}
        description={kind === "purchase" ? "Record supplier invoices and increase stock." : "Create customer invoices and decrease stock."}
        actions={<Button startIcon={<AddIcon />} variant="contained" onClick={() => setForm({ ...emptyForm })}>New invoice</Button>}
      />

      <Paper variant="outlined">
        {invoices.length === 0 ? <EmptyState label="No invoices recorded yet." /> : (
          <Table size="small">
            <TableHead><TableRow><TableCell>Invoice</TableCell><TableCell>Date</TableCell><TableCell>{partyLabel}</TableCell><TableCell align="right">Total</TableCell><TableCell align="right">Paid</TableCell><TableCell align="right">Remaining</TableCell><TableCell>Status</TableCell><TableCell align="right">Actions</TableCell></TableRow></TableHead>
            <TableBody>{invoices.map((invoice) => <InvoiceRow key={invoice.id} invoice={invoice} onPrint={printInvoice} onCancel={setCancelId} />)}</TableBody>
          </Table>
        )}
      </Paper>

      <InvoiceDialog
        kind={kind}
        form={form}
        parties={parties}
        products={products}
        error={error}
        saving={saveMutation.isPending}
        onClose={() => setForm(null)}
        onChange={setForm}
        onSubmit={(event) => {
          event.preventDefault();
          if (form) saveMutation.mutate(form);
        }}
      />

      <ConfirmDialog
        open={cancelId !== null}
        title="Cancel invoice"
        message="Cancelling an invoice reverses its stock movement and removes its linked invoice payment."
        confirmLabel="Cancel invoice"
        onClose={() => setCancelId(null)}
        onConfirm={() => cancelId && cancelMutation.mutate(cancelId)}
      />
      <PrintDialog open={Boolean(printHtml)} html={printHtml} onClose={() => setPrintHtml("")} />
    </Stack>
  );
}

function InvoiceRow({ invoice, onPrint, onCancel }: { invoice: InvoiceListRow; onPrint: (id: number) => void; onCancel: (id: number) => void }) {
  return (
    <TableRow hover>
      <TableCell>{invoice.invoice_number}</TableCell>
      <TableCell>{invoice.invoice_date}</TableCell>
      <TableCell>{invoice.party_name}</TableCell>
      <TableCell align="right"><MoneyText value={invoice.total_cents} /></TableCell>
      <TableCell align="right"><MoneyText value={invoice.paid_cents} /></TableCell>
      <TableCell align="right"><MoneyText value={invoice.remaining_cents} /></TableCell>
      <TableCell>{invoice.status} / {invoice.payment_status}</TableCell>
      <TableCell align="right">
        <Button size="small" startIcon={<PrintIcon />} onClick={() => onPrint(invoice.id)}>Print</Button>
        <Button size="small" color="error" startIcon={<CancelIcon />} disabled={invoice.status === "cancelled"} onClick={() => onCancel(invoice.id)}>Cancel</Button>
      </TableCell>
    </TableRow>
  );
}

function InvoiceDialog({
  kind,
  form,
  parties,
  products,
  error,
  saving,
  onClose,
  onChange,
  onSubmit
}: {
  kind: Kind;
  form: InvoiceForm | null;
  parties: Array<{ id: number; name: string }>;
  products: Product[];
  error: string | null;
  saving: boolean;
  onClose: () => void;
  onChange: (form: InvoiceForm | null) => void;
  onSubmit: (event: FormEvent) => void;
}) {
  const totals = useMemo(() => calculateTotals(form), [form]);
  const partyLabel = kind === "purchase" ? "Supplier" : "Customer";
  const extraLabel = kind === "purchase" ? "Shipping" : "Delivery";

  function updateItem(index: number, item: InvoiceItemForm) {
    if (!form) return;
    const items = [...form.items];
    items[index] = item;
    onChange({ ...form, items });
  }

  function addItem(product: Product | undefined) {
    if (!form || !product) return;
    onChange({
      ...form,
      items: [
        ...form.items,
        {
          product_id: product.id,
          quantity: "1",
          unit_price: fromCents(kind === "purchase" ? product.cost_price_cents : product.selling_price_cents)
        }
      ]
    });
  }

  return (
    <Dialog open={Boolean(form)} onClose={onClose} fullWidth maxWidth="lg">
      <DialogTitle>{kind === "purchase" ? "New purchase invoice" : "New sales invoice"}</DialogTitle>
      <DialogContent>
        <Stack component="form" id={`${kind}-invoice-form`} onSubmit={onSubmit} spacing={2} sx={{ pt: 1 }}>
          {error ? <Alert severity="error">{error}</Alert> : null}
          <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", md: "repeat(4, 1fr)" }, gap: 2 }}>
            <TextField select label={partyLabel} required={kind === "purchase"} value={form?.party_id ?? ""} onChange={(e) => onChange(form && { ...form, party_id: e.target.value ? Number(e.target.value) : "" })}>
              {kind === "sales" ? <MenuItem value="">Walk-in Customer</MenuItem> : null}
              {parties.map((party) => <MenuItem key={party.id} value={party.id}>{party.name}</MenuItem>)}
            </TextField>
            <TextField label="Invoice number" value={form?.invoice_number ?? ""} onChange={(e) => onChange(form && { ...form, invoice_number: e.target.value })} helperText="Leave blank for automatic numbering" />
            <TextField label="Invoice date" type="date" value={form?.invoice_date ?? today()} onChange={(e) => onChange(form && { ...form, invoice_date: e.target.value })} />
            <TextField select label="Add product" value="" onChange={(e) => addItem(products.find((product) => product.id === Number(e.target.value)))}>
              {products.map((product) => <MenuItem key={product.id} value={product.id}>{product.sku} - {product.name}</MenuItem>)}
            </TextField>
          </Box>

          <Paper variant="outlined">
            {form?.items.length ? (
              <Table size="small">
                <TableHead><TableRow><TableCell>Product</TableCell><TableCell align="right">Available</TableCell><TableCell align="right">Quantity</TableCell><TableCell align="right">Unit {kind === "purchase" ? "cost" : "price"}</TableCell><TableCell align="right">Row total</TableCell><TableCell /></TableRow></TableHead>
                <TableBody>{form.items.map((item, index) => {
                  const product = products.find((candidate) => candidate.id === item.product_id);
                  const rowTotal = Number(item.quantity || 0) * Number(item.unit_price || 0);
                  return (
                    <TableRow key={`${item.product_id}-${index}`}>
                      <TableCell>{product?.sku} - {product?.name}</TableCell>
                      <TableCell align="right">{kind === "sales" ? quantity(product?.current_quantity) : "-"}</TableCell>
                      <TableCell align="right"><TextField type="number" value={item.quantity} onChange={(e) => updateItem(index, { ...item, quantity: e.target.value })} sx={{ width: 96 }} /></TableCell>
                      <TableCell align="right"><TextField value={item.unit_price} onChange={(e) => updateItem(index, { ...item, unit_price: e.target.value })} sx={{ width: 120 }} /></TableCell>
                      <TableCell align="right">{rowTotal.toFixed(2)}</TableCell>
                      <TableCell align="right"><Button size="small" color="error" onClick={() => onChange(form && { ...form, items: form.items.filter((_, i) => i !== index) })}>Remove</Button></TableCell>
                    </TableRow>
                  );
                })}</TableBody>
              </Table>
            ) : <EmptyState label="Add at least one product." />}
          </Paper>

          <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", md: "repeat(4, 1fr)" }, gap: 2 }}>
            <TextField label="Discount" value={form?.discount ?? "0.00"} onChange={(e) => onChange(form && { ...form, discount: e.target.value })} />
            <TextField label="Tax" value={form?.tax ?? "0.00"} onChange={(e) => onChange(form && { ...form, tax: e.target.value })} />
            <TextField label={extraLabel} value={form?.extra ?? "0.00"} onChange={(e) => onChange(form && { ...form, extra: e.target.value })} />
            <TextField label="Paid amount" value={form?.paid ?? "0.00"} onChange={(e) => onChange(form && { ...form, paid: e.target.value })} />
          </Box>
          <TextField label="Notes" multiline minRows={2} value={form?.notes ?? ""} onChange={(e) => onChange(form && { ...form, notes: e.target.value })} />
          <Stack direction="row" spacing={3} justifyContent="flex-end">
            <Typography>Subtotal: <strong>{totals.subtotal.toFixed(2)}</strong></Typography>
            <Typography>Total: <strong>{totals.total.toFixed(2)}</strong></Typography>
            <Typography>Remaining: <strong>{Math.max(totals.total - Number(form?.paid || 0), 0).toFixed(2)}</strong></Typography>
          </Stack>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button type="submit" form={`${kind}-invoice-form`} variant="contained" disabled={saving}>Save invoice</Button>
      </DialogActions>
    </Dialog>
  );
}

function calculateTotals(form: InvoiceForm | null) {
  const subtotal = form?.items.reduce((sum, item) => sum + Number(item.quantity || 0) * Number(item.unit_price || 0), 0) ?? 0;
  const total = subtotal - Number(form?.discount || 0) + Number(form?.tax || 0) + Number(form?.extra || 0);
  return { subtotal, total };
}
