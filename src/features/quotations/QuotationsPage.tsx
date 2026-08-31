import { FormEvent, useMemo, useState } from "react";
import {
  Alert,
  Autocomplete,
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
import CheckCircleOutlineIcon from "@mui/icons-material/CheckCircleOutline";
import DeleteIcon from "@mui/icons-material/Delete";
import EditIcon from "@mui/icons-material/Edit";
import PrintIcon from "@mui/icons-material/Print";
import SendOutlinedIcon from "@mui/icons-material/SendOutlined";
import ShoppingCartCheckoutIcon from "@mui/icons-material/ShoppingCartCheckout";
import ThumbDownOutlinedIcon from "@mui/icons-material/ThumbDownOutlined";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { PageHeader } from "../../components/PageHeader";
import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { EmptyState, LoadingState } from "../../components/feedback/PageState";
import { PrintDialog } from "../../components/print/PrintDialog";
import {
  EnterpriseTable,
  RowActionsMenu,
  type RowAction,
  type TableColumn
} from "../../components/table/EnterpriseTable";
import { StatusBadge } from "../../components/table/StatusBadge";
import { customerApi, productApi, quotationApi, settingsApi } from "../../lib/api";
import { fromCents, money, quantity, toCents, today } from "../../lib/formatters";
import { normalizeError } from "../../lib/tauri";
import type { Party } from "../../types/party";
import type { Product } from "../../types/product";
import type {
  QuotationDetail,
  QuotationListRow,
  QuotationPayload,
  QuotationStatus
} from "../../types/quotation";

type QuotationLineForm = {
  line_id: string;
  product_id: number;
  quantity: string;
  unit_price: string;
};

type QuotationForm = {
  id?: number;
  customer_id: number | "";
  quotation_number: string;
  quotation_date: string;
  valid_until: string;
  discount: string;
  tax: string;
  notes: string;
  items: QuotationLineForm[];
};

type PendingAction = {
  id: number;
  status?: QuotationStatus;
  kind: "status" | "delete";
  title: string;
  message: string;
  confirmLabel: string;
};

type ConversionForm = {
  quotation: QuotationListRow;
  invoice_number: string;
  invoice_date: string;
  delivery: string;
  paid: string;
};

const statusOptions: Array<{ value: "all" | QuotationStatus; label: string }> = [
  { value: "all", label: "All statuses" },
  { value: "draft", label: "Draft" },
  { value: "sent", label: "Sent" },
  { value: "accepted", label: "Accepted" },
  { value: "rejected", label: "Rejected" },
  { value: "expired", label: "Expired" },
  { value: "converted", label: "Converted" }
];

export function QuotationsPage() {
  const queryClient = useQueryClient();
  const [statusFilter, setStatusFilter] = useState<"all" | QuotationStatus>("all");
  const [form, setForm] = useState<QuotationForm | null>(null);
  const [pendingAction, setPendingAction] = useState<PendingAction | null>(null);
  const [conversion, setConversion] = useState<ConversionForm | null>(null);
  const [printHtml, setPrintHtml] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [customerDialogOpen, setCustomerDialogOpen] = useState(false);

  const { data: quotations = [], isLoading } = useQuery({
    queryKey: ["quotations", statusFilter],
    queryFn: () => quotationApi.list({ status: statusFilter === "all" ? null : statusFilter })
  });
  const { data: customers = [] } = useQuery({
    queryKey: ["customer", "quotation-options"],
    queryFn: () => customerApi.list({ active_only: true })
  });
  const { data: products = [] } = useQuery({
    queryKey: ["products", "active"],
    queryFn: () => productApi.list({ active_only: true })
  });
  const { data: settings } = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get });

  const columns = useMemo<TableColumn<QuotationListRow>[]>(() => [
    { id: "number", label: "Pro forma", value: (row) => row.quotation_number, minWidth: 140 },
    { id: "date", label: "Date", value: (row) => row.quotation_date, width: 110 },
    { id: "valid", label: "Valid until", value: (row) => row.valid_until, width: 110 },
    { id: "customer", label: "Customer", value: (row) => row.customer_name, minWidth: 170 },
    {
      id: "total",
      label: "Quoted total",
      value: (row) => money(row.total_cents, settings?.default_currency),
      render: (row) => <MoneyText value={row.total_cents} currency={settings?.default_currency} />,
      align: "right",
      minWidth: 130
    },
    {
      id: "status",
      label: "Status",
      value: (row) => row.status,
      render: (row) => <StatusBadge value={row.status} />,
      width: 110
    }
  ], [settings?.default_currency]);

  const saveMutation = useMutation({
    mutationFn: (value: QuotationForm) => {
      const payload = formToPayload(value);
      return value.id ? quotationApi.update(value.id, payload) : quotationApi.create(payload);
    },
    onSuccess: async () => {
      setForm(null);
      setError(null);
      await queryClient.invalidateQueries({ queryKey: ["quotations"] });
    },
    onError: (reason) => setError(normalizeError(reason).message)
  });

  const actionMutation = useMutation({
    mutationFn: async (action: PendingAction) => {
      if (action.kind === "delete") await quotationApi.delete(action.id);
      else await quotationApi.changeStatus(action.id, action.status!);
    },
    onSuccess: async () => {
      setPendingAction(null);
      setActionError(null);
      await queryClient.invalidateQueries({ queryKey: ["quotations"] });
    },
    onError: (reason) => setActionError(normalizeError(reason).message)
  });

  const conversionMutation = useMutation({
    mutationFn: (value: ConversionForm) => quotationApi.convert(value.quotation.id, {
      invoice_number: value.invoice_number || null,
      invoice_date: value.invoice_date,
      delivery_cents: toCents(value.delivery),
      paid_cents: toCents(value.paid)
    }),
    onSuccess: async (invoice) => {
      setConversion(null);
      setActionError(null);
      await queryClient.invalidateQueries();
      setSuccess(`Pro forma converted to sales invoice ${invoice.invoice_number}.`);
    },
    onError: (reason) => setActionError(normalizeError(reason).message)
  });

  async function editQuotation(id: number) {
    try {
      const detail = await quotationApi.get(id);
      setForm(detailToForm(detail));
      setError(null);
    } catch (reason) {
      setActionError(normalizeError(reason).message);
    }
  }

  async function printQuotation(id: number) {
    try {
      setPrintHtml(await quotationApi.print(id));
      setActionError(null);
    } catch (reason) {
      setActionError(normalizeError(reason).message);
    }
  }

  if (isLoading) return <LoadingState label="Loading pro forma invoices" />;

  return (
    <Stack spacing={2}>
      <PageHeader
        title="Pro Forma"
        description="Prepare pro forma invoices without reserving stock, recording revenue, or creating a customer order."
        actions={
          <Stack direction="row" spacing={1}>
            <TextField
              select
              aria-label="Filter pro forma invoices by status"
              value={statusFilter}
              onChange={(event) => setStatusFilter(event.target.value as typeof statusFilter)}
              sx={{ minWidth: 150 }}
            >
              {statusOptions.map((option) => (
                <MenuItem key={option.value} value={option.value}>{option.label}</MenuItem>
              ))}
            </TextField>
            <Button startIcon={<AddIcon />} variant="contained" onClick={() => setForm(newQuotationForm())}>
              New pro forma
            </Button>
          </Stack>
        }
      />
      {actionError && !pendingAction && !conversion ? <Alert severity="error">{actionError}</Alert> : null}
      {success ? <Alert severity="success" onClose={() => setSuccess(null)}>{success}</Alert> : null}

      <EnterpriseTable
        title="Customer pro forma invoices"
        rows={quotations}
        columns={columns}
        rowId={(row) => row.id}
        emptyTitle="No pro forma invoices"
        emptyDescription="Create a pro forma invoice when a customer wants pricing before placing an order."
        initialSort={{ column: "date", direction: "desc" }}
        actions={(row) => quotationActions(row, {
          edit: editQuotation,
          print: printQuotation,
          setAction: (action) => {
            setActionError(null);
            setPendingAction(action);
          },
          convert: () => {
            setActionError(null);
            setConversion({
              quotation: row,
              invoice_number: "",
              invoice_date: today(),
              delivery: "0.00",
              paid: "0.00"
            });
          }
        })}
      />

      <QuotationDialog
        form={form}
        customers={customers}
        products={products}
        currency={settings?.default_currency}
        error={error}
        saving={saveMutation.isPending}
        onChange={setForm}
        onClose={() => {
          setForm(null);
          setError(null);
        }}
        onSubmit={(event) => {
          event.preventDefault();
          if (form) saveMutation.mutate(form);
        }}
        onCreateCustomer={() => setCustomerDialogOpen(true)}
      />

      <QuickCustomerDialog
        open={customerDialogOpen}
        onClose={() => setCustomerDialogOpen(false)}
        onCreated={(customer) => {
          setCustomerDialogOpen(false);
          setForm((current) => current ? { ...current, customer_id: customer.id } : current);
        }}
      />

      <ConfirmDialog
        open={Boolean(pendingAction)}
        title={pendingAction?.title ?? "Confirm action"}
        message={pendingAction?.message ?? ""}
        confirmLabel={pendingAction?.confirmLabel ?? "Confirm"}
        error={actionError}
        loading={actionMutation.isPending}
        onClose={() => {
          setPendingAction(null);
          setActionError(null);
        }}
        onConfirm={() => pendingAction && actionMutation.mutate(pendingAction)}
      />

      <ConversionDialog
        form={conversion}
        error={actionError}
        saving={conversionMutation.isPending}
        onChange={setConversion}
        onClose={() => {
          setConversion(null);
          setActionError(null);
        }}
        onSubmit={(event) => {
          event.preventDefault();
          if (conversion) conversionMutation.mutate(conversion);
        }}
      />

      <PrintDialog open={Boolean(printHtml)} html={printHtml} onClose={() => setPrintHtml("")} />
    </Stack>
  );
}

