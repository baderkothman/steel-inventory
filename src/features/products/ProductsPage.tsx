import { FormEvent, useMemo, useState } from "react";
import {
  Alert,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Divider,
  Drawer,
  MenuItem,
  Paper,
  Stack,
  TextField,
  Typography
} from "@mui/material";
import AddIcon from "@mui/icons-material/Add";
import DeleteIcon from "@mui/icons-material/Delete";
import EditIcon from "@mui/icons-material/Edit";
import HistoryIcon from "@mui/icons-material/History";
import TuneIcon from "@mui/icons-material/Tune";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { PageHeader } from "../../components/PageHeader";
import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { LoadingState } from "../../components/feedback/PageState";
import { EnterpriseTable, type TableColumn } from "../../components/table/EnterpriseTable";
import { StatusBadge } from "../../components/table/StatusBadge";
import { categoryApi, productApi, settingsApi, supplierApi } from "../../lib/api";
import { finishes, materials, productTypes, shapes, units } from "../../lib/constants";
import { fromCents, money, quantity, toCents } from "../../lib/formatters";
import { normalizeError } from "../../lib/tauri";
import type { InventoryTransaction, Product, ProductPayload } from "../../types/product";

type ProductForm = Omit<ProductPayload, "cost_price_cents" | "selling_price_cents" | "wholesale_price_cents"> & {
  id?: number;
  cost_price: string;
  selling_price: string;
  wholesale_price: string;
};

const blankProduct: ProductForm = {
  sku: "",
  category_id: 0,
  supplier_id: null,
  location: "",
  name: "",
  product_type: "pipe",
  material: "galvanized steel",
  shape: "square",
  finish: "galvanized",
  size_label: "",
  width_mm: null,
  height_mm: null,
  diameter_mm: null,
  thickness_mm: null,
  length_mm: null,
  unit: "piece",
  description: "",
  cost_price: "0.00",
  selling_price: "0.00",
  wholesale_price: "0.00",
  minimum_quantity: 0,
  initial_quantity: 0
};

