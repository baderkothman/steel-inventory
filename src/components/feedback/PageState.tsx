import { Alert, Box, Skeleton, Stack, Typography } from "@mui/material";

export function LoadingState({ label = "Loading" }: { label?: string }) {
  return (
    <Box role="status" aria-live="polite" sx={{ py: 2 }}>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 1.5 }}>{label}</Typography>
      <Stack spacing={1}>
        <Skeleton variant="rounded" height={42} />
        <Skeleton variant="rounded" height={42} />
        <Skeleton variant="rounded" height={42} />
      </Stack>
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
