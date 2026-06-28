import { FormEvent, useMemo, useState } from "react";
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
import ArchiveIcon from "@mui/icons-material/Archive";
import EditIcon from "@mui/icons-material/Edit";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { ConfirmDialog } from "../../components/feedback/ConfirmDialog";
import { EmptyState, LoadingState } from "../../components/feedback/PageState";
import { PageHeader } from "../../components/PageHeader";
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
  const [archiveId, setArchiveId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  const activeCategories = useMemo(() => data.filter((category) => category.is_active), [data]);
  const categoryNames = useMemo(() => new Map(data.map((category) => [category.id, category.name])), [data]);

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

  const archiveMutation = useMutation({
    mutationFn: (id: number) => categoryApi.archive(id),
    onSuccess: async () => {
      setArchiveId(null);
      await queryClient.invalidateQueries({ queryKey: ["categories"] });
    }
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
        actions={
          <Button startIcon={<AddIcon />} variant="contained" onClick={() => setForm(emptyForm)}>
            Add category
          </Button>
        }
      />

      <Paper variant="outlined">
        {data.length === 0 ? (
          <EmptyState label="No categories found." />
        ) : (
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>Name</TableCell>
                <TableCell>Parent</TableCell>
                <TableCell>Description</TableCell>
                <TableCell>Status</TableCell>
                <TableCell align="right">Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {data.map((category) => (
                <TableRow key={category.id} hover>
                  <TableCell>{category.name}</TableCell>
                  <TableCell>{category.parent_id ? categoryNames.get(category.parent_id) : "Root"}</TableCell>
                  <TableCell>{category.description}</TableCell>
                  <TableCell>{category.is_active ? "Active" : "Archived"}</TableCell>
                  <TableCell align="right">
                    <Button size="small" startIcon={<EditIcon />} onClick={() => setForm(categoryToForm(category))}>
                      Edit
                    </Button>
                    <Button
                      size="small"
                      color="warning"
                      startIcon={<ArchiveIcon />}
                      disabled={!category.is_active}
                      onClick={() => setArchiveId(category.id)}
                    >
                      Archive
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </Paper>

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
              {activeCategories
                .filter((category) => category.id !== form?.id)
                .map((category) => (
                  <MenuItem key={category.id} value={category.id}>
                    {category.name}
                  </MenuItem>
                ))}
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
        open={archiveId !== null}
        title="Archive category"
        message="Archived categories stay in history but are hidden from active workflows."
        confirmLabel="Archive"
        onClose={() => setArchiveId(null)}
        onConfirm={() => archiveId && archiveMutation.mutate(archiveId)}
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
