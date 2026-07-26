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
  TableContainer,
  TableHead,
  TableRow,
  TextField,
  Typography
} from "@mui/material";
import AddIcon from "@mui/icons-material/Add";
import DeleteIcon from "@mui/icons-material/Delete";
import PaymentsOutlinedIcon from "@mui/icons-material/PaymentsOutlined";
import PrintIcon from "@mui/icons-material/Print";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { PageHeader } from "../../components/PageHeader";
import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { EmptyState, LoadingState } from "../../components/feedback/PageState";
import { PrintDialog } from "../../components/print/PrintDialog";
import {
  EnterpriseTable,
  RowActionsMenu,
  type TableColumn
} from "../../components/table/EnterpriseTable";
import { StatusBadge } from "../../components/table/StatusBadge";
import {
  customerApi,
  invoicePaymentApi,
  productApi,
  purchaseApi,
  salesApi,
  settingsApi,
  supplierApi
} from "../../lib/api";
import { paymentMethods } from "../../lib/constants";
import { fromCents, money, quantity, toCents, today } from "../../lib/formatters";
import { normalizeError } from "../../lib/tauri";
import type { InvoiceListRow } from "../../types/invoice";
import type { InstallmentPaymentPayload, InstallmentPaymentRow } from "../../types/payment";
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
  const [paymentInvoiceId, setPaymentInvoiceId] = useState<number | null>(null);
  const [printHtml, setPrintHtml] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [cancelError, setCancelError] = useState<string | null>(null);

  const { data: invoices = [], isLoading } = useQuery({ queryKey: [kind, "invoices"], queryFn: () => invoiceApi.list({ active_only: true }) });
  const { data: parties = [] } = useQuery({ queryKey: [kind, "parties"], queryFn: () => partyApi.list({ active_only: true }) });
  const { data: products = [] } = useQuery({ queryKey: ["products", "active"], queryFn: () => productApi.list({ active_only: true }) });
  const { data: settings } = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get });
  const activeInvoices = useMemo(() => invoices.filter((invoice) => invoice.status !== "cancelled"), [invoices]);
  const paymentInvoice = activeInvoices.find((invoice) => invoice.id === paymentInvoiceId) ?? null;
  const columns = useMemo<TableColumn<InvoiceListRow>[]>(() => [
    { id: "invoice", label: "Invoice", value: (row) => row.invoice_number, minWidth: 130 },
    { id: "date", label: "Date", value: (row) => row.invoice_date, width: 110 },
    { id: "party", label: kind === "purchase" ? "Supplier" : "Customer", value: (row) => row.party_name, minWidth: 150 },
    { id: "total", label: "Total", value: (row) => money(row.total_cents, settings?.default_currency), render: (row) => <MoneyText value={row.total_cents} currency={settings?.default_currency} />, align: "right", minWidth: 100 },
    { id: "paid", label: "Paid", value: (row) => money(row.paid_cents, settings?.default_currency), render: (row) => <MoneyText value={row.paid_cents} currency={settings?.default_currency} />, align: "right", minWidth: 100 },
    { id: "remaining", label: "Remaining", value: (row) => money(row.remaining_cents, settings?.default_currency), render: (row) => <MoneyText value={row.remaining_cents} currency={settings?.default_currency} />, align: "right", minWidth: 100 },
    { id: "status", label: "Payment", value: (row) => row.payment_status, render: (row) => <StatusBadge value={row.payment_status} />, width: 105 }
  ], [kind, settings?.default_currency]);

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
      await queryClient.invalidateQueries();
    },
    onError: (err) => setError(normalizeError(err).message)
  });

  const cancelMutation = useMutation({
    mutationFn: (id: number) => invoiceApi.cancel(id),
    onSuccess: async () => {
      setCancelId(null);
      setCancelError(null);
      await queryClient.invalidateQueries();
    },
    onError: (err) => setCancelError(normalizeError(err).message)
  });
  async function printInvoice(id: number) {
    setPrintHtml(await invoiceApi.print(id));
  }

  if (isLoading) return <LoadingState label={`Loading ${kind} invoices`} />;

  const title = kind === "purchase" ? "Purchases" : "Sales Invoices";
  const partyLabel = kind === "purchase" ? "Supplier" : "Customer";

  return (
    <Stack spacing={2}>
      <PageHeader
        title={title}
        description={kind === "purchase" ? "Record supplier invoices and increase stock." : "Create customer invoices and decrease stock."}
        actions={
          <Button startIcon={<AddIcon />} variant="contained" onClick={() => setForm({ ...emptyForm })}>New invoice</Button>
        }
      />
      {cancelError && cancelId === null ? <Alert severity="error">{cancelError}</Alert> : null}

      <EnterpriseTable
        title={title}
        rows={activeInvoices}
        columns={columns}
        rowId={(row) => row.id}
        loading={isLoading}
        emptyTitle={`No active ${kind === "purchase" ? "purchases" : "sales invoices"}`}
        emptyDescription="Create an invoice to begin recording transactions."
        initialSort={{ column: "date", direction: "desc" }}
        actions={(row) => [
          { label: "Print invoice", icon: <PrintIcon fontSize="small" />, onClick: () => void printInvoice(row.id) },
          {
            label: "Payments",
            icon: <PaymentsOutlinedIcon fontSize="small" />,
            onClick: () => setPaymentInvoiceId(row.id)
          },
          { label: "Delete", icon: <DeleteIcon fontSize="small" />, destructive: true, onClick: () => { setCancelError(null); setCancelId(row.id); } }
        ]}
      />

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
        title="Delete invoice"
        message="This removes the invoice from active lists, reverses its stock and payment effects, and preserves its audit history."
        confirmLabel="Delete"
        error={cancelError}
        loading={cancelMutation.isPending}
        onClose={() => {
          setCancelId(null);
          setCancelError(null);
        }}
        onConfirm={() => cancelId && cancelMutation.mutate(cancelId)}
      />
      <InstallmentPaymentDialog
        kind={kind}
        invoice={paymentInvoice}
        currency={settings?.default_currency}
        onClose={() => setPaymentInvoiceId(null)}
      />
      <PrintDialog open={Boolean(printHtml)} html={printHtml} onClose={() => setPrintHtml("")} />
    </Stack>
  );
}

