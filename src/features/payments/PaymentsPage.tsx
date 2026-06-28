import { FormEvent, useMemo, useState } from "react";
import {
  Alert,
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
  TextField
} from "@mui/material";
import AddIcon from "@mui/icons-material/Add";
import DeleteIcon from "@mui/icons-material/Delete";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { PageHeader } from "../../components/PageHeader";
import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { EmptyState, LoadingState } from "../../components/feedback/PageState";
import { customerApi, paymentApi, supplierApi } from "../../lib/api";
import { paymentMethods } from "../../lib/constants";
import { toCents, today } from "../../lib/formatters";
import { normalizeError } from "../../lib/tauri";
import type { PaymentPayload } from "../../types/payment";

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
  const { data: payments = [], isLoading } = useQuery({ queryKey: ["payments"], queryFn: () => paymentApi.list() });
  const { data: customers = [] } = useQuery({ queryKey: ["customers", "payments"], queryFn: () => customerApi.list({ active_only: true }) });
  const { data: suppliers = [] } = useQuery({ queryKey: ["suppliers", "payments"], queryFn: () => supplierApi.list({ active_only: true }) });

  const partyOptions = useMemo(() => form?.party_type === "supplier" ? suppliers : customers, [customers, form?.party_type, suppliers]);

  const saveMutation = useMutation({
    mutationFn: (value: PaymentForm) => paymentApi.create({ ...value, party_id: value.party_id || partyOptions[0]?.id || 0, amount_cents: toCents(value.amount) }),
    onSuccess: async () => {
      setForm(null);
      setError(null);
      await queryClient.invalidateQueries({ queryKey: ["payments"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      await queryClient.invalidateQueries({ queryKey: ["customer"] });
      await queryClient.invalidateQueries({ queryKey: ["supplier"] });
    },
    onError: (err) => setError(normalizeError(err).message)
  });
  const deleteMutation = useMutation({
    mutationFn: (id: number) => paymentApi.delete(id),
    onSuccess: async () => {
      setDeleteId(null);
      await queryClient.invalidateQueries({ queryKey: ["payments"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    }
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    if (form) saveMutation.mutate(form);
  }

  if (isLoading) return <LoadingState label="Loading payments" />;

  return (
    <Stack spacing={2}>
      <PageHeader title="Payments" description="Record customer money-in and supplier money-out payments." actions={<Button startIcon={<AddIcon />} variant="contained" onClick={() => setForm(blankForm)}>Add payment</Button>} />
      <Paper variant="outlined">
        {payments.length === 0 ? <EmptyState label="No payments recorded." /> : (
          <Table size="small">
            <TableHead><TableRow><TableCell>Date</TableCell><TableCell>Party</TableCell><TableCell>Direction</TableCell><TableCell>Method</TableCell><TableCell align="right">Amount</TableCell><TableCell>Reference</TableCell><TableCell align="right">Actions</TableCell></TableRow></TableHead>
            <TableBody>{payments.map((payment) => <TableRow key={payment.id} hover><TableCell>{payment.payment_date}</TableCell><TableCell>{payment.party_name}</TableCell><TableCell>{payment.payment_direction}</TableCell><TableCell>{payment.payment_method}</TableCell><TableCell align="right"><MoneyText value={payment.amount_cents} currency={payment.currency} /></TableCell><TableCell>{payment.reference_type ? `${payment.reference_type} #${payment.reference_id}` : "General"}</TableCell><TableCell align="right"><Button size="small" color="error" startIcon={<DeleteIcon />} onClick={() => setDeleteId(payment.id)}>Delete</Button></TableCell></TableRow>)}</TableBody>
          </Table>
        )}
      </Paper>

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

      <ConfirmDialog open={deleteId !== null} title="Delete payment" message="Deleting a linked payment also updates the cached invoice paid and remaining values." confirmLabel="Delete" onClose={() => setDeleteId(null)} onConfirm={() => deleteId && deleteMutation.mutate(deleteId)} />
    </Stack>
  );
}
