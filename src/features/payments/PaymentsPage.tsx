import { FormEvent, useMemo, useState } from "react";
import {
  Alert,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  MenuItem,
  Stack,
  TextField
} from "@mui/material";
import AddIcon from "@mui/icons-material/Add";
import DeleteIcon from "@mui/icons-material/Delete";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { PageHeader } from "../../components/PageHeader";
import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { LoadingState } from "../../components/feedback/PageState";
import { EnterpriseTable, type TableColumn } from "../../components/table/EnterpriseTable";
import { StatusBadge } from "../../components/table/StatusBadge";
import { customerApi, paymentApi, supplierApi } from "../../lib/api";
import { paymentMethods } from "../../lib/constants";
import { money, toCents, today } from "../../lib/formatters";
import { normalizeError } from "../../lib/tauri";
import type { PaymentPayload, PaymentRow } from "../../types/payment";

type PaymentForm = Omit<PaymentPayload, "amount_cents"> & { amount: string };

const blankForm: PaymentForm = {
  party_type: "customer",
  party_id: 0,
  amount: "0.00",
  currency: "USD",
  payment_method: "cash",
  payment_date: today(),
  reference_type: null,
  reference_id: null,
  notes: ""
};

export function PaymentsPage() {
  const queryClient = useQueryClient();
  const [form, setForm] = useState<PaymentForm | null>(null);
  const [deleteId, setDeleteId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const { data: payments = [], isLoading } = useQuery({ queryKey: ["payments"], queryFn: () => paymentApi.list({ active_only: true }) });
  const { data: customers = [] } = useQuery({ queryKey: ["customers", "payments"], queryFn: () => customerApi.list({ active_only: true }) });
  const { data: suppliers = [] } = useQuery({ queryKey: ["suppliers", "payments"], queryFn: () => supplierApi.list({ active_only: true }) });
  const activePayments = useMemo(() => payments.filter((payment) => payment.status === "active"), [payments]);
  const columns = useMemo<TableColumn<PaymentRow>[]>(() => [
    { id: "date", label: "Date", value: (row) => row.payment_date, width: 110 },
    { id: "party", label: "Party", value: (row) => row.party_name, minWidth: 180 },
    { id: "direction", label: "Direction", value: (row) => row.payment_direction, render: (row) => <StatusBadge value={row.payment_direction} />, width: 110 },
    { id: "method", label: "Method", value: (row) => row.payment_method, width: 110 },
    { id: "amount", label: "Amount", value: (row) => money(row.amount_cents, row.currency), render: (row) => <MoneyText value={row.amount_cents} currency={row.currency} />, align: "right", minWidth: 120 },
    { id: "reference", label: "Reference", value: (row) => row.reference_type ? `${row.reference_type.replace(/_/g, " ")} #${row.reference_id}` : "General", minWidth: 170 }
  ], []);

  const partyOptions = useMemo(() => form?.party_type === "supplier" ? suppliers : customers, [customers, form?.party_type, suppliers]);

  const saveMutation = useMutation({
    mutationFn: (value: PaymentForm) => paymentApi.create({ ...value, party_id: value.party_id || partyOptions[0]?.id || 0, amount_cents: toCents(value.amount) }),
    onSuccess: async () => {
      setForm(null);
      setError(null);
      await queryClient.invalidateQueries();
    },
    onError: (err) => setError(normalizeError(err).message)
  });
  const deleteMutation = useMutation({
    mutationFn: (id: number) => paymentApi.cancel(id),
    onSuccess: async () => {
      setDeleteId(null);
      setError(null);
      await queryClient.invalidateQueries();
    },
    onError: (err) => setError(normalizeError(err).message)
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    if (form) saveMutation.mutate(form);
  }

  if (isLoading) return <LoadingState label="Loading payments" />;

  return (
    <Stack spacing={2}>
      <PageHeader title="Payments" description="Record customer money-in and supplier money-out payments." actions={
        <Button startIcon={<AddIcon />} variant="contained" onClick={() => setForm(blankForm)}>Add payment</Button>
      } />
      {error && !form && deleteId === null ? <Alert severity="error">{error}</Alert> : null}
      <EnterpriseTable
        title="Payments"
        rows={activePayments}
        columns={columns}
        rowId={(row) => row.id}
        loading={isLoading}
        initialSort={{ column: "date", direction: "desc" }}
        emptyTitle="No active payments"
        emptyDescription="Add a payment to begin tracking cash movement and balances."
        actions={(row) => [
          { label: "Delete", icon: <DeleteIcon fontSize="small" />, destructive: true, onClick: () => setDeleteId(row.id) }
        ]}
      />

      <Dialog open={Boolean(form)} onClose={() => setForm(null)} fullWidth maxWidth="sm">
        <DialogTitle>Add payment</DialogTitle>
        <DialogContent>
          <Stack component="form" id="payment-form" onSubmit={submit} spacing={2} sx={{ pt: 1 }}>
            {error ? <Alert severity="error">{error}</Alert> : null}
            <TextField select label="Party type" value={form?.party_type ?? "customer"} onChange={(e) => setForm((current) => current && { ...current, party_type: e.target.value, party_id: 0 })}>
              <MenuItem value="customer">Customer payment</MenuItem>
              <MenuItem value="supplier">Supplier payment</MenuItem>
            </TextField>
            <TextField select label="Party" value={form?.party_id || ""} onChange={(e) => setForm((current) => current && { ...current, party_id: Number(e.target.value) })}>
              {partyOptions.map((party) => <MenuItem key={party.id} value={party.id}>{party.name}</MenuItem>)}
            </TextField>
            <TextField label="Amount" value={form?.amount ?? "0.00"} onChange={(e) => setForm((current) => current && { ...current, amount: e.target.value })} />
            <TextField label="Payment date" type="date" value={form?.payment_date ?? today()} onChange={(e) => setForm((current) => current && { ...current, payment_date: e.target.value })} />
            <TextField select label="Payment method" value={form?.payment_method ?? "cash"} onChange={(e) => setForm((current) => current && { ...current, payment_method: e.target.value })}>
              {paymentMethods.map((method) => <MenuItem key={method} value={method}>{method}</MenuItem>)}
            </TextField>
            <TextField select label="Reference type" value={form?.reference_type ?? ""} onChange={(e) => setForm((current) => current && { ...current, reference_type: e.target.value || null })}>
              <MenuItem value="">General payment</MenuItem>
              <MenuItem value={form?.party_type === "supplier" ? "purchase_invoice" : "sales_invoice"}>Link to invoice ID</MenuItem>
            </TextField>
            {form?.reference_type ? <TextField label="Invoice ID" type="number" value={form.reference_id ?? ""} onChange={(e) => setForm((current) => current && { ...current, reference_id: e.target.value ? Number(e.target.value) : null })} /> : null}
            <TextField label="Notes" multiline minRows={2} value={form?.notes ?? ""} onChange={(e) => setForm((current) => current && { ...current, notes: e.target.value })} />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setForm(null)}>Cancel</Button>
          <Button type="submit" form="payment-form" variant="contained" disabled={saveMutation.isPending}>Save</Button>
        </DialogActions>
      </Dialog>

      <ConfirmDialog open={deleteId !== null} title="Delete payment" message="This removes the payment from active balances, reports, cash totals, and any linked invoice while preserving audit history." confirmLabel="Delete" error={error} loading={deleteMutation.isPending} onClose={() => setDeleteId(null)} onConfirm={() => deleteId && deleteMutation.mutate(deleteId)} />
    </Stack>
  );
}
