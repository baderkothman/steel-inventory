import { useMemo, useState } from "react";
import {
  Button,
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
import DownloadIcon from "@mui/icons-material/Download";
import PrintIcon from "@mui/icons-material/Print";
import { useQuery } from "@tanstack/react-query";

import { PageHeader } from "../../components/PageHeader";
import { EmptyState, LoadingState } from "../../components/feedback/PageState";
import { reportOptions } from "../../lib/constants";
import { money, today } from "../../lib/formatters";
import { reportApi } from "../../lib/api";
import type { ReportFilters, ReportRow } from "../../types/report";

type ReportKey = (typeof reportOptions)[number]["value"];

export function ReportsPage() {
  const [report, setReport] = useState<ReportKey>("daily_sales");
  const [filters, setFilters] = useState<ReportFilters>({ date_from: today(), date_to: today() });

  const { data = [], isLoading } = useQuery({
    queryKey: ["report", report, filters],
    queryFn: () => runReport(report, filters)
  });

  const columns = useMemo(() => Array.from(new Set(data.flatMap((row) => Object.keys(row)))), [data]);

  function exportCsv() {
    const csv = [columns.join(","), ...data.map((row) => columns.map((column) => csvCell(row[column])).join(","))].join("\n");
    const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `${report}.csv`;
    link.click();
    URL.revokeObjectURL(url);
  }

  function printReport() {
    window.print();
  }

  return (
    <Stack spacing={2}>
      <PageHeader
        title="Reports"
        description="Generate operational, profit, stock, debt, expense, payment, and inventory reports."
        actions={
          <Stack direction="row" spacing={1}>
            <Button startIcon={<PrintIcon />} onClick={printReport}>Print</Button>
            <Button startIcon={<DownloadIcon />} variant="contained" disabled={!data.length} onClick={exportCsv}>Export CSV</Button>
          </Stack>
        }
      />

      <Paper variant="outlined" sx={{ p: 2 }}>
        <Stack direction={{ xs: "column", md: "row" }} spacing={1.5}>
          <TextField select label="Report" value={report} onChange={(e) => setReport(e.target.value as ReportKey)} sx={{ minWidth: 240 }}>
            {reportOptions.map((option) => <MenuItem key={option.value} value={option.value}>{option.label}</MenuItem>)}
          </TextField>
          <TextField label="From" type="date" value={filters.date_from ?? ""} onChange={(e) => setFilters((current) => ({ ...current, date_from: e.target.value || null }))} />
          <TextField label="To" type="date" value={filters.date_to ?? ""} onChange={(e) => setFilters((current) => ({ ...current, date_to: e.target.value || null }))} />
          <TextField label="Payment status" value={filters.payment_status ?? ""} onChange={(e) => setFilters((current) => ({ ...current, payment_status: e.target.value || null }))} />
        </Stack>
      </Paper>

      <Paper variant="outlined">
        {isLoading ? <LoadingState label="Loading report" /> : data.length === 0 ? <EmptyState label="No rows for this report." /> : (
          <Table size="small">
            <TableHead><TableRow>{columns.map((column) => <TableCell key={column}>{label(column)}</TableCell>)}</TableRow></TableHead>
            <TableBody>{data.map((row, index) => <TableRow key={index}>{columns.map((column) => <TableCell key={column}>{formatCell(column, row[column])}</TableCell>)}</TableRow>)}</TableBody>
          </Table>
        )}
      </Paper>
    </Stack>
  );
}

function runReport(report: ReportKey, filters: ReportFilters): Promise<ReportRow[]> {
  switch (report) {
    case "daily_sales": return reportApi.dailySales(filters);
    case "daily_profit": return reportApi.profit(filters);
    case "monthly_profit": return reportApi.monthlyProfit(filters);
    case "stock": return reportApi.stock(filters);
    case "stock_movement": return reportApi.stockMovement(filters);
    case "low_stock": return reportApi.lowStock();
    case "purchase": return reportApi.purchase(filters);
    case "supplier_debt": return reportApi.supplierDebt(filters);
    case "customer_debt": return reportApi.customerDebt(filters);
    case "expense": return reportApi.expense(filters);
    case "payment": return reportApi.payment(filters);
    case "inventory_value": return reportApi.inventoryValue();
    case "best_selling": return reportApi.bestSelling(filters);
  }
}

function label(value: string) {
  return value.replace(/_/g, " ").replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function formatCell(column: string, value: unknown) {
  if (column.endsWith("_cents") && typeof value === "number") return money(value);
  if (value === null || value === undefined) return "";
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

function csvCell(value: unknown) {
  const text = value === null || value === undefined ? "" : String(value);
  return `"${text.replace(/"/g, '""')}"`;
}
