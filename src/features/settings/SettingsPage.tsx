import { FormEvent, useEffect, useReducer, useState } from "react";
import {
  Alert,
  Avatar,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Divider,
  FormControlLabel,
  Paper,
  Stack,
  Switch,
  TextField,
  Typography
} from "@mui/material";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import ImageOutlinedIcon from "@mui/icons-material/ImageOutlined";

import { PageHeader } from "../../components/PageHeader";
import { LoadingState } from "../../components/feedback/PageState";
import { useAuth } from "../auth/AuthContext";
import { settingsApi } from "../../lib/api";
import { isCurrencyCode } from "../../lib/formatters";
import { normalizeError } from "../../lib/tauri";
import { LOGO_ACCEPT, readLogoFile } from "../../lib/imageFiles";
import type { CompanySettings } from "../../types/common";

// The "Clear All Data" wizard: step, credentials, confirmation text, and error
// are written together by each step transition, so they move as one value.
type ClearDataState = {
  step: "credentials" | "confirm" | null;
  adminEmail: string;
  adminPassword: string;
  confirmation: string;
  error: string | null;
};

type ClearDataAction =
  | { type: "started"; adminEmail: string }
  | { type: "credentialsChanged"; changes: Partial<Pick<ClearDataState, "adminEmail" | "adminPassword">> }
  | { type: "confirmationChanged"; value: string }
  | { type: "confirmationRequested" }
  | { type: "backToCredentials" }
  | { type: "dismissed" }
  | { type: "failed"; message: string }
  | { type: "completed" };

const idleClearData: ClearDataState = {
  step: null,
  adminEmail: "",
  adminPassword: "",
  confirmation: "",
  error: null
};

function clearDataReducer(state: ClearDataState, action: ClearDataAction): ClearDataState {
  switch (action.type) {
    case "started":
      // Always re-enter the wizard with a blank secret and confirmation.
      return {
        step: "credentials",
        adminEmail: action.adminEmail,
        adminPassword: "",
        confirmation: "",
        error: null
      };
    case "credentialsChanged":
      return { ...state, ...action.changes };
    case "confirmationChanged":
      return { ...state, confirmation: action.value };
    case "confirmationRequested":
      return { ...state, step: "confirm", error: null };
    case "backToCredentials":
      // Stepping back keeps the current error visible, as before.
      return { ...state, step: "credentials" };
    case "dismissed":
      return { ...state, step: null };
    case "failed":
      return { ...state, error: action.message };
    case "completed":
      return { ...state, step: null, adminPassword: "", confirmation: "", error: null };
  }
}

