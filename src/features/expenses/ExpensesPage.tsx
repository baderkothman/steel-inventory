import { FormEvent, useState } from "react";
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
import EditIcon from "@mui/icons-material/Edit";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { PageHeader } from "../../components/PageHeader";
import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { EmptyState, LoadingState } from "../../components/feedback/PageState";
import { expenseApi } from "../../lib/api";
import { paymentMethods } from "../../lib/constants";
import { fromCents, toCents, today } from "../../lib/formatters";
import { normalizeError } from "../../lib/tauri";
import type { ExpensePayload, ExpenseRow } from "../../types/payment";

type ExpenseForm = Omit<ExpensePayload, "amount_cents"> & { id?: number; amount: string };

const blankForm: ExpenseForm = {
  expense_category_id: 0,
  title: "",
  amount: "0.00",
  currency: "USD",
  expense_date: today(),
  payment_method: "cash",
  notes: ""
};

export function ExpensesPage() {
  const queryClient = useQueryClient();
  const [form, setForm] = useState<ExpenseForm | null>(null);
  const [deleteId, setDeleteId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const { data: categories = [] } = useQuery({ queryKey: ["expense-categories"], queryFn: expenseApi.categories });
  const { data: expenses = [], isLoading } = useQuery({ queryKey: ["expenses"], queryFn: () => expenseApi.list() });

  const saveMutation = useMutation({
    mutationFn: (value: ExpenseForm) => value.id ? expenseApi.update(value.id, formToPayload(value)) : expenseApi.create(formToPayload(value)),
    onSuccess: async () => {
      setForm(null);
      setError(null);
      await queryClient.invalidateQueries({ queryKey: ["expenses"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    },
    onError: (err) => setError(normalizeError(err).message)
  });
  const deleteMutation = useMutation({
    mutationFn: (id: number) => expenseApi.delete(id),
    onSuccess: async () => {
      setDeleteId(null);
      await queryClient.invalidateQueries({ queryKey: ["expenses"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    }
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
        actions={<Button startIcon={<AddIcon />} variant="contained" onClick={() => setForm({ ...blankForm, expense_category_id: categories[0]?.id ?? 0 })}>Add expense</Button>}
      />
      <Paper variant="outlined">
        {expenses.length === 0 ? <EmptyState label="No expenses recorded." /> : (
          <Table size="small">
            <TableHead><TableRow><TableCell>Date</TableCell><TableCell>Category</TableCell><TableCell>Title</TableCell><TableCell>Method</TableCell><TableCell align="right">Amount</TableCell><TableCell align="right">Actions</TableCell></TableRow></TableHead>
            <TableBody>{expenses.map((expense) => <TableRow key={expense.id} hover><TableCell>{expense.expense_date}</TableCell><TableCell>{expense.category_name}</TableCell><TableCell>{expense.title}</TableCell><TableCell>{expense.payment_method}</TableCell><TableCell align="right"><MoneyText value={expense.amount_cents} currency={expense.currency} /></TableCell><TableCell align="right"><Button size="small" startIcon={<EditIcon />} onClick={() => setForm(rowToForm(expense))}>Edit</Button><Button size="small" color="error" startIcon={<DeleteIcon />} onClick={() => setDeleteId(expense.id)}>Delete</Button></TableCell></TableRow>)}</TableBody>
          </Table>
        )}
      </Paper>

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

      <ConfirmDialog open={deleteId !== null} title="Delete expense" message="Delete this expense only if it was entered by mistake." confirmLabel="Delete" onClose={() => setDeleteId(null)} onConfirm={() => deleteId && deleteMutation.mutate(deleteId)} />
    </Stack>
  );
}

function formToPayload(form: ExpenseForm): ExpensePayload {
  return { ...form, amount_cents: toCents(form.amount) };
}

function rowToForm(row: ExpenseRow): ExpenseForm {
  return {
    id: row.id,
    expense_category_id: row.expense_category_id,
    title: row.title,
    amount: fromCents(row.amount_cents),
    currency: row.currency,
    expense_date: row.expense_date,
    payment_method: row.payment_method,
    notes: row.notes ?? ""
  };
}
