import { Chip } from "@mui/material";

const success = new Set(["active", "paid", "completed", "success", "in", "accepted", "converted"]);
const warning = new Set(["partial", "unpaid", "warning", "out", "sent", "draft"]);
const danger = new Set(["cancelled", "deleted", "failed", "archived", "rejected", "expired"]);

export function StatusBadge({ value }: { value?: string | null }) {
  const normalized = (value || "unknown").trim().toLowerCase();
  const color = success.has(normalized)
    ? "success"
    : warning.has(normalized)
      ? "warning"
      : danger.has(normalized)
        ? "error"
        : "default";
  return (
    <Chip
      size="small"
      color={color}
      variant="outlined"
      label={normalized.replace(/_/g, " ").replace(/\b\w/g, (letter) => letter.toUpperCase())}
      sx={{ height: 24, fontWeight: 700, "& .MuiChip-label": { px: 1 } }}
    />
  );
}
