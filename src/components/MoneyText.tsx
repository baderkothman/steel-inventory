import { Typography } from "@mui/material";

import { money } from "../lib/formatters";

export function MoneyText({
  value,
  currency = "USD",
  color
}: {
  value?: number | null;
  currency?: string;
  color?: string;
}) {
  return (
    <Typography component="span" sx={{ fontVariantNumeric: "tabular-nums", color }}>
      {money(value, currency)}
    </Typography>
  );
}
