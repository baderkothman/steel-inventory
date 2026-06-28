import { FormEvent, useEffect, useState } from "react";
import { Alert, Button, Paper, Stack, TextField, FormControlLabel, Switch } from "@mui/material";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { PageHeader } from "../../components/PageHeader";
import { LoadingState } from "../../components/feedback/PageState";
import { settingsApi } from "../../lib/api";
import { normalizeError } from "../../lib/tauri";
import type { CompanySettings } from "../../types/common";

export function SettingsPage() {
  const queryClient = useQueryClient();
  const { data, isLoading } = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get });
  const [form, setForm] = useState<CompanySettings | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (data) setForm(data);
  }, [data]);

  const mutation = useMutation({
    mutationFn: (value: CompanySettings) => settingsApi.update({
      company_name: value.company_name,
      phone: value.phone,
      email: value.email,
      address: value.address,
      tax_number: value.tax_number,
      default_currency: value.default_currency,
      invoice_prefix_sales: value.invoice_prefix_sales,
      invoice_prefix_purchase: value.invoice_prefix_purchase,
      allow_negative_stock: value.allow_negative_stock,
      backup_path: value.backup_path,
      default_tax_rate: value.default_tax_rate,
      default_profit_method: value.default_profit_method
    }),
    onSuccess: async () => {
      setError(null);
      await queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
    onError: (err) => setError(normalizeError(err).message)
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    if (form && window.confirm("Save settings changes?")) mutation.mutate(form);
  }

  if (isLoading || !form) return <LoadingState label="Loading settings" />;

  return (
    <Stack spacing={2}>
      <PageHeader title="Settings" description="Company information, invoice numbering, stock rules, tax, and backup path." />
      <Paper component="form" variant="outlined" onSubmit={submit} sx={{ p: 3, maxWidth: 820 }}>
        <Stack spacing={2}>
          {error ? <Alert severity="error">{error}</Alert> : null}
          <TextField label="Company name" required value={form.company_name} onChange={(e) => setForm({ ...form, company_name: e.target.value })} />
          <TextField label="Phone" value={form.phone ?? ""} onChange={(e) => setForm({ ...form, phone: e.target.value })} />
          <TextField label="Email" value={form.email ?? ""} onChange={(e) => setForm({ ...form, email: e.target.value })} />
          <TextField label="Address" multiline minRows={2} value={form.address ?? ""} onChange={(e) => setForm({ ...form, address: e.target.value })} />
          <TextField label="Tax number" value={form.tax_number ?? ""} onChange={(e) => setForm({ ...form, tax_number: e.target.value })} />
          <TextField label="Default currency" value={form.default_currency} onChange={(e) => setForm({ ...form, default_currency: e.target.value.toUpperCase() })} />
          <TextField label="Sales invoice prefix" value={form.invoice_prefix_sales} onChange={(e) => setForm({ ...form, invoice_prefix_sales: e.target.value.toUpperCase() })} />
          <TextField label="Purchase invoice prefix" value={form.invoice_prefix_purchase} onChange={(e) => setForm({ ...form, invoice_prefix_purchase: e.target.value.toUpperCase() })} />
          <TextField label="Default tax value" type="number" value={form.default_tax_rate} onChange={(e) => setForm({ ...form, default_tax_rate: Number(e.target.value) })} />
          <TextField label="Profit calculation method" value={form.default_profit_method} onChange={(e) => setForm({ ...form, default_profit_method: e.target.value })} />
          <TextField label="Backup path" value={form.backup_path ?? ""} onChange={(e) => setForm({ ...form, backup_path: e.target.value })} />
          <FormControlLabel control={<Switch checked={form.allow_negative_stock} onChange={(e) => setForm({ ...form, allow_negative_stock: e.target.checked })} />} label="Allow negative stock" />
          <Button type="submit" variant="contained" disabled={mutation.isPending}>Save settings</Button>
        </Stack>
      </Paper>
    </Stack>
  );
}