function QuotationDialog({
  form,
  customers,
  products,
  currency,
  error,
  saving,
  onChange,
  onClose,
  onSubmit,
  onCreateCustomer
}: {
  form: QuotationForm | null;
  customers: Party[];
  products: Product[];
  currency?: string;
  error: string | null;
  saving: boolean;
  onChange: (form: QuotationForm | null) => void;
  onClose: () => void;
  onSubmit: (event: FormEvent) => void;
  onCreateCustomer: () => void;
}) {
  const totals = quotationTotals(form);
  const selectedCustomer = customers.find((customer) => customer.id === form?.customer_id) ?? null;

  function addProduct(product: Product | null) {
    if (!form || !product) return;
    onChange({
      ...form,
      items: [...form.items, {
        line_id: newLineId(),
        product_id: product.id,
        quantity: "1",
        unit_price: fromCents(product.selling_price_cents)
      }]
    });
  }

  function updateLine(index: number, line: QuotationLineForm) {
    if (!form) return;
    const items = [...form.items];
    items[index] = line;
    onChange({ ...form, items });
  }

  return (
    <Dialog open={Boolean(form)} onClose={onClose} fullWidth maxWidth="lg">
      <DialogTitle>{form?.id ? "Edit draft pro forma" : "New pro forma"}</DialogTitle>
      <DialogContent>
        <Stack component="form" id="quotation-form" onSubmit={onSubmit} spacing={2} sx={{ pt: 1 }}>
          <Alert severity="info">
            Saving a pro forma does not reserve or reduce stock and does not appear in sales or accounting totals.
          </Alert>
          {error ? <Alert severity="error">{error}</Alert> : null}
          <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", md: "2fr 1fr 1fr" }, gap: 2 }}>
            <Stack direction="row" spacing={1} alignItems="flex-start">
              <Autocomplete
                fullWidth
                options={customers}
                value={selectedCustomer}
                getOptionLabel={(customer) => [customer.name, customer.company_name].filter(Boolean).join(" — ")}
                isOptionEqualToValue={(option, value) => option.id === value.id}
                onChange={(_, customer) => onChange(form && { ...form, customer_id: customer?.id ?? "" })}
                renderInput={(params) => <TextField {...params} label="Customer" required />}
              />
              <Button variant="outlined" onClick={onCreateCustomer} sx={{ whiteSpace: "nowrap" }}>New customer</Button>
            </Stack>
            <TextField
              label="Pro forma date"
              type="date"
              required
              value={form?.quotation_date ?? today()}
              onChange={(event) => onChange(form && { ...form, quotation_date: event.target.value })}
            />
            <TextField
              label="Valid until"
              type="date"
              required
              value={form?.valid_until ?? today()}
              onChange={(event) => onChange(form && { ...form, valid_until: event.target.value })}
            />
          </Box>
          <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", md: "1fr 2fr" }, gap: 2 }}>
            <TextField
              label="Pro forma number"
              value={form?.quotation_number ?? ""}
              helperText="Leave blank for automatic numbering"
              onChange={(event) => onChange(form && { ...form, quotation_number: event.target.value })}
            />
            <Autocomplete
              options={products}
              value={null}
              getOptionLabel={(product) => `${product.sku} — ${product.name}`}
              onChange={(_, product) => addProduct(product)}
              renderInput={(params) => (
                <TextField {...params} label="Add product" placeholder="Search by product name, SKU, or barcode" />
              )}
            />
          </Box>

          <Paper variant="outlined">
            {form?.items.length ? (
              <TableContainer sx={{ maxHeight: 380 }}>
                <Table size="small" stickyHeader aria-label="Pro forma line items">
                  <TableHead><TableRow><TableCell>Product</TableCell><TableCell align="right">Available</TableCell><TableCell align="right">Quantity</TableCell><TableCell align="right">Unit price</TableCell><TableCell align="right">Line total</TableCell><TableCell /></TableRow></TableHead>
                  <TableBody>
                    {form.items.map((line, index) => {
                      const product = products.find((candidate) => candidate.id === line.product_id);
                      const lineTotal = Number(line.quantity || 0) * Number(line.unit_price || 0);
                      return (
                        <TableRow key={line.line_id}>
                          <TableCell sx={{ minWidth: 260 }}>
                            <Typography variant="body2" fontWeight={700} sx={{ overflowWrap: "anywhere" }}>{product?.name ?? "Unavailable product"}</Typography>
                            <Typography variant="caption" color="text.secondary">{product?.sku ?? "Historical item"}</Typography>
                          </TableCell>
                          <TableCell align="right">{quantity(product?.current_quantity)}</TableCell>
                          <TableCell align="right">
                            <TextField
                              type="number"
                              required
                              value={line.quantity}
                              onChange={(event) => updateLine(index, { ...line, quantity: event.target.value })}
                              sx={{ width: 100 }}
                              slotProps={{ htmlInput: { min: 0.001, step: 0.001 } }}
                            />
                          </TableCell>
                          <TableCell align="right">
                            <TextField
                              type="number"
                              required
                              value={line.unit_price}
                              onChange={(event) => updateLine(index, { ...line, unit_price: event.target.value })}
                              sx={{ width: 130 }}
                              slotProps={{ htmlInput: { min: 0, step: 0.01 } }}
                            />
                          </TableCell>
                          <TableCell align="right">{money(toCents(lineTotal.toFixed(2)), currency)}</TableCell>
                          <TableCell align="right">
                            <RowActionsMenu
                              row={line}
                              actions={[{
                                label: "Remove item",
                                icon: <DeleteIcon fontSize="small" />,
                                destructive: true,
                                onClick: () => onChange(form && { ...form, items: form.items.filter((_, itemIndex) => itemIndex !== index) })
                              }]}
                            />
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </TableContainer>
            ) : <EmptyState label="Search for a product to add the first pro forma line." />}
          </Paper>

          <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", md: "1fr 1fr 1.2fr" }, gap: 2 }}>
            <TextField
              label="Discount"
              type="number"
              value={form?.discount ?? "0.00"}
              onChange={(event) => onChange(form && { ...form, discount: event.target.value })}
              slotProps={{ htmlInput: { min: 0, step: 0.01 } }}
            />
            <TextField
              label="Tax / VAT"
              type="number"
              value={form?.tax ?? "0.00"}
              onChange={(event) => onChange(form && { ...form, tax: event.target.value })}
              slotProps={{ htmlInput: { min: 0, step: 0.01 } }}
            />
            <Paper variant="outlined" sx={{ px: 2, py: 1.25, bgcolor: "#f5f8f8" }}>
              <Typography variant="caption" color="text.secondary">Final quoted total</Typography>
              <Typography variant="h5" sx={{ mt: 0.25, fontVariantNumeric: "tabular-nums" }}>
                {money(toCents(Math.max(totals.total, 0).toFixed(2)), currency)}
              </Typography>
              <Typography variant="caption" color="text.secondary">Subtotal {money(toCents(totals.subtotal.toFixed(2)), currency)}</Typography>
            </Paper>
          </Box>
          <TextField label="Notes" multiline minRows={2} value={form?.notes ?? ""} onChange={(event) => onChange(form && { ...form, notes: event.target.value })} />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button type="submit" form="quotation-form" variant="contained" disabled={saving || !form?.items.length || !form?.customer_id}>
          Save pro forma
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function QuickCustomerDialog({ open, onClose, onCreated }: {
  open: boolean;
  onClose: () => void;
  onCreated: (customer: Party) => void;
}) {
  const queryClient = useQueryClient();
  const [name, setName] = useState("");
  const [company, setCompany] = useState("");
  const [phone, setPhone] = useState("");
  const [email, setEmail] = useState("");
  const [error, setError] = useState<string | null>(null);
  const mutation = useMutation({
    mutationFn: () => customerApi.create({
      name,
      company_name: company || null,
      phone: phone || null,
      email: email || null,
      address: null,
      tax_number: null,
      opening_balance_cents: 0,
      notes: null
    }),
    onSuccess: async (customer) => {
      setName(""); setCompany(""); setPhone(""); setEmail(""); setError(null);
      await queryClient.invalidateQueries({ queryKey: ["customer"] });
      onCreated(customer);
    },
    onError: (reason) => setError(normalizeError(reason).message)
  });
  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>New customer</DialogTitle>
      <DialogContent>
        <Stack component="form" id="quick-customer-form" spacing={2} sx={{ pt: 1 }} onSubmit={(event) => { event.preventDefault(); mutation.mutate(); }}>
          {error ? <Alert severity="error">{error}</Alert> : null}
          <TextField label="Customer name" required value={name} onChange={(event) => setName(event.target.value)} />
          <TextField label="Company name" value={company} onChange={(event) => setCompany(event.target.value)} />
          <TextField label="Phone" value={phone} onChange={(event) => setPhone(event.target.value)} />
          <TextField label="Email" type="email" value={email} onChange={(event) => setEmail(event.target.value)} />
        </Stack>
      </DialogContent>
      <DialogActions><Button onClick={onClose}>Cancel</Button><Button type="submit" form="quick-customer-form" variant="contained" disabled={!name.trim() || mutation.isPending}>Create customer</Button></DialogActions>
    </Dialog>
  );
}

function ConversionDialog({ form, error, saving, onChange, onClose, onSubmit }: {
  form: ConversionForm | null;
  error: string | null;
  saving: boolean;
  onChange: (form: ConversionForm | null) => void;
  onClose: () => void;
  onSubmit: (event: FormEvent) => void;
}) {
  return (
    <Dialog open={Boolean(form)} onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>Convert pro forma to sales invoice</DialogTitle>
      <DialogContent>
        <Stack component="form" id="convert-quotation-form" onSubmit={onSubmit} spacing={2} sx={{ pt: 1 }}>
          <Alert severity="warning">
            This creates a real sale using the quoted customer, quantities, and prices. Current stock will be validated and reduced when the invoice is created.
          </Alert>
          {error ? <Alert severity="error">{error}</Alert> : null}
          <Typography variant="body2"><strong>{form?.quotation.quotation_number}</strong> · {form?.quotation.customer_name}</Typography>
          <TextField label="Sales invoice number" value={form?.invoice_number ?? ""} helperText="Leave blank for automatic numbering" onChange={(event) => onChange(form && { ...form, invoice_number: event.target.value })} />
          <TextField label="Invoice date" type="date" required value={form?.invoice_date ?? today()} onChange={(event) => onChange(form && { ...form, invoice_date: event.target.value })} />
          <TextField label="Delivery" type="number" value={form?.delivery ?? "0.00"} slotProps={{ htmlInput: { min: 0, step: 0.01 } }} onChange={(event) => onChange(form && { ...form, delivery: event.target.value })} />
          <TextField label="Amount paid now" type="number" value={form?.paid ?? "0.00"} slotProps={{ htmlInput: { min: 0, step: 0.01 } }} onChange={(event) => onChange(form && { ...form, paid: event.target.value })} />
        </Stack>
      </DialogContent>
      <DialogActions><Button onClick={onClose}>Cancel</Button><Button type="submit" form="convert-quotation-form" variant="contained" startIcon={<ShoppingCartCheckoutIcon />} disabled={saving}>Create sales invoice</Button></DialogActions>
    </Dialog>
  );
}

function quotationActions(row: QuotationListRow, handlers: {
  edit: (id: number) => void;
  print: (id: number) => void;
  setAction: (action: PendingAction) => void;
  convert: () => void;
}) {
  const actions: RowAction<QuotationListRow>[] = [{ label: "Print pro forma", icon: <PrintIcon fontSize="small" />, onClick: () => void handlers.print(row.id) }];
  if (row.status === "draft") {
    actions.unshift({ label: "Edit draft", icon: <EditIcon fontSize="small" />, onClick: () => void handlers.edit(row.id) });
    actions.push({ label: "Mark as sent", icon: <SendOutlinedIcon fontSize="small" />, onClick: () => handlers.setAction({ id: row.id, status: "sent", kind: "status", title: "Mark pro forma as sent", message: "This locks quoted values from editing and records that the pro forma was sent to the customer.", confirmLabel: "Mark sent" }) });
    actions.push({ label: "Delete draft", icon: <DeleteIcon fontSize="small" />, destructive: true, onClick: () => handlers.setAction({ id: row.id, kind: "delete", title: "Delete draft pro forma", message: "This permanently deletes the unsent draft. This action cannot be undone.", confirmLabel: "Delete draft" }) });
  }
  if (row.status === "sent") {
    actions.push({ label: "Mark accepted", icon: <CheckCircleOutlineIcon fontSize="small" />, onClick: () => handlers.setAction({ id: row.id, status: "accepted", kind: "status", title: "Accept pro forma", message: "Confirm that the customer accepted this pro forma. It can then be converted to a real sales invoice.", confirmLabel: "Mark accepted" }) });
    actions.push({ label: "Mark rejected", icon: <ThumbDownOutlinedIcon fontSize="small" />, destructive: true, onClick: () => handlers.setAction({ id: row.id, status: "rejected", kind: "status", title: "Reject pro forma", message: "Mark this pro forma as rejected while preserving its history.", confirmLabel: "Mark rejected" }) });
  }
  if (row.status === "accepted") {
    actions.push({ label: "Convert to sales invoice", icon: <ShoppingCartCheckoutIcon fontSize="small" />, onClick: handlers.convert });
    actions.push({ label: "Mark rejected", icon: <ThumbDownOutlinedIcon fontSize="small" />, destructive: true, onClick: () => handlers.setAction({ id: row.id, status: "rejected", kind: "status", title: "Reject accepted pro forma", message: "Use this when the customer withdrew acceptance. The historical pro forma will remain available.", confirmLabel: "Mark rejected" }) });
  }
  return actions;
}

function newQuotationForm(): QuotationForm {
  return {
    customer_id: "",
    quotation_number: "",
    quotation_date: today(),
    valid_until: addDays(today(), 30),
    discount: "0.00",
    tax: "0.00",
    notes: "",
    items: []
  };
}

function detailToForm(detail: QuotationDetail): QuotationForm {
  return {
    id: detail.quotation.id,
    customer_id: detail.quotation.customer_id ?? "",
    quotation_number: detail.quotation.quotation_number,
    quotation_date: detail.quotation.quotation_date,
    valid_until: detail.quotation.valid_until,
    discount: fromCents(detail.quotation.discount_cents),
    tax: fromCents(detail.quotation.tax_cents),
    notes: detail.quotation.notes ?? "",
    items: detail.items.map((item) => ({
      line_id: `quotation-item-${item.id}`,
      product_id: item.product_id ?? 0,
      quantity: String(item.quantity),
      unit_price: fromCents(item.unit_price_cents)
    }))
  };
}

function formToPayload(form: QuotationForm): QuotationPayload {
  return {
    customer_id: Number(form.customer_id),
    quotation_number: form.quotation_number || null,
    quotation_date: form.quotation_date,
    valid_until: form.valid_until,
    discount_cents: toCents(form.discount),
    tax_cents: toCents(form.tax),
    notes: form.notes || null,
    items: form.items.map((item) => ({
      product_id: item.product_id,
      quantity: Number(item.quantity),
      unit_price_cents: toCents(item.unit_price)
    }))
  };
}

function quotationTotals(form: QuotationForm | null) {
  const subtotal = form?.items.reduce((sum, item) => sum + Number(item.quantity || 0) * Number(item.unit_price || 0), 0) ?? 0;
  return { subtotal, total: subtotal - Number(form?.discount || 0) + Number(form?.tax || 0) };
}

function newLineId() {
  return globalThis.crypto?.randomUUID?.() ?? `quotation-line-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function addDays(value: string, days: number) {
  const date = new Date(`${value}T12:00:00`);
  date.setDate(date.getDate() + days);
  return date.toISOString().slice(0, 10);
}
