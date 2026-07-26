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
  Stack,
  TextField
} from "@mui/material";
import AddIcon from "@mui/icons-material/Add";
import EditIcon from "@mui/icons-material/Edit";
import DeleteIcon from "@mui/icons-material/Delete";
import PaymentsOutlinedIcon from "@mui/icons-material/PaymentsOutlined";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { PageHeader } from "../../components/PageHeader";
import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { LoadingState } from "../../components/feedback/PageState";
import { EnterpriseTable, type TableColumn } from "../../components/table/EnterpriseTable";
import { StatusBadge } from "../../components/table/StatusBadge";
import { expenseApi, settingsApi } from "../../lib/api";
import { paymentMethods } from "../../lib/constants";
import { fromCents, money, toCents, today } from "../../lib/formatters";
import { normalizeError } from "../../lib/tauri";
import type {
  ExpensePayload,
  ExpenseRow,
  InstallmentPaymentPayload,
  InstallmentPaymentRow
} from "../../types/payment";

type ExpenseForm = Omit<ExpensePayload, "amount_cents" | "paid_cents"> & {
  id?: number;
  amount: string;
  paid: string;
};

const blankForm: ExpenseForm = {
  expense_category_id: 0,
  title: "",
  amount: "0.00",
  paid: "0.00",
  currency: "USD",
  expense_date: today(),
  payment_method: "cash",
  notes: ""
};

