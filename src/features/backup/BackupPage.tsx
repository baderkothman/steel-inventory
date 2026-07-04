import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Button, Paper, Stack, Table, TableBody, TableCell, TableHead, TableRow, TextField } from "@mui/material";
import BackupIcon from "@mui/icons-material/Backup";
import RestoreIcon from "@mui/icons-material/Restore";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { PageHeader } from "../../components/PageHeader";
import { EmptyState, LoadingState } from "../../components/feedback/PageState";
import { backupApi } from "../../lib/api";

export function BackupPage() {
  const queryClient = useQueryClient();
  const [restorePath, setRestorePath] = useState("");
  const { data = [], isLoading } = useQuery({ queryKey: ["backups"], queryFn: backupApi.list });

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
      <PageHeader title="Backup" description="Create manual backups and restore local SQLite database files." />
      <Paper variant="outlined" sx={{ p: 2 }}>
        <Stack direction={{ xs: "column", md: "row" }} spacing={1.5}>
          <Button startIcon={<BackupIcon />} variant="contained" disabled={backupMutation.isPending} onClick={() => backupMutation.mutate()}>Create manual backup</Button>
          <TextField label="Restore file path" value={restorePath} onChange={(e) => setRestorePath(e.target.value)} sx={{ flex: 1 }} />
          <Button onClick={chooseFile}>Browse</Button>
          <Button color="warning" startIcon={<RestoreIcon />} variant="contained" disabled={!restorePath || restoreMutation.isPending} onClick={restore}>Restore</Button>
        </Stack>
      </Paper>
      <Paper variant="outlined">
        {isLoading ? <LoadingState label="Loading backups" /> : data.length === 0 ? <EmptyState label="No backups recorded." /> : (
          <Table size="small">
            <TableHead><TableRow><TableCell>Date</TableCell><TableCell>Type</TableCell><TableCell>Status</TableCell><TableCell>Path</TableCell><TableCell>Notes</TableCell></TableRow></TableHead>
            <TableBody>{data.map((row) => <TableRow key={row.id}><TableCell>{row.created_at}</TableCell><TableCell>{row.backup_type}</TableCell><TableCell>{row.status}</TableCell><TableCell>{row.backup_path}</TableCell><TableCell>{row.notes}</TableCell></TableRow>)}</TableBody>
          </Table>
        )}
      </Paper>
    </Stack>
  );
}
