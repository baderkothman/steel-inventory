import { FormEvent, useState } from "react";
import {
  Alert,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Drawer,
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
import ArchiveIcon from "@mui/icons-material/Archive";
import EditIcon from "@mui/icons-material/Edit";
import ReceiptLongIcon from "@mui/icons-material/ReceiptLong";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { PageHeader } from "../../components/PageHeader";
import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { EmptyState, LoadingState } from "../../components/feedback/PageState";
import { customerApi, supplierApi } from "../../lib/api";
import { fromCents, toCents } from "../../lib/formatters";
import { normalizeError } from "../../lib/tauri";
import type { Party, PartyPayload } from "../../types/party";

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
  const [search, setSearch] = useState("");
  const [form, setForm] = useState<PartyForm | null>(null);
  const [archiveId, setArchiveId] = useState<number | null>(null);
  const [statementParty, setStatementParty] = useState<Party | null>(null);
  const [error, setError] = useState<string | null>(null);

  const { data = [], isLoading } = useQuery({
    queryKey: [kind, search],
    queryFn: () => api.list({ search: search || null, active_only: false })
  });

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

  const archiveMutation = useMutation({
    mutationFn: (id: number) => api.archive(id),
    onSuccess: async () => {
      setArchiveId(null);
      await queryClient.invalidateQueries({ queryKey: [kind] });
    }
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
        actions={<Button startIcon={<AddIcon />} variant="contained" onClick={() => setForm(blankForm)}>Add {kind}</Button>}
      />

      <Paper variant="outlined" sx={{ p: 2 }}>
        <TextField label="Search" value={search} onChange={(event) => setSearch(event.target.value)} sx={{ minWidth: 280 }} />
      </Paper>

      <Paper variant="outlined">
        {data.length === 0 ? <EmptyState label={`No ${kind}s found.`} /> : (
          <Table size="small">
            <TableHead><TableRow><TableCell>Name</TableCell><TableCell>Company</TableCell><TableCell>Phone</TableCell><TableCell align="right">Balance</TableCell><TableCell>Status</TableCell><TableCell align="right">Actions</TableCell></TableRow></TableHead>
            <TableBody>
              {data.map((party) => (
                <TableRow key={party.id} hover>
                  <TableCell>{party.name}</TableCell>
                  <TableCell>{party.company_name}</TableCell>
                  <TableCell>{party.phone}</TableCell>
                  <TableCell align="right"><MoneyText value={party.balance_cents} /></TableCell>
                  <TableCell>{party.is_active ? "Active" : "Archived"}</TableCell>
                  <TableCell align="right">
                    <Button size="small" startIcon={<EditIcon />} onClick={() => setForm(partyToForm(party))}>Edit</Button>
                    <Button size="small" startIcon={<ReceiptLongIcon />} onClick={() => setStatementParty(party)}>Statement</Button>
                    <Button size="small" color="warning" startIcon={<ArchiveIcon />} disabled={!party.is_active} onClick={() => setArchiveId(party.id)}>Archive</Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </Paper>

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
        open={archiveId !== null}
        title={`Archive ${kind}`}
        message={`Archived ${kind}s stay available in invoice history and statements.`}
        confirmLabel="Archive"
        onClose={() => setArchiveId(null)}
        onConfirm={() => archiveId && archiveMutation.mutate(archiveId)}
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

  return (
    <Drawer anchor="right" open={Boolean(party)} onClose={onClose}>
      <Box sx={{ width: 680, p: 3 }}>
        <PageHeader title="Statement" description={party?.name} />
        {isLoading ? <LoadingState /> : data.length === 0 ? <EmptyState label="No statement rows." /> : (
          <Table size="small">
            <TableHead><TableRow><TableCell>Date</TableCell><TableCell>Type</TableCell><TableCell>Reference</TableCell><TableCell align="right">Debit</TableCell><TableCell align="right">Credit</TableCell><TableCell align="right">Balance</TableCell></TableRow></TableHead>
            <TableBody>{data.map((row, index) => <TableRow key={`${row.reference}-${index}`}><TableCell>{row.date}</TableCell><TableCell>{row.entry_type}</TableCell><TableCell>{row.reference}</TableCell><TableCell align="right"><MoneyText value={row.debit_cents} /></TableCell><TableCell align="right"><MoneyText value={row.credit_cents} /></TableCell><TableCell align="right"><MoneyText value={row.balance_cents} /></TableCell></TableRow>)}</TableBody>
          </Table>
        )}
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