export function ExpensesPage() {
  const queryClient = useQueryClient();
  const [form, setForm] = useState<ExpenseForm | null>(null);
  const [deleteId, setDeleteId] = useState<number | null>(null);
  const [paymentExpenseId, setPaymentExpenseId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const { data: categories = [] } = useQuery({ queryKey: ["expense-categories"], queryFn: expenseApi.categories });
  const { data: expenses = [], isLoading } = useQuery({ queryKey: ["expenses"], queryFn: () => expenseApi.list({ active_only: true }) });
  const { data: settings } = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get });
  const activeExpenses = useMemo(() => expenses.filter((expense) => expense.status === "active"), [expenses]);
  const paymentExpense = activeExpenses.find((expense) => expense.id === paymentExpenseId) ?? null;
  const columns = useMemo<TableColumn<ExpenseRow>[]>(() => [
    { id: "date", label: "Date", value: (row) => row.expense_date, width: 110 },
    { id: "category", label: "Category", value: (row) => row.category_name, minWidth: 150 },
    { id: "title", label: "Title", value: (row) => row.title, minWidth: 200 },
    { id: "amount", label: "Total", value: (row) => money(row.amount_cents, row.currency), render: (row) => <MoneyText value={row.amount_cents} currency={row.currency} />, align: "right", minWidth: 100 },
    { id: "paid", label: "Paid", value: (row) => money(row.paid_cents, row.currency), render: (row) => <MoneyText value={row.paid_cents} currency={row.currency} />, align: "right", minWidth: 100 },
    { id: "remaining", label: "Remaining", value: (row) => money(row.remaining_cents, row.currency), render: (row) => <MoneyText value={row.remaining_cents} currency={row.currency} />, align: "right", minWidth: 105 },
    { id: "status", label: "Payment", value: (row) => row.payment_status, render: (row) => <StatusBadge value={row.payment_status} />, width: 105 }
  ], []);

  const saveMutation = useMutation({
    mutationFn: (value: ExpenseForm) => value.id ? expenseApi.update(value.id, formToPayload(value)) : expenseApi.create(formToPayload(value)),
    onSuccess: async () => {
      setForm(null);
      setError(null);
      await queryClient.invalidateQueries();
    },
    onError: (err) => setError(normalizeError(err).message)
  });
  const deleteMutation = useMutation({
    mutationFn: (id: number) => expenseApi.cancel(id),
    onSuccess: async () => {
      setDeleteId(null);
      setError(null);
      await queryClient.invalidateQueries();
    },
    onError: (err) => setError(normalizeError(err).message)
  });
  function submit(event: FormEvent) {
    event.preventDefault();
    if (form) saveMutation.mutate({ ...form, expense_category_id: form.expense_category_id || categories[0]?.id || 0 });
  }

  if (isLoading) return <LoadingState label="Loading expenses" />;

  return (
    <Stack spacing={2}>
      <PageHeader
        title="Expenses"
        description="Record business expenses by category, date, and payment method."
        actions={
          <Button startIcon={<AddIcon />} variant="contained" onClick={() => setForm({ ...blankForm, expense_category_id: categories[0]?.id ?? 0 })}>Add expense</Button>
        }
      />
      {error && !form && deleteId === null ? <Alert severity="error">{error}</Alert> : null}
      <EnterpriseTable
        title="Expenses"
        rows={activeExpenses}
        columns={columns}
        rowId={(row) => row.id}
        loading={isLoading}
        initialSort={{ column: "date", direction: "desc" }}
        emptyTitle="No active expenses"
        emptyDescription="Add an expense to begin tracking operating costs."
        actions={(row) => [
          { label: "Edit", icon: <EditIcon fontSize="small" />, onClick: () => setForm(rowToForm(row)) },
          { label: "Payments", icon: <PaymentsOutlinedIcon fontSize="small" />, onClick: () => setPaymentExpenseId(row.id) },
          { label: "Delete", icon: <DeleteIcon fontSize="small" />, destructive: true, onClick: () => setDeleteId(row.id) }
        ]}
      />

      <Dialog open={Boolean(form)} onClose={() => setForm(null)} fullWidth maxWidth="sm">
        <DialogTitle>{form?.id ? "Edit expense" : "Add expense"}</DialogTitle>
        <DialogContent>
          <Stack component="form" id="expense-form" onSubmit={submit} spacing={2} sx={{ pt: 1 }}>
            {error ? <Alert severity="error">{error}</Alert> : null}
            <TextField select label="Category" value={form?.expense_category_id || ""} onChange={(e) => setForm((current) => current && { ...current, expense_category_id: Number(e.target.value) })}>
              {categories.map((category) => <MenuItem key={category.id} value={category.id}>{category.name}</MenuItem>)}
            </TextField>
            <TextField label="Title" required value={form?.title ?? ""} onChange={(e) => setForm((current) => current && { ...current, title: e.target.value })} />
            <TextField label="Amount" value={form?.amount ?? "0.00"} onChange={(e) => setForm((current) => current && { ...current, amount: e.target.value })} />
            <TextField
              label={form?.id ? "Paid to date" : "Amount paid now"}
              type="number"
              disabled={Boolean(form?.id)}
              value={form?.paid ?? "0.00"}
              onChange={(e) => setForm((current) => current && { ...current, paid: e.target.value })}
              slotProps={{ htmlInput: { min: 0, max: form?.amount || "0", step: "0.01" } }}
              helperText={form?.id ? "Use Payments to record another installment" : "Any unpaid amount remains open"}
            />
            <TextField label="Date" type="date" value={form?.expense_date ?? today()} onChange={(e) => setForm((current) => current && { ...current, expense_date: e.target.value })} />
            <TextField select label="Payment method" value={form?.payment_method ?? "cash"} onChange={(e) => setForm((current) => current && { ...current, payment_method: e.target.value })}>
              {paymentMethods.map((method) => <MenuItem key={method} value={method}>{method}</MenuItem>)}
            </TextField>
            <TextField label="Notes" multiline minRows={2} value={form?.notes ?? ""} onChange={(e) => setForm((current) => current && { ...current, notes: e.target.value })} />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setForm(null)}>Cancel</Button>
          <Button type="submit" form="expense-form" variant="contained" disabled={saveMutation.isPending}>Save</Button>
        </DialogActions>
      </Dialog>

      <ConfirmDialog open={deleteId !== null} title="Delete expense" message="This removes the expense and its payments from active totals while preserving audit history." confirmLabel="Delete" error={error} loading={deleteMutation.isPending} onClose={() => setDeleteId(null)} onConfirm={() => deleteId && deleteMutation.mutate(deleteId)} />
      <ExpensePaymentDialog expense={paymentExpense} currency={settings?.default_currency} onClose={() => setPaymentExpenseId(null)} />
    </Stack>
  );
}

