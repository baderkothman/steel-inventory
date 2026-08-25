import { FormEvent, ReactNode, useMemo, useState } from "react";
import {
  Alert,
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
import DeleteIcon from "@mui/icons-material/Delete";
import EditIcon from "@mui/icons-material/Edit";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { LoadingState } from "../../components/feedback/PageState";
import { PageHeader } from "../../components/PageHeader";
import { EnterpriseTable, type TableColumn } from "../../components/table/EnterpriseTable";
import { categoryApi } from "../../lib/api";
import { normalizeError } from "../../lib/tauri";
import type { Category } from "../../types/common";

type FormState = {
  id?: number;
  name: string;
  parent_id?: number | null;
  description?: string | null;
};

const emptyForm: FormState = { name: "", parent_id: null, description: "" };

export function CategoriesPage() {
  const queryClient = useQueryClient();
  const { data = [], isLoading } = useQuery({ queryKey: ["categories"], queryFn: categoryApi.list });
  const [form, setForm] = useState<FormState | null>(null);
  const [deleteId, setDeleteId] = useState<number | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const activeCategories = useMemo(() => data.filter((category) => category.is_active), [data]);
  const categoryNames = useMemo(() => new Map(data.map((category) => [category.id, category.name])), [data]);
  const columns = useMemo<TableColumn<Category>[]>(() => [
    { id: "name", label: "Name", value: (row) => row.name, minWidth: 180 },
    { id: "parent", label: "Parent", value: (row) => row.parent_id ? categoryNames.get(row.parent_id) ?? "Unknown" : "Root", minWidth: 140 },
    { id: "description", label: "Description", value: (row) => row.description ?? "", minWidth: 240 }
  ], [categoryNames]);

  const saveMutation = useMutation({
    mutationFn: (value: FormState) =>
      value.id
        ? categoryApi.update(value.id, {
            name: value.name,
            parent_id: value.parent_id ?? null,
            description: value.description ?? null
          })
        : categoryApi.create({
            name: value.name,
            parent_id: value.parent_id ?? null,
            description: value.description ?? null
          }),
    onSuccess: async () => {
      setForm(null);
      setError(null);
      await queryClient.invalidateQueries({ queryKey: ["categories"] });
    },
    onError: (err) => setError(normalizeError(err).message)
  });

  const deleteMutation = useMutation({
    mutationFn: (id: number) => categoryApi.archive(id),
    onSuccess: async () => {
      setDeleteId(null);
      setDeleteError(null);
      await queryClient.invalidateQueries({ queryKey: ["categories"] });
    },
    onError: (err) => setDeleteError(normalizeError(err).message)
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    if (form) {
      saveMutation.mutate(form);
    }
  }

  if (isLoading) {
    return <LoadingState label="Loading categories" />;
  }

  return (
    <Stack spacing={2}>
      <PageHeader
        title="Categories"
        description="Manage parent and child product categories."
        actions={<Button startIcon={<AddIcon />} variant="contained" onClick={() => setForm(emptyForm)}>Add category</Button>}
      />
      {deleteError && deleteId === null ? <Alert severity="error">{deleteError}</Alert> : null}

      <EnterpriseTable
        title="Product Categories"
        rows={activeCategories}
        columns={columns}
        rowId={(row) => row.id}
        loading={isLoading}
        emptyTitle="No active categories"
        emptyDescription="Add a category to organize your product catalog."
        actions={(row) => [
          { label: "Edit", icon: <EditIcon fontSize="small" />, onClick: () => setForm(categoryToForm(row)) },
          { label: "Delete", icon: <DeleteIcon fontSize="small" />, destructive: true, onClick: () => setDeleteId(row.id) }
        ]}
      />

      <Dialog open={Boolean(form)} onClose={() => setForm(null)} fullWidth maxWidth="sm">
        <DialogTitle>{form?.id ? "Edit category" : "Add category"}</DialogTitle>
        <DialogContent>
          <Stack component="form" id="category-form" onSubmit={submit} spacing={2} sx={{ pt: 1 }}>
            {error ? <Alert severity="error">{error}</Alert> : null}
            <TextField
              label="Name"
              value={form?.name ?? ""}
              required
              onChange={(event) => setForm((current) => current && { ...current, name: event.target.value })}
            />
            <TextField
              select
              label="Parent category"
              value={form?.parent_id ?? ""}
              onChange={(event) =>
                setForm((current) =>
                  current && {
                    ...current,
                    parent_id: event.target.value ? Number(event.target.value) : null
                  }
                )
              }
            >
              <MenuItem value="">Root</MenuItem>
              {activeCategories.reduce<ReactNode[]>((options, category) => {
                if (category.id !== form?.id) {
                  options.push(
                    <MenuItem key={category.id} value={category.id}>
                      {category.name}
                    </MenuItem>
                  );
                }
                return options;
              }, [])}
            </TextField>
            <TextField
              label="Description"
              value={form?.description ?? ""}
              multiline
              minRows={3}
              onChange={(event) =>
                setForm((current) => current && { ...current, description: event.target.value })
              }
            />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setForm(null)}>Cancel</Button>
          <Button type="submit" form="category-form" variant="contained" disabled={saveMutation.isPending}>
            Save
          </Button>
        </DialogActions>
      </Dialog>

      <ConfirmDialog
        open={deleteId !== null}
        title="Delete category"
        message="This removes the category from active lists while preserving existing transaction history."
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

function categoryToForm(category: Category): FormState {
  return {
    id: category.id,
    name: category.name,
    parent_id: category.parent_id,
    description: category.description
  };
}
