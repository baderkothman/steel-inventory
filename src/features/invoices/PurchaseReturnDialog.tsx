import { FormEvent, useMemo, useReducer, useState } from "react";
import {
  Alert,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
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
import DeleteOutlineIcon from "@mui/icons-material/DeleteOutline";
import EditOutlinedIcon from "@mui/icons-material/EditOutlined";
import PrintOutlinedIcon from "@mui/icons-material/PrintOutlined";
import RestoreOutlinedIcon from "@mui/icons-material/RestoreOutlined";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { LoadingState } from "../../components/feedback/PageState";
import { PrintDialog } from "../../components/print/PrintDialog";
import { RowActionsMenu } from "../../components/table/EnterpriseTable";
import { StatusBadge } from "../../components/table/StatusBadge";
import { purchaseApi } from "../../lib/api";
import { money, quantity, today } from "../../lib/formatters";
import { normalizeError } from "../../lib/tauri";
import type {
  PurchaseReturnContext,
  PurchaseReturnDetail,
  PurchaseReturnItemPayload,
  PurchaseReturnableItem
} from "../../types/invoice";

type Props = {
  invoiceId: number | null;
  currency?: string;
  onClose: () => void;
};

// The return draft: every field below is written together by the reset and
// edit transitions, so it is one state value rather than six.
type ReturnDraft = {
  returnDate: string;
  reason: string;
  notes: string;
  quantities: Record<number, string>;
  editingId: number | null;
  idempotencyKey: string;
};

type ReturnDraftAction =
  | { type: "detailsChanged"; changes: Partial<Pick<ReturnDraft, "returnDate" | "reason" | "notes">> }
  | { type: "quantityChanged"; purchaseInvoiceItemId: number; value: string }
  | { type: "editStarted"; detail: PurchaseReturnDetail }
  | { type: "reset" };

function createReturnDraft(): ReturnDraft {
  return {
    returnDate: today(),
    reason: "",
    notes: "",
    quantities: {},
    editingId: null,
    idempotencyKey: newIdempotencyKey()
  };
}

function returnDraftReducer(draft: ReturnDraft, action: ReturnDraftAction): ReturnDraft {
  switch (action.type) {
    case "detailsChanged":
      return { ...draft, ...action.changes };
    case "quantityChanged":
      return {
        ...draft,
        quantities: {
          ...draft.quantities,
          [action.purchaseInvoiceItemId]: action.value
        }
      };
    case "editStarted":
      // Loading an existing return keeps the current idempotency key: it only
      // guards the create call, which editing does not use.
      return {
        ...draft,
        editingId: action.detail.return_record.id,
        returnDate: action.detail.return_record.return_date,
        reason: action.detail.return_record.reason ?? "",
        notes: action.detail.return_record.notes ?? "",
        quantities: Object.fromEntries(
          action.detail.items.map((item) => [
            item.purchase_invoice_item_id,
            String(item.quantity)
          ])
        )
      };
    case "reset":
      return createReturnDraft();
  }
}

export function PurchaseReturnDialog({ invoiceId, currency, onClose }: Props) {
  // The Dialog itself stays mounted so MUI can play its close transition, while the
  // workspace is keyed by invoice: opening a different invoice (or reopening after a
  // close) remounts it, which resets the form, the edit selection, and the
  // idempotency key without an effect that syncs state to the prop.
  return (
    <Dialog open={invoiceId !== null} onClose={onClose} fullWidth maxWidth="lg">
      <PurchaseReturnWorkspace
        key={invoiceId}
        invoiceId={invoiceId}
        currency={currency}
        onClose={onClose}
      />
    </Dialog>
  );
}

function PurchaseReturnWorkspace({ invoiceId, currency, onClose }: Props) {
  const queryClient = useQueryClient();
  const [draft, dispatch] = useReducer(returnDraftReducer, undefined, createReturnDraft);
  const { returnDate, reason, notes, quantities, editingId, idempotencyKey } = draft;
  const [cancelId, setCancelId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [printHtml, setPrintHtml] = useState("");

  const { data: context, isLoading } = useQuery({
    queryKey: ["purchase-return-context", invoiceId],
    queryFn: () => purchaseApi.returnContext(invoiceId!),
    enabled: invoiceId !== null
  });
  const editingReturn = context?.returns.find(
    (item) => item.return_record.id === editingId
  ) ?? null;
  const selectedItems = useMemo(
    () => Object.entries(quantities).reduce<PurchaseReturnItemPayload[]>(
      (items, [purchaseInvoiceItemId, value]) => {
        const returnQuantity = Number(value);
        if (Number.isFinite(returnQuantity) && returnQuantity > 0) {
          items.push({
            purchase_invoice_item_id: Number(purchaseInvoiceItemId),
            quantity: returnQuantity
          });
        }
        return items;
      },
      []
    ),
    [quantities]
  );
  const merchandiseTotal = useMemo(
    () => selectedItems.reduce((sum, item) => {
      const original = context?.items.find(
        (candidate) =>
          candidate.purchase_invoice_item_id === item.purchase_invoice_item_id
      );
      return sum + item.quantity * (original?.unit_cost_cents ?? 0);
    }, 0),
    [context?.items, selectedItems]
  );

  const saveMutation = useMutation({
    mutationFn: (items: PurchaseReturnItemPayload[]) => {
      const common = {
        return_date: returnDate,
        reason: reason || null,
        notes: notes || null,
        items
      };
      return editingId
        ? purchaseApi.updateReturn(editingId, common)
        : purchaseApi.createReturn({
            ...common,
            purchase_invoice_id: invoiceId!,
            idempotency_key: idempotencyKey
          });
    },
    onSuccess: async () => {
      resetForm();
      setError(null);
      await queryClient.invalidateQueries();
    },
    onError: (failure) => setError(normalizeError(failure).message)
  });
  const cancelMutation = useMutation({
    mutationFn: (id: number) => purchaseApi.cancelReturn(id),
    onSuccess: async () => {
      setCancelId(null);
      setError(null);
      if (editingId === cancelId) resetForm();
      await queryClient.invalidateQueries();
    },
    onError: (failure) => setError(normalizeError(failure).message)
  });
  const restoreMutation = useMutation({
    mutationFn: (id: number) => purchaseApi.restoreReturn(id),
    onSuccess: async () => {
      setError(null);
      await queryClient.invalidateQueries();
    },
    onError: (failure) => setError(normalizeError(failure).message)
  });

  function resetForm() {
    dispatch({ type: "reset" });
  }

  function startEdit(detail: PurchaseReturnDetail) {
    dispatch({ type: "editStarted", detail });
    setError(null);
  }

  function maximumFor(
    purchaseInvoiceItemId: number,
    returnableQuantity: number
  ) {
    const ownQuantity = editingReturn?.items.find(
      (item) => item.purchase_invoice_item_id === purchaseInvoiceItemId
    )?.quantity ?? 0;
    return returnableQuantity + ownQuantity;
  }

  function submit(event: FormEvent) {
    event.preventDefault();
    if (!selectedItems.length) {
      setError("Enter a return quantity for at least one product.");
      return;
    }
    saveMutation.mutate(selectedItems);
  }

  async function printReturn(id: number) {
    try {
      setPrintHtml(await purchaseApi.printReturn(id));
    } catch (failure) {
      setError(normalizeError(failure).message);
    }
  }

  return (
    <>
      <DialogTitle>
        Purchase returns — {context?.invoice.invoice_number ?? ""}
      </DialogTitle>
      <DialogContent>
        {isLoading ? (
          <LoadingState label="Loading returnable products" />
        ) : (
          <Stack spacing={2.5} sx={{ pt: 1 }}>
            {error ? <Alert severity="error">{error}</Alert> : null}
            <InvoiceSummaryGrid invoice={context?.invoice} currency={currency} />

            <Stack
              component="form"
              id="purchase-return-form"
              onSubmit={submit}
              spacing={2}
            >
              <Typography variant="subtitle1" fontWeight={700}>
                {editingId ? "Edit purchase return" : "Create purchase return"}
              </Typography>
              <Box
                sx={{
                  display: "grid",
                  gridTemplateColumns: { xs: "1fr", md: "180px 1fr 1fr" },
                  gap: 1.5
                }}
              >
                <TextField
                  label="Return date"
                  type="date"
                  required
                  value={returnDate}
                  onChange={(event) => dispatch({
                    type: "detailsChanged",
                    changes: { returnDate: event.target.value }
                  })}
                />
                <TextField
                  label="Reason"
                  placeholder="Damaged goods"
                  value={reason}
                  onChange={(event) => dispatch({
                    type: "detailsChanged",
                    changes: { reason: event.target.value }
                  })}
                />
                <TextField
                  label="Notes"
                  value={notes}
                  onChange={(event) => dispatch({
                    type: "detailsChanged",
                    changes: { notes: event.target.value }
                  })}
                />
              </Box>

              <ReturnableItemsTable
                items={context?.items}
                quantities={quantities}
                currency={currency}
                maximumFor={maximumFor}
                onQuantityChange={(purchaseInvoiceItemId, value) => dispatch({
                  type: "quantityChanged",
                  purchaseInvoiceItemId,
                  value
                })}
              />

              <Stack direction="row" justifyContent="space-between" alignItems="center">
                <Typography variant="body2" color="text.secondary">
                  Merchandise subtotal: {money(merchandiseTotal, currency)}.
                  Invoice discount, tax, and shipping credits are prorated by the server.
                </Typography>
                <Stack direction="row" spacing={1}>
                  {editingId ? <Button onClick={resetForm}>Stop editing</Button> : null}
                  <Button
                    type="submit"
                    variant="contained"
                    disabled={saveMutation.isPending || !selectedItems.length}
                  >
                    {editingId ? "Save return changes" : "Confirm return"}
                  </Button>
                </Stack>
              </Stack>
            </Stack>

            <ReturnHistoryTable
              returns={context?.returns}
              currency={currency}
              onPrint={(id) => void printReturn(id)}
              onEdit={startEdit}
              onCancel={setCancelId}
              onRestore={(id) => restoreMutation.mutate(id)}
            />
          </Stack>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Close</Button>
      </DialogActions>

      <ConfirmDialog
        open={cancelId !== null}
        title="Cancel purchase return"
        message="This reverses the return credit, restores its inventory quantities, and keeps the return in history."
        confirmLabel="Cancel return"
        error={null}
        loading={cancelMutation.isPending}
        onClose={() => setCancelId(null)}
        onConfirm={() => cancelId !== null && cancelMutation.mutate(cancelId)}
      />
      <PrintDialog
        open={Boolean(printHtml)}
        html={printHtml}
        onClose={() => setPrintHtml("")}
      />
    </>
  );
}

function InvoiceSummaryGrid({
  invoice,
  currency
}: {
  invoice: PurchaseReturnContext["invoice"] | undefined;
  currency?: string;
}) {
  return (
    <Box
      sx={{
        display: "grid",
        gridTemplateColumns: { xs: "1fr", sm: "repeat(4, 1fr)" },
        gap: 1.5
      }}
    >
      <Summary label="Original total" value={invoice?.total_cents ?? 0} currency={currency} />
      <Summary label="Returned" value={invoice?.returned_cents ?? 0} currency={currency} />
      <Summary label="Net purchase" value={invoice?.net_total_cents ?? 0} currency={currency} />
      <Summary
        label="Invoice balance due"
        value={invoice?.remaining_cents ?? 0}
        currency={currency}
      />
    </Box>
  );
}

function ReturnableItemsTable({
  items,
  quantities,
  currency,
  maximumFor,
  onQuantityChange
}: {
  items: PurchaseReturnableItem[] | undefined;
  quantities: Record<number, string>;
  currency?: string;
  maximumFor: (purchaseInvoiceItemId: number, returnableQuantity: number) => number;
  onQuantityChange: (purchaseInvoiceItemId: number, value: string) => void;
}) {
  return (
    <TableContainer sx={{ maxHeight: 320, border: "1px solid", borderColor: "divider", borderRadius: 1 }}>
      <Table size="small" stickyHeader aria-label="Returnable purchase products">
        <TableHead>
          <TableRow>
            <TableCell>Product</TableCell>
            <TableCell align="right">Purchased</TableCell>
            <TableCell align="right">Already returned</TableCell>
            <TableCell align="right">Returnable</TableCell>
            <TableCell align="right">Unit cost</TableCell>
            <TableCell align="right">Return quantity</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {items?.map((item) => {
            const maximum = maximumFor(
              item.purchase_invoice_item_id,
              item.returnable_quantity
            );
            return (
              <TableRow key={item.purchase_invoice_item_id}>
                <TableCell>{item.sku} — {item.product_name}</TableCell>
                <TableCell align="right">{quantity(item.purchased_quantity)}</TableCell>
                <TableCell align="right">{quantity(item.returned_quantity)}</TableCell>
                <TableCell align="right">{quantity(maximum)}</TableCell>
                <TableCell align="right">
                  <MoneyText value={item.unit_cost_cents} currency={currency} />
                </TableCell>
                <TableCell align="right">
                  <TextField
                    type="number"
                    value={quantities[item.purchase_invoice_item_id] ?? ""}
                    onChange={(event) => onQuantityChange(
                      item.purchase_invoice_item_id,
                      event.target.value
                    )}
                    slotProps={{
                      htmlInput: {
                        min: 0,
                        max: maximum,
                        step: "any",
                        "aria-label": `Return quantity for ${item.product_name}`
                      }
                    }}
                    sx={{ width: 120 }}
                  />
                </TableCell>
              </TableRow>
            );
          })}
        </TableBody>
      </Table>
    </TableContainer>
  );
}

function ReturnHistoryTable({
  returns,
  currency,
  onPrint,
  onEdit,
  onCancel,
  onRestore
}: {
  returns: PurchaseReturnDetail[] | undefined;
  currency?: string;
  onPrint: (id: number) => void;
  onEdit: (detail: PurchaseReturnDetail) => void;
  onCancel: (id: number) => void;
  onRestore: (id: number) => void;
}) {
  return (
    <Stack spacing={1}>
      <Typography variant="subtitle1" fontWeight={700}>
        Return history
      </Typography>
      <TableContainer sx={{ border: "1px solid", borderColor: "divider", borderRadius: 1 }}>
        <Table size="small" aria-label="Purchase return history">
          <TableHead>
            <TableRow>
              <TableCell>Return</TableCell>
              <TableCell>Date</TableCell>
              <TableCell>Reason</TableCell>
              <TableCell align="right">Quantity</TableCell>
              <TableCell align="right">Total credit</TableCell>
              <TableCell>Status</TableCell>
              <TableCell />
            </TableRow>
          </TableHead>
          <TableBody>
            {returns?.length ? returns.map((detail) => (
              <TableRow key={detail.return_record.id}>
                <TableCell>{detail.return_record.return_number}</TableCell>
                <TableCell>{detail.return_record.return_date}</TableCell>
                <TableCell>{detail.return_record.reason ?? "—"}</TableCell>
                <TableCell align="right">
                  {quantity(detail.items.reduce((sum, item) => sum + item.quantity, 0))}
                </TableCell>
                <TableCell align="right">
                  <MoneyText value={detail.return_record.total_cents} currency={currency} />
                </TableCell>
                <TableCell><StatusBadge value={detail.return_record.status} /></TableCell>
                <TableCell align="right">
                  <RowActionsMenu
                    row={detail}
                    actions={[
                      {
                        label: "Print return",
                        icon: <PrintOutlinedIcon fontSize="small" />,
                        onClick: () => onPrint(detail.return_record.id)
                      },
                      ...(detail.return_record.status === "active"
                        ? [
                            {
                              label: "Edit return",
                              icon: <EditOutlinedIcon fontSize="small" />,
                              onClick: () => onEdit(detail)
                            },
                            {
                              label: "Cancel return",
                              icon: <DeleteOutlineIcon fontSize="small" />,
                              destructive: true,
                              onClick: () => onCancel(detail.return_record.id)
                            }
                          ]
                        : [
                            {
                              label: "Restore return",
                              icon: <RestoreOutlinedIcon fontSize="small" />,
                              onClick: () => onRestore(detail.return_record.id)
                            }
                          ])
                    ]}
                  />
                </TableCell>
              </TableRow>
            )) : (
              <TableRow>
                <TableCell colSpan={7}>No purchase returns have been recorded.</TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </TableContainer>
    </Stack>
  );
}

function Summary({
  label,
  value,
  currency
}: {
  label: string;
  value: number;
  currency?: string;
}) {
  return (
    <Box sx={{ borderBottom: "1px solid", borderColor: "divider", pb: 1 }}>
      <Typography variant="body2" color="text.secondary">{label}</Typography>
      <Typography variant="h6" sx={{ mt: 0.25, fontVariantNumeric: "tabular-nums" }}>
        <MoneyText value={value} currency={currency} />
      </Typography>
    </Box>
  );
}

function newIdempotencyKey() {
  return globalThis.crypto?.randomUUID?.()
    ?? `purchase-return-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}