export function SettingsPage() {
  const queryClient = useQueryClient();
  const { admin } = useAuth();
  const { data, isLoading } = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get });
  const [form, setForm] = useState<CompanySettings | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [clearData, dispatchClearData] = useReducer(clearDataReducer, idleClearData);
  const { step: clearStep, adminEmail, adminPassword, confirmation, error: clearError } = clearData;
  const [clearMessage, setClearMessage] = useState<string | null>(null);

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
      quotation_prefix: value.quotation_prefix,
      allow_negative_stock: value.allow_negative_stock,
      backup_path: value.backup_path,
      default_tax_rate: value.default_tax_rate,
      default_profit_method: value.default_profit_method,
      deleted_retention_days: value.deleted_retention_days
    }),
    onSuccess: async () => {
      setError(null);
      await queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
    onError: (err) => setError(normalizeError(err).message)
  });
  const clearMutation = useMutation({
    mutationFn: () => settingsApi.clearAllData({
      admin_email: adminEmail,
      admin_password: adminPassword,
      confirmation
    }),
    onSuccess: async (result) => {
      dispatchClearData({ type: "completed" });
      setClearMessage(`${result.message} ${result.deleted_records} records were removed.`);
      await queryClient.invalidateQueries();
    },
    onError: (err) => dispatchClearData({ type: "failed", message: normalizeError(err).message })
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    if (!form) return;
    if (!isCurrencyCode(form.default_currency)) {
      setError("Default currency must be a three-letter code such as USD, SAR, AED, or EUR.");
      return;
    }
    if (window.confirm("Save settings changes?")) mutation.mutate(form);
  }

  if (isLoading || !form) return <LoadingState label="Loading settings" />;

  return (
    <Stack spacing={2}>
      <PageHeader title="Settings" description="Company information, invoice numbering, stock rules, tax, and backup path." />
      <Paper component="form" variant="outlined" onSubmit={submit} sx={{ p: 3, maxWidth: 820 }}>
        <Stack spacing={2}>
          {error ? <Alert severity="error">{error}</Alert> : null}
          <TextField label="Company name" required value={form.company_name} onChange={(e) => setForm({ ...form, company_name: e.target.value })} />
          <CompanyLogoEditor />
          <TextField label="Phone" value={form.phone ?? ""} onChange={(e) => setForm({ ...form, phone: e.target.value })} />
          <TextField label="Email" value={form.email ?? ""} onChange={(e) => setForm({ ...form, email: e.target.value })} />
          <TextField label="Address" multiline minRows={2} value={form.address ?? ""} onChange={(e) => setForm({ ...form, address: e.target.value })} />
          <TextField label="Tax number" value={form.tax_number ?? ""} onChange={(e) => setForm({ ...form, tax_number: e.target.value })} />
          <TextField
            label="Default currency"
            value={form.default_currency}
            error={!isCurrencyCode(form.default_currency)}
            helperText="Use a three-letter currency code, for example SAR or USD."
            slotProps={{ htmlInput: { maxLength: 3 } }}
            onChange={(e) => setForm({ ...form, default_currency: e.target.value.toUpperCase() })}
          />
          <TextField label="Sales invoice prefix" value={form.invoice_prefix_sales} onChange={(e) => setForm({ ...form, invoice_prefix_sales: e.target.value.toUpperCase() })} />
          <TextField label="Purchase invoice prefix" value={form.invoice_prefix_purchase} onChange={(e) => setForm({ ...form, invoice_prefix_purchase: e.target.value.toUpperCase() })} />
          <TextField label="Pro forma prefix" value={form.quotation_prefix} onChange={(e) => setForm({ ...form, quotation_prefix: e.target.value.toUpperCase() })} />
          <TextField label="Default tax value" type="number" value={form.default_tax_rate} onChange={(e) => setForm({ ...form, default_tax_rate: Number(e.target.value) })} />
          <TextField label="Profit calculation method" value={form.default_profit_method} onChange={(e) => setForm({ ...form, default_profit_method: e.target.value })} />
          <TextField label="Backup path" value={form.backup_path ?? ""} onChange={(e) => setForm({ ...form, backup_path: e.target.value })} />
          <FormControlLabel control={<Switch checked={form.allow_negative_stock} onChange={(e) => setForm({ ...form, allow_negative_stock: e.target.checked })} />} label="Allow negative stock" />
          <Button type="submit" variant="contained" disabled={mutation.isPending}>Save settings</Button>
        </Stack>
      </Paper>
      {admin?.role === "admin" ? (
        <Paper variant="outlined" sx={{ p: 3, maxWidth: 820, borderColor: "error.main" }}>
          <Stack spacing={2}>
            <Typography variant="h6" color="error">Danger zone</Typography>
            {clearMessage ? <Alert severity="success">{clearMessage}</Alert> : null}
            <Typography variant="body2" color="text.secondary">
              Clear All Data permanently removes products, parties, invoices, payments, inventory,
              expenses, backup history, and business logs. Your administrator account and
              company settings remain.
            </Typography>
            <Divider />
            <Button
              color="error"
              variant="contained"
              sx={{ alignSelf: "flex-start" }}
              onClick={() => dispatchClearData({ type: "started", adminEmail: admin.email })}
            >
              Clear All Data
            </Button>
          </Stack>
        </Paper>
      ) : null}

      <Dialog
        open={clearStep === "credentials"}
        onClose={() => !clearMutation.isPending && dispatchClearData({ type: "dismissed" })}
        fullWidth
        maxWidth="sm"
      >
        <DialogTitle>Verify administrator</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ pt: 1 }}>
            <Alert severity="error">
              This reset is irreversible. Existing backups and records listed above will be
              removed from the application.
            </Alert>
            {clearError ? <Alert severity="error">{clearError}</Alert> : null}
            <TextField
              label="Admin email"
              type="email"
              required
              value={adminEmail}
              onChange={(event) => dispatchClearData({
                type: "credentialsChanged",
                changes: { adminEmail: event.target.value }
              })}
            />
            <TextField
              label="Admin password"
              type="password"
              required
              autoComplete="current-password"
              value={adminPassword}
              onChange={(event) => dispatchClearData({
                type: "credentialsChanged",
                changes: { adminPassword: event.target.value }
              })}
            />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => dispatchClearData({ type: "dismissed" })}>Cancel</Button>
          <Button
            color="error"
            variant="contained"
            disabled={!adminEmail.trim() || !adminPassword}
            onClick={() => dispatchClearData({ type: "confirmationRequested" })}
          >
            Continue
          </Button>
        </DialogActions>
      </Dialog>

      <Dialog
        open={clearStep === "confirm"}
        onClose={() => !clearMutation.isPending && dispatchClearData({ type: "dismissed" })}
        fullWidth
        maxWidth="sm"
      >
        <DialogTitle>Final confirmation</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ pt: 1 }}>
            <Alert severity="error">
              All operational data will be permanently deleted in one database transaction.
              This cannot be undone.
            </Alert>
            {clearError ? <Alert severity="error">{clearError}</Alert> : null}
            <Typography variant="body2">
              Type <strong>CLEAR ALL DATA</strong> to continue.
            </Typography>
            <TextField
              label="Confirmation"
              value={confirmation}
              onChange={(event) => dispatchClearData({
                type: "confirmationChanged",
                value: event.target.value
              })}
              autoFocus
            />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button disabled={clearMutation.isPending} onClick={() => dispatchClearData({ type: "backToCredentials" })}>Back</Button>
          <Button
            color="error"
            variant="contained"
            disabled={confirmation !== "CLEAR ALL DATA" || clearMutation.isPending}
            onClick={() => clearMutation.mutate()}
          >
            {clearMutation.isPending ? "Clearing…" : "Permanently clear data"}
          </Button>
        </DialogActions>
      </Dialog>
    </Stack>
  );
}

