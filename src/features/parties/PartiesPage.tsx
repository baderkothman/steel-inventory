import { FormEvent, useMemo, useState } from "react";
import {
  Alert,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Drawer,
  Stack,
  TextField
} from "@mui/material";
import AddIcon from "@mui/icons-material/Add";
import DeleteIcon from "@mui/icons-material/Delete";
import EditIcon from "@mui/icons-material/Edit";
import ReceiptLongIcon from "@mui/icons-material/ReceiptLong";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { PageHeader } from "../../components/PageHeader";
import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { LoadingState } from "../../components/feedback/PageState";
import { EnterpriseTable, type TableColumn } from "../../components/table/EnterpriseTable";
import { customerApi, settingsApi, supplierApi } from "../../lib/api";
import { fromCents, money, toCents } from "../../lib/formatters";
import { normalizeError } from "../../lib/tauri";
import type { Party, PartyPayload, StatementRow } from "../../types/party";

type Kind = "supplier" | "customer";
type PartyForm = Omit<PartyPayload, "opening_balance_cents"> & {
  id?: number;
  opening_balance: string;
};

const blankForm: PartyForm = {
  name: "",
  company_name: "",
  phone: "",
  email: "",
  address: "",
  tax_number: "",
  opening_balance: "0.00",
  notes: ""
};

export function SuppliersPage() {
  return <PartiesPage kind="supplier" />;
}

export function CustomersPage() {
  return <PartiesPage kind="customer" />;
}

