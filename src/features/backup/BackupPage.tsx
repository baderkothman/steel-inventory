import { useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Button, Paper, Stack, TextField } from "@mui/material";
import BackupIcon from "@mui/icons-material/Backup";
import RestoreIcon from "@mui/icons-material/Restore";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { PageHeader } from "../../components/PageHeader";
import { EnterpriseTable, type TableColumn } from "../../components/table/EnterpriseTable";
import { StatusBadge } from "../../components/table/StatusBadge";
import { backupApi } from "../../lib/api";
import type { BackupRow } from "../../types/common";

export function BackupPage() {
  const queryClient = useQueryClient();
  const [restorePath, setRestorePath] = useState("");
  const { data = [], isLoading } = useQuery({ queryKey: ["backups"], queryFn: backupApi.list });
  const columns = useMemo<TableColumn<BackupRow>[]>(() => [
    { id: "date", label: "Date", value: (row) => row.created_at, minWidth: 170 },
    { id: "type", label: "Type", value: (row) => row.backup_type, width: 120 },
    { id: "status", label: "Status", value: (row) => row.status, render: (row) => <StatusBadge value={row.status} />, width: 110 },
    { id: "path", label: "Path", value: (row) => row.backup_path, minWidth: 280 },
    { id: "notes", label: "Notes", value: (row) => row.notes ?? "", minWidth: 180 }
  ], []);

  const backupMutation = useMutation({
    mutationFn: backupApi.create,
    onSuccess: async () => queryClient.invalidateQueries({ queryKey: ["backups"] })
  });
  const restoreMutation = useMutation({
    mutationFn: (path: string) => backupApi.restore(path),
    onSuccess: async () => queryClient.invalidateQueries({ queryKey: ["backups"] })
  });

  async function chooseFile() {
    const selected = await open({ multiple: false, filters: [{ name: "SQLite database", extensions: ["db", "sqlite"] }] });
    if (typeof selected === "string") setRestorePath(selected);
  }

  function restore() {
    if (restorePath && window.confirm("Restore this backup? An emergency backup of the current database will be created first.")) {
      restoreMutation.mutate(restorePath);
    }
  }

  return (
    <Stack spacing={2}>
      <PageHeader
        title="Backup"
        description="Create manual backups and restore local SQLite database files."
      />
      <Paper variant="outlined" sx={{ p: 2 }}>
        <Stack direction={{ xs: "column", md: "row" }} spacing={1.5}>
          <Button startIcon={<BackupIcon />} variant="contained" disabled={backupMutation.isPending} onClick={() => backupMutation.mutate()}>Create manual backup</Button>
          <TextField label="Restore file path" value={restorePath} onChange={(e) => setRestorePath(e.target.value)} sx={{ flex: 1 }} />
          <Button onClick={chooseFile}>Browse</Button>
          <Button color="warning" startIcon={<RestoreIcon />} variant="contained" disabled={!restorePath || restoreMutation.isPending} onClick={restore}>Restore</Button>
        </Stack>
      </Paper>
      <EnterpriseTable
        title="Backup History"
        rows={data}
        columns={columns}
        rowId={(row) => row.id}
        loading={isLoading}
        initialSort={{ column: "date", direction: "desc" }}
        emptyTitle="No backups recorded"
        emptyDescription="Create a manual backup to establish a recovery point."
      />
    </Stack>
  );
}
