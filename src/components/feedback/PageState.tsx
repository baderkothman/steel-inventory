import { Alert, Box, CircularProgress, Typography } from "@mui/material";

export function LoadingState({ label = "Loading" }: { label?: string }) {
  return (
    <Box sx={{ display: "flex", alignItems: "center", gap: 1, py: 4 }}>
      <CircularProgress size={18} />
      <Typography color="text.secondary">{label}</Typography>
    </Box>
  );
}

export function EmptyState({ label }: { label: string }) {
  return (
    <Box sx={{ py: 5, textAlign: "center", color: "text.secondary" }}>
      <Typography>{label}</Typography>
    </Box>
  );
}

export function ErrorState({ message }: { message: string }) {
  return <Alert severity="error">{message}</Alert>;
}