function PartiesPage({ kind }: { kind: Kind }) {
  const api = kind === "supplier" ? supplierApi : customerApi;
  const queryClient = useQueryClient();
  const [form, setForm] = useState<PartyForm | null>(null);
  const [deleteId, setDeleteId] = useState<number | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [statementParty, setStatementParty] = useState<Party | null>(null);
  const [error, setError] = useState<string | null>(null);

  const { data = [], isLoading } = useQuery({
    queryKey: [kind],
    queryFn: () => api.list({ active_only: true })
  });
  const { data: settings } = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get });
  const activeRows = useMemo(() => data.filter((row) => row.is_active), [data]);
  const columns = useMemo<TableColumn<Party>[]>(() => [
    { id: "name", label: "Name", value: (row) => row.name, minWidth: 180 },
    { id: "company", label: "Company", value: (row) => row.company_name ?? "", minWidth: 180 },
    { id: "phone", label: "Phone", value: (row) => row.phone ?? "", minWidth: 140 },
    { id: "balance", label: "Balance", value: (row) => money(row.balance_cents, settings?.default_currency), render: (row) => <MoneyText value={row.balance_cents} currency={settings?.default_currency} />, align: "right", minWidth: 130 }
  ], [settings?.default_currency]);

  const saveMutation = useMutation({
    mutationFn: (value: PartyForm) => (value.id ? api.update(value.id, formToPayload(value)) : api.create(formToPayload(value))),
    onSuccess: async () => {
      setForm(null);
      setError(null);
      await queryClient.invalidateQueries({ queryKey: [kind] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    },
    onError: (err) => setError(normalizeError(err).message)
  });

  const deleteMutation = useMutation({
    mutationFn: (id: number) => api.archive(id),
    onSuccess: async () => {
      setDeleteId(null);
      setDeleteError(null);
      await queryClient.invalidateQueries({ queryKey: [kind] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    },
    onError: (err) => setDeleteError(normalizeError(err).message)
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    if (form) saveMutation.mutate(form);
  }

  if (isLoading) return <LoadingState label={`Loading ${kind}s`} />;

  const title = kind === "supplier" ? "Suppliers" : "Customers";

  return (
    <Stack spacing={2}>
      <PageHeader
        title={title}
        description={`Manage ${kind} profiles, opening balances, payments, and statements.`}
        actions={
          <Button startIcon={<AddIcon />} variant="contained" onClick={() => setForm(blankForm)}>Add {kind}</Button>
        }
      />
      {deleteError && deleteId === null ? <Alert severity="error">{deleteError}</Alert> : null}

      <EnterpriseTable
        title={title}
        rows={activeRows}
        columns={columns}
        rowId={(row) => row.id}
        loading={isLoading}
        emptyTitle={`No active ${kind}s`}
        emptyDescription={`Add a ${kind} to begin tracking invoices and balances.`}
        actions={(row) => [
          { label: "Edit", icon: <EditIcon fontSize="small" />, onClick: () => setForm(partyToForm(row)) },
          { label: "View statement", icon: <ReceiptLongIcon fontSize="small" />, onClick: () => setStatementParty(row) },
          { label: "Delete", icon: <DeleteIcon fontSize="small" />, destructive: true, onClick: () => setDeleteId(row.id) }
        ]}
      />

      <Dialog open={Boolean(form)} onClose={() => setForm(null)} fullWidth maxWidth="sm">
        <DialogTitle>{form?.id ? `Edit ${kind}` : `Add ${kind}`}</DialogTitle>
        <DialogContent>
          <Stack component="form" id={`${kind}-form`} onSubmit={submit} spacing={2} sx={{ pt: 1 }}>
            {error ? <Alert severity="error">{error}</Alert> : null}
            <TextField label="Name" required value={form?.name ?? ""} onChange={(e) => setForm((current) => current && { ...current, name: e.target.value })} />
            <TextField label="Company name" value={form?.company_name ?? ""} onChange={(e) => setForm((current) => current && { ...current, company_name: e.target.value })} />
            <TextField label="Phone" value={form?.phone ?? ""} onChange={(e) => setForm((current) => current && { ...current, phone: e.target.value })} />
            <TextField label="Email" type="email" value={form?.email ?? ""} onChange={(e) => setForm((current) => current && { ...current, email: e.target.value })} />
            <TextField label="Address" value={form?.address ?? ""} onChange={(e) => setForm((current) => current && { ...current, address: e.target.value })} />
            <TextField label="Tax number" value={form?.tax_number ?? ""} onChange={(e) => setForm((current) => current && { ...current, tax_number: e.target.value })} />
            <TextField label="Opening balance" value={form?.opening_balance ?? "0.00"} onChange={(e) => setForm((current) => current && { ...current, opening_balance: e.target.value })} />
            <TextField label="Notes" multiline minRows={2} value={form?.notes ?? ""} onChange={(e) => setForm((current) => current && { ...current, notes: e.target.value })} />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setForm(null)}>Cancel</Button>
          <Button type="submit" form={`${kind}-form`} variant="contained" disabled={saveMutation.isPending}>Save</Button>
        </DialogActions>
      </Dialog>

      <StatementDrawer kind={kind} party={statementParty} onClose={() => setStatementParty(null)} />
      <ConfirmDialog
        open={deleteId !== null}
        title={`Delete ${kind}`}
        message={`This removes the ${kind} from active lists while preserving invoices, payments, and statements.`}
        confirmLabel="Delete"
        error={deleteError}
        loading={deleteMutation.isPending}
        onClose={() => {
          setDeleteId(null);
          setDeleteError(null);
        }}
        onConfirm={() => deleteId !== null && deleteMutation.mutate(deleteId)}
      />
    </Stack>
  );
}

function StatementDrawer({ kind, party, onClose }: { kind: Kind; party: Party | null; onClose: () => void }) {
  const api = kind === "supplier" ? supplierApi : customerApi;
  const { data = [], isLoading } = useQuery({
    queryKey: [kind, "statement", party?.id],
    queryFn: () => api.statement(party!.id),
    enabled: Boolean(party)
  });
  const statementColumns = useMemo<TableColumn<StatementRow>[]>(() => [
    { id: "date", label: "Date", value: (row) => row.date, width: 110 },
    { id: "type", label: "Type", value: (row) => row.entry_type, minWidth: 140 },
    { id: "reference", label: "Reference", value: (row) => row.reference, minWidth: 150 },
    { id: "debit", label: "Debit", value: (row) => money(row.debit_cents), render: (row) => <MoneyText value={row.debit_cents} />, align: "right" },
    { id: "credit", label: "Credit", value: (row) => money(row.credit_cents), render: (row) => <MoneyText value={row.credit_cents} />, align: "right" },
    { id: "balance", label: "Balance", value: (row) => money(row.balance_cents), render: (row) => <MoneyText value={row.balance_cents} />, align: "right" }
  ], []);

  return (
    <Drawer anchor="right" open={Boolean(party)} onClose={onClose}>
      <Box sx={{ width: { xs: "100vw", md: 760 }, maxWidth: "100vw", p: { xs: 2, md: 3 } }}>
        <PageHeader
          title="Statement"
          description={party?.name}
        />
        <Box sx={{ mt: 2 }}>
          <EnterpriseTable
            title={`${kind === "supplier" ? "Supplier" : "Customer"} Statement — ${party?.name ?? ""}`}
            rows={data}
            columns={statementColumns}
            rowId={(row) => `${row.reference}-${row.date}-${row.balance_cents}`}
            loading={isLoading}
            searchable={false}
            emptyTitle="No statement activity"
            emptyDescription="Active invoices and payments will appear here."
          />
        </Box>
      </Box>
    </Drawer>
  );
}

function formToPayload(form: PartyForm): PartyPayload {
  return {
    name: form.name,
    company_name: form.company_name,
    phone: form.phone,
    email: form.email,
    address: form.address,
    tax_number: form.tax_number,
    opening_balance_cents: toCents(form.opening_balance),
    notes: form.notes
  };
}

function partyToForm(party: Party): PartyForm {
  return {
    id: party.id,
    name: party.name,
    company_name: party.company_name ?? "",
    phone: party.phone ?? "",
    email: party.email ?? "",
    address: party.address ?? "",
    tax_number: party.tax_number ?? "",
    opening_balance: fromCents(party.opening_balance_cents),
    notes: party.notes ?? ""
  };
}