export function ProductsPage() {
  const queryClient = useQueryClient();
  const [categoryId, setCategoryId] = useState<number | "">("");
  const [supplierId, setSupplierId] = useState<number | "">("");
  const [form, setForm] = useState<ProductForm | null>(null);
  const [deleteId, setDeleteId] = useState<number | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [movementProduct, setMovementProduct] = useState<Product | null>(null);
  const [adjustProduct, setAdjustProduct] = useState<Product | null>(null);
  const [error, setError] = useState<string | null>(null);

  const { data: categories = [] } = useQuery({ queryKey: ["categories"], queryFn: categoryApi.list });
  const { data: suppliers = [] } = useQuery({ queryKey: ["suppliers"], queryFn: () => supplierApi.list({ active_only: true }) });
  const { data: settings } = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get });
  const { data: products = [], isLoading } = useQuery({
    queryKey: ["products", categoryId, supplierId],
    queryFn: () =>
      productApi.list({
        category_id: categoryId || null,
        supplier_id: supplierId || null,
        active_only: true
      })
  });

  const activeCategories = useMemo(() => categories.filter((category) => category.is_active), [categories]);
  const activeProducts = useMemo(() => products.filter((product) => product.is_active), [products]);
  const columns = useMemo<TableColumn<Product>[]>(() => [
    { id: "sku", label: "SKU", value: (row) => row.sku, minWidth: 140 },
    { id: "product", label: "Product", value: (row) => row.name, minWidth: 190 },
    { id: "supplier", label: "Supplier", value: (row) => row.supplier_name, minWidth: 160 },
    { id: "size", label: "Size", value: (row) => row.size_label || "—", minWidth: 100 },
    { id: "thickness", label: "Thickness", value: (row) => row.thickness_mm ? `${quantity(row.thickness_mm)} mm` : "—", minWidth: 100 },
    { id: "stock", label: "Stock remaining", value: (row) => quantity(row.current_quantity), align: "right", minWidth: 130 },
    { id: "unit_cost", label: "Unit cost", value: (row) => money(row.cost_price_cents, settings?.default_currency), render: (row) => <MoneyText value={row.cost_price_cents} currency={settings?.default_currency} />, align: "right", minWidth: 120 },
    { id: "total_cost", label: "Total cost", value: (row) => money(Math.round(row.current_quantity * row.cost_price_cents), settings?.default_currency), render: (row) => <MoneyText value={Math.round(row.current_quantity * row.cost_price_cents)} currency={settings?.default_currency} />, align: "right", minWidth: 120 }
  ], [settings?.default_currency]);

  const saveMutation = useMutation({
    mutationFn: (value: ProductForm) => {
      const payload = formToPayload(value);
      return value.id ? productApi.update(value.id, payload) : productApi.create(payload);
    },
    onSuccess: async () => {
      setForm(null);
      setError(null);
      await queryClient.invalidateQueries({ queryKey: ["products"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    },
    onError: (err) => setError(normalizeError(err).message)
  });

  const deleteMutation = useMutation({
    mutationFn: (id: number) => productApi.archive(id),
    onSuccess: async () => {
      setDeleteId(null);
      setDeleteError(null);
      await queryClient.invalidateQueries({ queryKey: ["products"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    },
    onError: (err) => setDeleteError(normalizeError(err).message)
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    if (form) {
      if (!form.category_id && activeCategories[0]) {
        saveMutation.mutate({ ...form, category_id: activeCategories[0].id });
      } else {
        saveMutation.mutate(form);
      }
    }
  }

  if (isLoading) {
    return <LoadingState label="Loading products" />;
  }

  return (
    <Stack spacing={2}>
      <PageHeader
        title="Products"
        description="Manage SKUs, steel dimensions, prices, stock levels, and movement history."
        actions={
          <Button
            startIcon={<AddIcon />}
            variant="contained"
            onClick={() => setForm({ ...blankProduct, category_id: activeCategories[0]?.id ?? 0 })}
          >
            Add product
          </Button>
        }
      />
      {deleteError && deleteId === null ? <Alert severity="error">{deleteError}</Alert> : null}

      <Paper variant="outlined" sx={{ p: 2 }}>
        <Stack direction={{ xs: "column", sm: "row" }} spacing={1.5}>
          <TextField
            select
            label="Category"
            value={categoryId}
            onChange={(event) => setCategoryId(event.target.value ? Number(event.target.value) : "")}
            sx={{ minWidth: 240 }}
          >
            <MenuItem value="">All categories</MenuItem>
            {activeCategories.map((category) => (
              <MenuItem key={category.id} value={category.id}>
                {category.name}
              </MenuItem>
            ))}
          </TextField>
          <TextField
            select
            label="Supplier"
            value={supplierId}
            onChange={(event) => setSupplierId(event.target.value ? Number(event.target.value) : "")}
            sx={{ minWidth: 220 }}
          >
            <MenuItem value="">All suppliers</MenuItem>
            {suppliers.map((supplier) => (
              <MenuItem key={supplier.id} value={supplier.id}>
                {supplier.name}
              </MenuItem>
            ))}
          </TextField>
        </Stack>
      </Paper>

      <EnterpriseTable
        title="Product Stock & Cost"
        rows={activeProducts}
        columns={columns}
        rowId={(row) => row.id}
        loading={isLoading}
        emptyTitle="No active products"
        emptyDescription="Add a product to begin tracking stock and pricing."
        toolbarExtras={
          <Typography variant="body2" color="text.secondary" whiteSpace="nowrap">
            Inventory cost: <MoneyText currency={settings?.default_currency} value={activeProducts.reduce((total, product) => total + Math.round(product.current_quantity * product.cost_price_cents), 0)} />
          </Typography>
        }
        actions={(row) => [
          { label: "Edit", icon: <EditIcon fontSize="small" />, onClick: () => setForm(productToForm(row)) },
          { label: "Adjust stock", icon: <TuneIcon fontSize="small" />, onClick: () => setAdjustProduct(row) },
          { label: "View movement", icon: <HistoryIcon fontSize="small" />, onClick: () => setMovementProduct(row) },
          { label: "Delete", icon: <DeleteIcon fontSize="small" />, destructive: true, onClick: () => setDeleteId(row.id) }
        ]}
      />

      <ProductDialog
        form={form}
        categories={activeCategories}
        suppliers={suppliers}
        error={error}
        saving={saveMutation.isPending}
        onClose={() => setForm(null)}
        onSubmit={submit}
        onChange={setForm}
      />
      <MovementDrawer product={movementProduct} onClose={() => setMovementProduct(null)} />
      <StockAdjustDialog product={adjustProduct} onClose={() => setAdjustProduct(null)} />
      <ConfirmDialog
        open={deleteId !== null}
        title="Delete product"
        message="This removes the product from active lists while preserving its invoice and stock history."
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

function ProductDialog({
  form,
  categories,
  suppliers,
  error,
  saving,
  onClose,
  onSubmit,
  onChange
}: {
  form: ProductForm | null;
  categories: Array<{ id: number; name: string }>;
  suppliers: Array<{ id: number; name: string }>;
  error: string | null;
  saving: boolean;
  onClose: () => void;
  onSubmit: (event: FormEvent) => void;
  onChange: (value: ProductForm | null) => void;
}) {
  return (
    <Dialog open={Boolean(form)} onClose={onClose} fullWidth maxWidth="md">
      <DialogTitle>{form?.id ? "Edit product" : "Add product"}</DialogTitle>
      <DialogContent>
        <Stack component="form" id="product-form" onSubmit={onSubmit} spacing={2} sx={{ pt: 1 }}>
          {error ? <Alert severity="error">{error}</Alert> : null}
          <Box>
            <Typography variant="h6">Required product details</Typography>
            <Typography variant="body2" color="text.secondary">
              Complete these fields before saving.
            </Typography>
          </Box>
          <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", md: "repeat(3, 1fr)" }, gap: 2 }}>
            <TextField label="Product name" required value={form?.name ?? ""} onChange={(e) => onChange(form && { ...form, name: e.target.value })} />
            <TextField select label="Category" required value={form?.category_id || ""} onChange={(e) => onChange(form && { ...form, category_id: Number(e.target.value) })}>
              {categories.map((category) => <MenuItem key={category.id} value={category.id}>{category.name}</MenuItem>)}
            </TextField>
            <TextField select label="Supplier" required value={form?.supplier_id ?? ""} onChange={(e) => onChange(form && { ...form, supplier_id: e.target.value ? Number(e.target.value) : null })}>
              {suppliers.map((supplier) => <MenuItem key={supplier.id} value={supplier.id}>{supplier.name}</MenuItem>)}
            </TextField>
            <SelectField label="Type" required value={form?.product_type ?? ""} values={productTypes} onChange={(value) => onChange(form && { ...form, product_type: value })} />
            <SelectField label="Material" required value={form?.material ?? ""} values={materials} onChange={(value) => onChange(form && { ...form, material: value })} />
            <SelectField label="Shape" required value={form?.shape ?? ""} values={shapes} onChange={(value) => onChange(form && { ...form, shape: value })} />
            <SelectField label="Finish" required value={form?.finish ?? ""} values={finishes} onChange={(value) => onChange(form && { ...form, finish: value })} />
            <NumberField label="Thickness mm" required min={0.001} value={form?.thickness_mm} onChange={(value) => onChange(form && { ...form, thickness_mm: value })} />
            <NumberField label="Minimum stock" required min={0} value={form?.minimum_quantity} onChange={(value) => onChange(form && { ...form, minimum_quantity: value ?? 0 })} />
            <TextField label="Cost price" type="number" required slotProps={{ htmlInput: { min: 0, step: "0.01" } }} value={form?.cost_price ?? "0.00"} onChange={(e) => onChange(form && { ...form, cost_price: e.target.value })} />
            <TextField label="Selling price" type="number" required slotProps={{ htmlInput: { min: 0, step: "0.01" } }} value={form?.selling_price ?? "0.00"} onChange={(e) => onChange(form && { ...form, selling_price: e.target.value })} />
            <TextField label="Wholesale price" type="number" required slotProps={{ htmlInput: { min: 0, step: "0.01" } }} value={form?.wholesale_price ?? "0.00"} onChange={(e) => onChange(form && { ...form, wholesale_price: e.target.value })} />
          </Box>
          <Divider />
          <Box>
            <Typography variant="h6">Optional details</Typography>
            <Typography variant="body2" color="text.secondary">
              Add dimensions, storage, and notes when they apply to this product.
            </Typography>
          </Box>
          <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", md: "repeat(3, 1fr)" }, gap: 2 }}>
            <TextField label="SKU" value={form?.sku ?? ""} onChange={(e) => onChange(form && { ...form, sku: e.target.value })} helperText="Leave blank to auto-generate" />
            <TextField label="Location" value={form?.location ?? ""} onChange={(e) => onChange(form && { ...form, location: e.target.value })} />
            <TextField label="Size label" value={form?.size_label ?? ""} onChange={(e) => onChange(form && { ...form, size_label: e.target.value })} />
            <SelectField label="Unit" value={form?.unit ?? ""} values={units} onChange={(value) => onChange(form && { ...form, unit: value })} />
            <NumberField label="Width mm" value={form?.width_mm} onChange={(value) => onChange(form && { ...form, width_mm: value })} />
            <NumberField label="Height mm" value={form?.height_mm} onChange={(value) => onChange(form && { ...form, height_mm: value })} />
            <NumberField label="Diameter mm" value={form?.diameter_mm} onChange={(value) => onChange(form && { ...form, diameter_mm: value })} />
            <NumberField label="Length mm" value={form?.length_mm} onChange={(value) => onChange(form && { ...form, length_mm: value })} />
            {!form?.id ? <NumberField label="Initial quantity" min={0} value={form?.initial_quantity} onChange={(value) => onChange(form && { ...form, initial_quantity: value ?? 0 })} /> : null}
          </Box>
          <TextField label="Description" multiline minRows={2} value={form?.description ?? ""} onChange={(e) => onChange(form && { ...form, description: e.target.value })} />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button type="submit" form="product-form" variant="contained" disabled={saving}>Save</Button>
      </DialogActions>
    </Dialog>
  );
}

function MovementDrawer({ product, onClose }: { product: Product | null; onClose: () => void }) {
  const queryClient = useQueryClient();
  const [cancelId, setCancelId] = useState<number | null>(null);
  const [cancelError, setCancelError] = useState<string | null>(null);
  const { data = [], isLoading } = useQuery({
    queryKey: ["movement", product?.id],
    queryFn: () => productApi.movement(product!.id),
    enabled: Boolean(product)
  });
  const cancelMutation = useMutation({
    mutationFn: (id: number) => productApi.cancelStockAdjustment(id),
    onSuccess: async () => {
      setCancelId(null);
      setCancelError(null);
      await queryClient.invalidateQueries();
    },
    onError: (error) => setCancelError(normalizeError(error).message)
  });
  const activeRows = useMemo(() => data.filter((row) => row.status === "active"), [data]);
  const columns = useMemo<TableColumn<InventoryTransaction>[]>(() => [
    { id: "date", label: "Date", value: (row) => row.created_at.slice(0, 10), width: 110 },
    { id: "type", label: "Type", value: (row) => row.transaction_type.replace(/_/g, " "), minWidth: 140 },
    { id: "in", label: "In", value: (row) => quantity(row.quantity_in), align: "right", width: 90 },
    { id: "out", label: "Out", value: (row) => quantity(row.quantity_out), align: "right", width: 90 },
    { id: "status", label: "Status", value: (row) => row.status, render: (row) => <StatusBadge value={row.status} />, width: 110 },
    { id: "notes", label: "Notes", value: (row) => row.notes ?? "", minWidth: 180 }
  ], []);

  return (
    <Drawer anchor="right" open={Boolean(product)} onClose={onClose}>
      <Box sx={{ width: { xs: "100vw", lg: 900 }, maxWidth: "100vw", p: { xs: 2, md: 3 } }}>
        <PageHeader
          title="Stock movement"
          description={product?.name}
        />
        {cancelError && cancelId === null ? <Alert severity="error" sx={{ mt: 2 }}>{cancelError}</Alert> : null}
        <Stack spacing={2} sx={{ mt: 2 }}>
          <EnterpriseTable
            title={`Stock Movement — ${product?.name ?? ""}`}
            rows={activeRows}
            columns={columns}
            rowId={(row) => row.id}
            loading={isLoading}
            emptyTitle="No active stock movement"
            emptyDescription="Purchases, sales, and adjustments will appear here."
            actions={(row) => [
              {
                label: "Delete adjustment",
                icon: <DeleteIcon fontSize="small" />,
                destructive: true,
                disabled: !["manual", "product"].includes(row.reference_type),
                onClick: () => { setCancelError(null); setCancelId(row.id); }
              }
            ]}
          />
        </Stack>
        <ConfirmDialog
          open={cancelId !== null}
          title="Delete inventory adjustment"
          message="The movement remains in history, but its stock effect is removed immediately."
          confirmLabel="Delete"
          error={cancelError}
          loading={cancelMutation.isPending}
          onClose={() => setCancelId(null)}
          onConfirm={() => cancelId && cancelMutation.mutate(cancelId)}
        />
      </Box>
    </Drawer>
  );
}

function StockAdjustDialog({ product, onClose }: { product: Product | null; onClose: () => void }) {
  const queryClient = useQueryClient();
  const [type, setType] = useState("adjustment_in");
  const [amount, setAmount] = useState("0");
  const [notes, setNotes] = useState("");
  const mutation = useMutation({
    mutationFn: () => productApi.adjustStock({ product_id: product!.id, transaction_type: type, quantity: Number(amount), notes }),
    onSuccess: async () => {
      onClose();
      await queryClient.invalidateQueries();
    }
  });

  return (
    <Dialog open={Boolean(product)} onClose={onClose} fullWidth maxWidth="xs">
      <DialogTitle>Adjust stock</DialogTitle>
      <DialogContent>
        <Stack spacing={2} sx={{ pt: 1 }}>
          <TextField select label="Transaction type" value={type} onChange={(e) => setType(e.target.value)}>
            <MenuItem value="opening_stock">Opening stock</MenuItem>
            <MenuItem value="adjustment_in">Adjustment in</MenuItem>
            <MenuItem value="adjustment_out">Adjustment out</MenuItem>
            <MenuItem value="damaged_stock">Damaged stock</MenuItem>
          </TextField>
          <TextField label="Quantity" type="number" value={amount} onChange={(e) => setAmount(e.target.value)} />
          <TextField label="Notes" multiline minRows={2} value={notes} onChange={(e) => setNotes(e.target.value)} />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button variant="contained" disabled={mutation.isPending} onClick={() => mutation.mutate()}>Save</Button>
      </DialogActions>
    </Dialog>
  );
}

function SelectField<T extends readonly string[]>({ label, value, values, required = false, onChange }: { label: string; value: string; values: T; required?: boolean; onChange: (value: string) => void }) {
  return (
    <TextField select label={label} required={required} value={value} onChange={(e) => onChange(e.target.value)}>
      {values.map((item) => <MenuItem key={item} value={item}>{item}</MenuItem>)}
    </TextField>
  );
}

function NumberField({ label, value, required = false, min = 0.001, onChange }: { label: string; value?: number | null; required?: boolean; min?: number; onChange: (value: number | null) => void }) {
  return <TextField label={label} type="number" required={required} slotProps={{ htmlInput: { min, step: "any" } }} value={value ?? ""} onChange={(e) => onChange(e.target.value === "" ? null : Number(e.target.value))} />;
}

function formToPayload(form: ProductForm): ProductPayload {
  return {
    ...form,
    cost_price_cents: toCents(form.cost_price),
    selling_price_cents: toCents(form.selling_price),
    wholesale_price_cents: toCents(form.wholesale_price)
  };
}

function productToForm(product: Product): ProductForm {
  return {
    ...product,
    supplier_id: product.supplier_id ?? null,
    location: product.location ?? "",
    size_label: product.size_label ?? "",
    description: product.description ?? "",
    cost_price: fromCents(product.cost_price_cents),
    selling_price: fromCents(product.selling_price_cents),
    wholesale_price: fromCents(product.wholesale_price_cents),
    initial_quantity: 0
  };
}