function ExpensePaymentDialog({
  expense,
  currency,
  onClose
}: {
  expense: ExpenseRow | null;
  currency?: string;
  onClose: () => void;
}) {
  const queryClient = useQueryClient();
  const [amount, setAmount] = useState("0.00");
  const [paymentMethod, setPaymentMethod] = useState("cash");
  const [paymentDate, setPaymentDate] = useState(today());
  const [notes, setNotes] = useState("");
  const [paymentError, setPaymentError] = useState<string | null>(null);
  const { data: payments = [], isLoading } = useQuery({
    queryKey: ["expense-payments", expense?.id],
    queryFn: () => expenseApi.payments(expense!.id),
    enabled: Boolean(expense)
  });
  const paymentColumns = useMemo<TableColumn<InstallmentPaymentRow>[]>(() => [
    { id: "date", label: "Date", value: (row) => row.payment_date, width: 110 },
    { id: "amount", label: "Paid this time", value: (row) => money(row.amount_cents, row.currency), render: (row) => <MoneyText value={row.amount_cents} currency={row.currency} />, align: "right", minWidth: 130 },
    { id: "method", label: "Method", value: (row) => row.payment_method, width: 100 },
    { id: "notes", label: "Notes", value: (row) => row.notes ?? "", minWidth: 170 }
  ], []);
  const mutation = useMutation({
    mutationFn: (payload: InstallmentPaymentPayload) =>
      expenseApi.recordPayment(expense!.id, payload),
    onSuccess: async () => {
      setAmount("0.00");
      setNotes("");
      setPaymentError(null);
      await queryClient.invalidateQueries();
    },
    onError: (err) => setPaymentError(normalizeError(err).message)
  });

  function close() {
    setAmount("0.00");
    setNotes("");
    setPaymentError(null);
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
    <Dialog open={Boolean(expense)} onClose={close} fullWidth maxWidth="md">
      <DialogTitle>Payments — {expense?.title}</DialogTitle>
      <DialogContent>
        <Stack spacing={2} sx={{ pt: 1 }}>
          <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", sm: "repeat(3, 1fr)" }, gap: 1.5 }}>
            <ExpensePaymentSummary label="Expense total" value={expense?.amount_cents ?? 0} currency={expense?.currency ?? currency} />
            <ExpensePaymentSummary label="Paid to date" value={expense?.paid_cents ?? 0} currency={expense?.currency ?? currency} />
            <ExpensePaymentSummary label="Remaining" value={expense?.remaining_cents ?? 0} currency={expense?.currency ?? currency} />
          </Box>
          <EnterpriseTable
            title={`Payment history — ${expense?.title ?? ""}`}
            rows={payments}
            columns={paymentColumns}
            rowId={(row) => row.id}
            loading={isLoading}
            searchable={false}
            selectable={false}
            compact
            maxHeight={260}
            emptyTitle="No payments recorded"
            emptyDescription="Record the first payment below."
          />
          {expense && expense.remaining_cents > 0 ? (
            <Stack component="form" id="expense-payment-form" onSubmit={submit} spacing={2}>
              {paymentError ? <Alert severity="error">{paymentError}</Alert> : null}
              <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", sm: "repeat(3, 1fr)" }, gap: 2 }}>
                <TextField
                  label="Amount paid this time"
                  type="number"
                  required
                  value={amount}
                  onChange={(event) => setAmount(event.target.value)}
                  slotProps={{ htmlInput: { min: 0.01, max: fromCents(expense.remaining_cents), step: "0.01" } }}
                  helperText={`Maximum ${money(expense.remaining_cents, currency)}`}
                />
                <TextField select label="Payment method" value={paymentMethod} onChange={(event) => setPaymentMethod(event.target.value)}>
                  {paymentMethods.map((method) => <MenuItem key={method} value={method}>{method}</MenuItem>)}
                </TextField>
                <TextField label="Payment date" type="date" value={paymentDate} onChange={(event) => setPaymentDate(event.target.value)} />
              </Box>
              <TextField label="Notes" value={notes} onChange={(event) => setNotes(event.target.value)} />
            </Stack>
          ) : (
            <Alert severity="success">This expense is fully paid.</Alert>
          )}
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={close}>Close</Button>
        {expense && expense.remaining_cents > 0 ? (
          <Button type="submit" form="expense-payment-form" variant="contained" disabled={mutation.isPending}>
            Record payment
          </Button>
        ) : null}
      </DialogActions>
    </Dialog>
  );
}

function ExpensePaymentSummary({ label, value, currency }: { label: string; value: number; currency?: string }) {
  return (
    <Box sx={{ borderBottom: "1px solid", borderColor: "divider", pb: 1 }}>
      <Box component="span" sx={{ display: "block", color: "text.secondary", fontSize: 13 }}>{label}</Box>
      <Box component="strong" sx={{ display: "block", mt: 0.25, fontSize: 18, fontVariantNumeric: "tabular-nums" }}>
        <MoneyText value={value} currency={currency} />
      </Box>
    </Box>
  );
}

function formToPayload(form: ExpenseForm): ExpensePayload {
  return { ...form, amount_cents: toCents(form.amount), paid_cents: toCents(form.paid) };
}

function rowToForm(row: ExpenseRow): ExpenseForm {
  return {
    id: row.id,
    expense_category_id: row.expense_category_id,
    title: row.title,
    amount: fromCents(row.amount_cents),
    paid: fromCents(row.paid_cents),
    currency: row.currency,
    expense_date: row.expense_date,
    payment_method: row.payment_method,
    notes: row.notes ?? ""
  };
}