function InstallmentPaymentDialog({
  kind,
  invoice,
  currency,
  onClose
}: {
  kind: Kind;
  invoice: InvoiceListRow | null;
  currency?: string;
  onClose: () => void;
}) {
  const queryClient = useQueryClient();
  const [amount, setAmount] = useState("0.00");
  const [paymentMethod, setPaymentMethod] = useState("cash");
  const [paymentDate, setPaymentDate] = useState(today());
  const [notes, setNotes] = useState("");
  const [error, setError] = useState<string | null>(null);
  const { data: payments = [], isLoading } = useQuery({
    queryKey: [kind, "invoice-payments", invoice?.id],
    queryFn: () => invoicePaymentApi.list(kind, invoice!.id),
    enabled: Boolean(invoice)
  });
  const columns = useMemo<TableColumn<InstallmentPaymentRow>[]>(() => [
    { id: "date", label: "Date", value: (row) => row.payment_date, width: 110 },
    { id: "amount", label: "Paid this time", value: (row) => money(row.amount_cents, row.currency), render: (row) => <MoneyText value={row.amount_cents} currency={row.currency} />, align: "right", minWidth: 130 },
    { id: "method", label: "Method", value: (row) => row.payment_method, width: 100 },
    { id: "notes", label: "Notes", value: (row) => row.notes ?? "", minWidth: 170 }
  ], []);
  const mutation = useMutation({
    mutationFn: (payload: InstallmentPaymentPayload) =>
      invoicePaymentApi.create(kind, invoice!.id, payload),
    onSuccess: async () => {
      setAmount("0.00");
      setNotes("");
      setError(null);
      await queryClient.invalidateQueries();
    },
    onError: (err) => setError(normalizeError(err).message)
  });

  function close() {
    setAmount("0.00");
    setNotes("");
    setError(null);
    onClose();
  }

  function submit(event: FormEvent) {
    event.preventDefault();
    mutation.mutate({
      amount_cents: toCents(amount),
      payment_method: paymentMethod,
      payment_date: paymentDate,
      notes: notes || null
    });
  }

  return (
    <Dialog open={Boolean(invoice)} onClose={close} fullWidth maxWidth="md">
      <DialogTitle>Payments — {invoice?.invoice_number}</DialogTitle>
      <DialogContent>
        <Stack spacing={2} sx={{ pt: 1 }}>
          <Box
            sx={{
              display: "grid",
              gridTemplateColumns: { xs: "1fr", sm: "repeat(3, minmax(0, 1fr))" },
              gap: 1.5
            }}
          >
            <PaymentSummary label="Invoice total" value={invoice?.total_cents ?? 0} currency={currency} />
            <PaymentSummary label="Paid to date" value={invoice?.paid_cents ?? 0} currency={currency} />
            <PaymentSummary label="Remaining" value={invoice?.remaining_cents ?? 0} currency={currency} />
          </Box>

          <EnterpriseTable
            title={`Payment history — ${invoice?.invoice_number ?? ""}`}
            rows={payments}
            columns={columns}
            rowId={(row) => row.id}
            loading={isLoading}
            searchable={false}
            selectable={false}
            compact
            maxHeight={260}
            emptyTitle="No payments recorded"
            emptyDescription="Record the first payment below."
          />

          {invoice && invoice.remaining_cents > 0 ? (
            <Stack component="form" id="invoice-payment-form" onSubmit={submit} spacing={2}>
              {error ? <Alert severity="error">{error}</Alert> : null}
              <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", sm: "repeat(3, 1fr)" }, gap: 2 }}>
                <TextField
                  label="Amount paid this time"
                  type="number"
                  required
                  value={amount}
                  onChange={(event) => setAmount(event.target.value)}
                  slotProps={{ htmlInput: { min: 0.01, max: fromCents(invoice.remaining_cents), step: "0.01" } }}
                  helperText={`Maximum ${money(invoice.remaining_cents, currency)}`}
                />
                <TextField select label="Payment method" value={paymentMethod} onChange={(event) => setPaymentMethod(event.target.value)}>
                  {paymentMethods.map((method) => <MenuItem key={method} value={method}>{method}</MenuItem>)}
                </TextField>
                <TextField label="Payment date" type="date" value={paymentDate} onChange={(event) => setPaymentDate(event.target.value)} />
              </Box>
              <TextField label="Notes" value={notes} onChange={(event) => setNotes(event.target.value)} />
            </Stack>
          ) : (
            <Alert severity="success">This invoice is fully paid.</Alert>
          )}
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={close}>Close</Button>
        {invoice && invoice.remaining_cents > 0 ? (
          <Button type="submit" form="invoice-payment-form" variant="contained" disabled={mutation.isPending}>
            Record payment
          </Button>
        ) : null}
      </DialogActions>
    </Dialog>
  );
}

