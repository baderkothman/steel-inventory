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
import ArchiveIcon from "@mui/icons-material/Archive";
import EditIcon from "@mui/icons-material/Edit";
import HistoryIcon from "@mui/icons-material/History";
import TuneIcon from "@mui/icons-material/Tune";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MoneyText } from "../../components/MoneyText";
import { PageHeader } from "../../components/PageHeader";
import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { EmptyState, LoadingState } from "../../components/feedback/PageState";
import { categoryApi, productApi } from "../../lib/api";
import { finishes, materials, productTypes, shapes, units } from "../../lib/constants";
import { fromCents, quantity, toCents } from "../../lib/formatters";
import { normalizeError } from "../../lib/tauri";
import type { Product, ProductPayload } from "../../types/product";

type ProductForm = Omit<ProductPayload, "cost_price_cents" | "selling_price_cents" | "wholesale_price_cents"> & {
  id?: number;
  cost_price: string;
  selling_price: string;
  wholesale_price: string;
};

const blankProduct: ProductForm = {
  sku: "",
  category_id: 0,
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
  const [search, setSearch] = useState("");
  const [categoryId, setCategoryId] = useState<number | "">("");
  const [form, setForm] = useState<ProductForm | null>(null);
  const [archiveId, setArchiveId] = useState<number | null>(null);
  const [movementProduct, setMovementProduct] = useState<Product | null>(null);
  const [adjustProduct, setAdjustProduct] = useState<Product | null>(null);
  const [error, setError] = useState<string | null>(null);

  const { data: categories = [] } = useQuery({ queryKey: ["categories"], queryFn: categoryApi.list });
  const { data: products = [], isLoading } = useQuery({
    queryKey: ["products", search, categoryId],
    queryFn: () =>
      productApi.list({
        search: search || null,
        category_id: categoryId || null,
        active_only: false
      })
  });

  const activeCategories = useMemo(() => categories.filter((category) => category.is_active), [categories]);

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

  const archiveMutation = useMutation({
    mutationFn: (id: number) => productApi.archive(id),
    onSuccess: async () => {
      setArchiveId(null);
      await queryClient.invalidateQueries({ queryKey: ["products"] });
    }
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

      <Paper variant="outlined" sx={{ p: 2 }}>
        <Stack direction={{ xs: "column", sm: "row" }} spacing={1.5}>
          <TextField label="Search" value={search} onChange={(event) => setSearch(event.target.value)} sx={{ minWidth: 260 }} />
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
        </Stack>
      </Paper>

      <Paper variant="outlined">
        {products.length === 0 ? (
          <EmptyState label="No products found." />
        ) : (
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>SKU</TableCell>
                <TableCell>Product</TableCell>
                <TableCell>Category</TableCell>
                <TableCell>Size</TableCell>
                <TableCell align="right">Stock</TableCell>
                <TableCell align="right">Cost</TableCell>
                <TableCell align="right">Selling</TableCell>
                <TableCell>Status</TableCell>
                <TableCell align="right">Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {products.map((product) => (
                <TableRow key={product.id} hover>
                  <TableCell>{product.sku}</TableCell>
                  <TableCell>{product.name}</TableCell>
                  <TableCell>{product.category_name}</TableCell>
                  <TableCell>
                    {product.size_label} {product.thickness_mm ? `${product.thickness_mm}mm` : ""}
                  </TableCell>
                  <TableCell align="right">{quantity(product.current_quantity)}</TableCell>
                  <TableCell align="right"><MoneyText value={product.cost_price_cents} /></TableCell>
                  <TableCell align="right"><MoneyText value={product.selling_price_cents} /></TableCell>
                  <TableCell>{product.is_active ? "Active" : "Archived"}</TableCell>
                  <TableCell align="right">
                    <Button size="small" startIcon={<EditIcon />} onClick={() => setForm(productToForm(product))}>Edit</Button>
                    <Button size="small" startIcon={<TuneIcon />} onClick={() => setAdjustProduct(product)}>Adjust</Button>
                    <Button size="small" startIcon={<HistoryIcon />} onClick={() => setMovementProduct(product)}>Movement</Button>
                    <Button size="small" color="warning" startIcon={<ArchiveIcon />} disabled={!product.is_active} onClick={() => setArchiveId(product.id)}>Archive</Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </Paper>

      <ProductDialog
        form={form}
        categories={activeCategories}
        error={error}
        saving={saveMutation.isPending}
        onClose={() => setForm(null)}
        onSubmit={submit}
        onChange={setForm}
      />
      <MovementDrawer product={movementProduct} onClose={() => setMovementProduct(null)} />
      <StockAdjustDialog product={adjustProduct} onClose={() => setAdjustProduct(null)} />
      <ConfirmDialog
        open={archiveId !== null}
        title="Archive product"
        message="Archived products stay in invoice history but are hidden from active workflows."
        confirmLabel="Archive"
        onClose={() => setArchiveId(null)}
        onConfirm={() => archiveId && archiveMutation.mutate(archiveId)}
      />
    </Stack>
  );
}

function ProductDialog({
  form,
  categories,
  error,
  saving,
  onClose,
  onSubmit,
  onChange
}: {
  form: ProductForm | null;
  categories: Array<{ id: number; name: string }>;
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
          <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", md: "repeat(3, 1fr)" }, gap: 2 }}>
            <TextField label="SKU" value={form?.sku ?? ""} onChange={(e) => onChange(form && { ...form, sku: e.target.value })} helperText="Leave blank to auto-generate" />
            <TextField label="Product name" required value={form?.name ?? ""} onChange={(e) => onChange(form && { ...form, name: e.target.value })} />
            <TextField select label="Category" required value={form?.category_id || ""} onChange={(e) => onChange(form && { ...form, category_id: Number(e.target.value) })}>
              {categories.map((category) => <MenuItem key={category.id} value={category.id}>{category.name}</MenuItem>)}
            </TextField>
            <SelectField label="Type" value={form?.product_type ?? ""} values={productTypes} onChange={(value) => onChange(form && { ...form, product_type: value })} />
            <SelectField label="Material" value={form?.material ?? ""} values={materials} onChange={(value) => onChange(form && { ...form, material: value })} />
            <SelectField label="Shape" value={form?.shape ?? ""} values={shapes} onChange={(value) => onChange(form && { ...form, shape: value })} />
            <SelectField label="Finish" value={form?.finish ?? ""} values={finishes} onChange={(value) => onChange(form && { ...form, finish: value })} />
            <TextField label="Size label" value={form?.size_label ?? ""} onChange={(e) => onChange(form && { ...form, size_label: e.target.value })} />
            <SelectField label="Unit" value={form?.unit ?? ""} values={units} onChange={(value) => onChange(form && { ...form, unit: value })} />
            <NumberField label="Width mm" value={form?.width_mm} onChange={(value) => onChange(form && { ...form, width_mm: value })} />
            <NumberField label="Height mm" value={form?.height_mm} onChange={(value) => onChange(form && { ...form, height_mm: value })} />
            <NumberField label="Diameter mm" value={form?.diameter_mm} onChange={(value) => onChange(form && { ...form, diameter_mm: value })} />
            <NumberField label="Thickness mm" value={form?.thickness_mm} onChange={(value) => onChange(form && { ...form, thickness_mm: value })} />
            <NumberField label="Length mm" value={form?.length_mm} onChange={(value) => onChange(form && { ...form, length_mm: value })} />
            <NumberField label="Minimum stock" value={form?.minimum_quantity} onChange={(value) => onChange(form && { ...form, minimum_quantity: value ?? 0 })} />
            <TextField label="Cost price" value={form?.cost_price ?? "0.00"} onChange={(e) => onChange(form && { ...form, cost_price: e.target.value })} />
            <TextField label="Selling price" value={form?.selling_price ?? "0.00"} onChange={(e) => onChange(form && { ...form, selling_price: e.target.value })} />
            <TextField label="Wholesale price" value={form?.wholesale_price ?? "0.00"} onChange={(e) => onChange(form && { ...form, wholesale_price: e.target.value })} />
            {!form?.id ? <NumberField label="Initial quantity" value={form?.initial_quantity} onChange={(value) => onChange(form && { ...form, initial_quantity: value ?? 0 })} /> : null}
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
  const { data = [], isLoading } = useQuery({
    queryKey: ["movement", product?.id],
    queryFn: () => productApi.movement(product!.id),
    enabled: Boolean(product)
  });

  return (
    <Drawer anchor="right" open={Boolean(product)} onClose={onClose}>
      <Box sx={{ width: 620, p: 3 }}>
        <PageHeader title="Stock movement" description={product?.name} />
        {isLoading ? <LoadingState /> : data.length === 0 ? <EmptyState label="No movement recorded." /> : (
          <Table size="small">
            <TableHead><TableRow><TableCell>Date</TableCell><TableCell>Type</TableCell><TableCell align="right">In</TableCell><TableCell align="right">Out</TableCell><TableCell>Notes</TableCell></TableRow></TableHead>
            <TableBody>{data.map((row) => <TableRow key={row.id}><TableCell>{row.created_at.slice(0, 10)}</TableCell><TableCell>{row.transaction_type}</TableCell><TableCell align="right">{quantity(row.quantity_in)}</TableCell><TableCell align="right">{quantity(row.quantity_out)}</TableCell><TableCell>{row.notes}</TableCell></TableRow>)}</TableBody>
          </Table>
        )}
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
      await queryClient.invalidateQueries({ queryKey: ["products"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
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

function SelectField<T extends readonly string[]>({ label, value, values, onChange }: { label: string; value: string; values: T; onChange: (value: string) => void }) {
  return (
    <TextField select label={label} value={value} onChange={(e) => onChange(e.target.value)}>
      {values.map((item) => <MenuItem key={item} value={item}>{item}</MenuItem>)}
    </TextField>
  );
}

function NumberField({ label, value, onChange }: { label: string; value?: number | null; onChange: (value: number | null) => void }) {
  return <TextField label={label} type="number" value={value ?? ""} onChange={(e) => onChange(e.target.value === "" ? null : Number(e.target.value))} />;
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
    size_label: product.size_label ?? "",
    description: product.description ?? "",
    cost_price: fromCents(product.cost_price_cents),
    selling_price: fromCents(product.selling_price_cents),
    wholesale_price: fromCents(product.wholesale_price_cents),
    initial_quantity: 0
  };
}