function CompanyLogoEditor() {
  const queryClient = useQueryClient();
  const [error, setError] = useState<string | null>(null);
  const [broken, setBroken] = useState(false);
  const { data: logo, isLoading } = useQuery({
    queryKey: ["settings", "company-logo"],
    queryFn: settingsApi.getLogo
  });
  const uploadMutation = useMutation({
    mutationFn: async (file: File) => settingsApi.saveLogo(await readLogoFile(file)),
    onSuccess: async () => {
      setError(null);
      setBroken(false);
      await queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
    onError: (reason) => setError(normalizeError(reason).message)
  });
  const removeMutation = useMutation({
    mutationFn: settingsApi.removeLogo,
    onSuccess: async () => {
      setError(null);
      setBroken(false);
      await queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
    onError: (reason) => setError(normalizeError(reason).message)
  });

  return (
    <Paper variant="outlined" sx={{ p: 2 }}>
      <Stack direction={{ xs: "column", sm: "row" }} spacing={2} alignItems={{ sm: "center" }}>
        <Avatar
          variant="rounded"
          src={!broken ? logo ?? undefined : undefined}
          imgProps={{ onError: () => setBroken(true), style: { objectFit: "contain" } }}
          sx={{ width: 94, height: 94, bgcolor: "#eef3f4", color: "primary.main" }}
        >
          <ImageOutlinedIcon />
        </Avatar>
        <Box sx={{ flex: 1 }}>
          <Typography variant="subtitle2">Company logo</Typography>
          <Typography variant="caption" color="text.secondary">
            Used on professional pro forma printouts. PNG, JPEG, or WebP up to 2 MB.
          </Typography>
          {error ? <Alert severity="error" sx={{ mt: 1 }}>{error}</Alert> : null}
          <Stack direction="row" spacing={1} sx={{ mt: 1 }}>
            <Button component="label" variant="outlined" disabled={isLoading || uploadMutation.isPending}>
              {logo ? "Change logo" : "Upload logo"}
              <input
                hidden
                type="file"
                accept={LOGO_ACCEPT}
                onChange={(event) => {
                  const file = event.target.files?.[0];
                  event.target.value = "";
                  if (file) uploadMutation.mutate(file);
                }}
              />
            </Button>
            {logo ? <Button color="error" disabled={removeMutation.isPending} onClick={() => removeMutation.mutate()}>Remove</Button> : null}
          </Stack>
        </Box>
      </Stack>
    </Paper>
  );
}