function PaymentSummary({ label, value, currency }: { label: string; value: number; currency?: string }) {
  return (
    <Box sx={{ borderBottom: "1px solid", borderColor: "divider", pb: 1 }}>
      <Typography variant="body2" color="text.secondary">{label}</Typography>
      <Typography variant="h6" sx={{ mt: 0.25, fontVariantNumeric: "tabular-nums" }}>
        <MoneyText value={value} currency={currency} />
      </Typography>
    </Box>
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
              {products.map((product) => <MenuItem key={product.id} value={product.id}>{product.supplier_name} — {product.name} ({product.sku})</MenuItem>)}
            </TextField>
          </Box>

          <Paper variant="outlined">
            {form?.items.length ? (
              <TableContainer sx={{ maxHeight: 360 }}>
              <Table size="small" stickyHeader aria-label="Invoice line items">
                <TableHead><TableRow><TableCell>Supplier</TableCell><TableCell>Product</TableCell><TableCell align="right">Available</TableCell><TableCell align="right">Quantity</TableCell><TableCell align="right">Unit {kind === "purchase" ? "cost" : "price"}</TableCell><TableCell align="right">Row total</TableCell><TableCell /></TableRow></TableHead>
                <TableBody>{form.items.map((item, index) => {
                  const product = products.find((candidate) => candidate.id === item.product_id);
                  const rowTotal = Number(item.quantity || 0) * Number(item.unit_price || 0);
                  return (
                    <TableRow key={`${item.product_id}-${index}`}>
                      <TableCell>{product?.supplier_name}</TableCell>
                      <TableCell>{product?.sku} - {product?.name}</TableCell>
                      <TableCell align="right">{kind === "sales" ? quantity(product?.current_quantity) : "-"}</TableCell>
                      <TableCell align="right"><TextField type="number" value={item.quantity} onChange={(e) => updateItem(index, { ...item, quantity: e.target.value })} sx={{ width: 96 }} /></TableCell>
                      <TableCell align="right"><TextField value={item.unit_price} onChange={(e) => updateItem(index, { ...item, unit_price: e.target.value })} sx={{ width: 120 }} /></TableCell>
                      <TableCell align="right">{rowTotal.toFixed(2)}</TableCell>
                      <TableCell align="right">
                        <RowActionsMenu
                          row={item}
                          actions={[{
                            label: "Remove item",
                            icon: <DeleteIcon fontSize="small" />,
                            destructive: true,
                            onClick: () => onChange(form && { ...form, items: form.items.filter((_, i) => i !== index) })
                          }]}
                        />
                      </TableCell>
                    </TableRow>
                  );
                })}</TableBody>
              </Table>
              </TableContainer>
            ) : <EmptyState label="Add at least one product." />}
          </Paper>

          <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", md: "repeat(4, 1fr)" }, gap: 2 }}>
            <TextField label="Discount" value={form?.discount ?? "0.00"} onChange={(e) => onChange(form && { ...form, discount: e.target.value })} />
            <TextField label="Tax" value={form?.tax ?? "0.00"} onChange={(e) => onChange(form && { ...form, tax: e.target.value })} />
            <TextField label={extraLabel} value={form?.extra ?? "0.00"} onChange={(e) => onChange(form && { ...form, extra: e.target.value })} />
            <TextField
              label="Amount paid now"
              type="number"
              value={form?.paid ?? "0.00"}
              onChange={(e) => onChange(form && { ...form, paid: e.target.value })}
              slotProps={{ htmlInput: { min: 0, max: Math.max(totals.total, 0), step: "0.01" } }}
              helperText="The remaining balance stays open"
            />
          </Box>
          <TextField label="Notes" multiline minRows={2} value={form?.notes ?? ""} onChange={(e) => onChange(form && { ...form, notes: e.target.value })} />
          <Box
            sx={{
              display: "grid",
              gridTemplateColumns: { xs: "1fr", sm: "repeat(3, minmax(0, 180px))" },
              justifyContent: "end",
              gap: 2
            }}
          >
            <PaymentSummary label="Subtotal" value={toCents(totals.subtotal.toFixed(2))} />
            <PaymentSummary label="Invoice total" value={toCents(totals.total.toFixed(2))} />
            <PaymentSummary
              label="Remaining"
              value={toCents(Math.max(totals.total - Number(form?.paid || 0), 0).toFixed(2))}
            />
          </Box>
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
