import { useMemo, useState } from "react";
import {
  Button,
  MenuItem,
  Paper,
  Stack,
  TextField
} from "@mui/material";
import PrintIcon from "@mui/icons-material/Print";
import { useQuery } from "@tanstack/react-query";

import { PageHeader } from "../../components/PageHeader";
import { PrintDialog } from "../../components/print/PrintDialog";
import { EnterpriseTable, type TableColumn } from "../../components/table/EnterpriseTable";
import { reportOptions } from "../../lib/constants";
import { money, today } from "../../lib/formatters";
import { categoryApi, reportApi, settingsApi, supplierApi } from "../../lib/api";
import type { CompanySettings } from "../../types/common";
import type { ReportFilters, ReportRow } from "../../types/report";

type ReportKey = (typeof reportOptions)[number]["value"];

const supplierReports: ReportKey[] = [
  "stock",
  "stock_count",
  "supplier_settlement",
  "supplier_settlement_summary",
  "supplier_debt"
];
const categoryReports: ReportKey[] = ["stock", "stock_count", "cheapest_supplier", "best_selling", "low_stock"];

export function ReportsPage() {
  const [report, setReport] = useState<ReportKey>("daily_sales");
  const [filters, setFilters] = useState<ReportFilters>({ date_from: today(), date_to: today() });

  const { data: suppliers = [] } = useQuery({ queryKey: ["suppliers"], queryFn: () => supplierApi.list({ active_only: true }) });
  const { data: categories = [] } = useQuery({ queryKey: ["categories"], queryFn: categoryApi.list });
  const { data: settings, isLoading: isSettingsLoading } = useQuery({
    queryKey: ["settings"],
    queryFn: settingsApi.get
  });

  const showSupplier = supplierReports.includes(report);
  const showCategory = categoryReports.includes(report);

  const [countSheetHtml, setCountSheetHtml] = useState("");

  const { data = [], isLoading } = useQuery({
    queryKey: ["report", report, filters],
    queryFn: () => runReport(report, filters)
  });

  const columns = useMemo(
    () => orderReportColumns(
      report,
      Array.from(new Set(data.flatMap((row) => Object.keys(row))))
    ),
    [data, report]
  );
  const reportTitle = reportOptions.find((option) => option.value === report)?.label ?? "Report";
  const tableRows = useMemo(() => data.map((row, index) => ({ id: index, data: row })), [data]);
  const tableColumns = useMemo<TableColumn<{ id: number; data: ReportRow }>[]>(() =>
    columns.map((column) => ({
      id: column,
      label: label(column),
      value: (row) => formatCell(column, row.data[column], settings?.default_currency ?? "USD"),
      align: numericReportColumn(column) ? "right" : "left",
      minWidth: numericReportColumn(column) ? 110 : 140
    })),
  [columns, settings?.default_currency]);

  function printStockCount() {
    const supplierName = filters.supplier_id ? suppliers.find((s) => s.id === filters.supplier_id)?.name : "All suppliers";
    const categoryName = filters.category_id ? categories.find((c) => c.id === filters.category_id)?.name : "All categories";
    setCountSheetHtml(buildStockCountSheet(
      data,
      supplierName ?? "All suppliers",
      categoryName ?? "All categories",
      settings
    ));
  }

  return (
    <Stack spacing={2}>
      <PageHeader
        title="Reports"
        description="Generate operational, profit, stock, debt, expense, payment, and inventory reports."
        actions={
          report === "stock_count"
            ? <Button startIcon={<PrintIcon />} variant="outlined" disabled={!data.length} onClick={printStockCount}>Print count sheet</Button>
            : null
        }
      />

      <Paper variant="outlined" sx={{ p: 2 }}>
        <Stack direction={{ xs: "column", md: "row" }} spacing={1.5}>
          <TextField select label="Report" value={report} onChange={(e) => setReport(e.target.value as ReportKey)} sx={{ minWidth: 240 }}>
            {reportOptions.map((option) => <MenuItem key={option.value} value={option.value}>{option.label}</MenuItem>)}
          </TextField>
          <TextField label="From" type="date" value={filters.date_from ?? ""} onChange={(e) => setFilters((current) => ({ ...current, date_from: e.target.value || null }))} />
          <TextField label="To" type="date" value={filters.date_to ?? ""} onChange={(e) => setFilters((current) => ({ ...current, date_to: e.target.value || null }))} />
          {showSupplier ? (
            <TextField select label="Supplier" value={filters.supplier_id ?? ""} sx={{ minWidth: 200 }}
              onChange={(e) => setFilters((current) => ({ ...current, supplier_id: e.target.value ? Number(e.target.value) : null }))}>
              <MenuItem value="">All suppliers</MenuItem>
              {suppliers.map((s) => <MenuItem key={s.id} value={s.id}>{s.name}</MenuItem>)}
            </TextField>
          ) : null}
          {showCategory ? (
            <TextField select label="Category" value={filters.category_id ?? ""} sx={{ minWidth: 200 }}
              onChange={(e) => setFilters((current) => ({ ...current, category_id: e.target.value ? Number(e.target.value) : null }))}>
              <MenuItem value="">All categories</MenuItem>
              {categories.filter((c) => c.is_active).map((c) => <MenuItem key={c.id} value={c.id}>{c.name}</MenuItem>)}
            </TextField>
          ) : null}
          {report === "stock_count" ? (
            <TextField select label="Stock filter" value={filters.payment_status === "low" ? "low" : "all"} sx={{ minWidth: 160 }}
              onChange={(e) => setFilters((current) => ({ ...current, payment_status: e.target.value === "low" ? "low" : null }))}>
              <MenuItem value="all">All stock</MenuItem>
              <MenuItem value="low">Low stock only</MenuItem>
            </TextField>
          ) : (
            <TextField label="Payment status" value={filters.payment_status ?? ""} onChange={(e) => setFilters((current) => ({ ...current, payment_status: e.target.value || null }))} />
          )}
        </Stack>
      </Paper>

      <EnterpriseTable
        title={reportTitle}
        filename={report}
        rows={tableRows}
        columns={tableColumns}
        rowId={(row) => row.id}
        loading={isLoading || isSettingsLoading}
        emptyTitle="No report results"
        emptyDescription="Adjust the filters or select another report."
        initialSort={columns.includes("date") ? { column: "date", direction: "desc" } : undefined}
      />

      <PrintDialog open={Boolean(countSheetHtml)} html={countSheetHtml} onClose={() => setCountSheetHtml("")} />
    </Stack>
  );
}

function runReport(report: ReportKey, filters: ReportFilters): Promise<ReportRow[]> {
  switch (report) {
    case "daily_sales": return reportApi.dailySales(filters);
    case "daily_profit": return reportApi.profit(filters);
    case "monthly_profit": return reportApi.monthlyProfit(filters);
    case "stock": return reportApi.stock(filters);
    case "stock_count": return reportApi.stockCount(filters);
    case "stock_movement": return reportApi.stockMovement(filters);
    case "low_stock": return reportApi.lowStock();
    case "cheapest_supplier": return reportApi.cheapestSupplier(filters);
    case "supplier_settlement": return reportApi.supplierSettlement(filters);
    case "supplier_settlement_summary": return reportApi.supplierSettlementSummary(filters);
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
  if (value === "product_name") return "Name";
  if (value === "thickness_mm") return "Thickness";
  if (value === "selling_price_cents") return "Price";
  return value.replace(/_/g, " ").replace(/\b\w/g, (letter) => letter.toUpperCase());
}

export function orderReportColumns(report: ReportKey, columns: string[]) {
  if (report !== "stock" && report !== "stock_count") return columns;
  const productOrder = ["product_name", "thickness_mm", "selling_price_cents"];
  return [
    ...productOrder.filter((column) => columns.includes(column)),
    ...columns.filter((column) => !productOrder.includes(column))
  ];
}

function formatCell(column: string, value: unknown, currency: string) {
  if (column.endsWith("_cents") && typeof value === "number") return money(value, currency);
  if (value === null || value === undefined) return "";
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

function numericReportColumn(column: string) {
  return column.endsWith("_cents")
    || column.endsWith("_mm")
    || column.includes("quantity")
    || column.includes("amount")
    || column.includes("total")
    || column.includes("profit")
    || column.includes("balance");
}

function escapeHtml(value: unknown) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

/** Builds a print-friendly physical stock-count sheet with blank counted/difference
 *  columns and prepared-by / checked-by fields for manual entry on paper. */
function buildStockCountSheet(
  rows: ReportRow[],
  supplierName: string,
  categoryName: string,
  settings?: CompanySettings
) {
  const generated = new Date().toISOString().slice(0, 10);
  const body = rows
    .map(
      (row) => `<tr>
        <td>${escapeHtml(row.product_name)}</td>
        <td class="num">${escapeHtml(row.thickness_mm)}</td>
        <td class="num">${escapeHtml(formatCell("selling_price_cents", row.selling_price_cents, settings?.default_currency ?? "USD"))}</td>
        <td>${escapeHtml(row.supplier)}</td>
        <td>${escapeHtml(row.category)}</td>
        <td>${escapeHtml(row.location)}</td>
        <td>${escapeHtml(row.unit)}</td>
        <td class="num">${escapeHtml(row.system_quantity)}</td>
        <td class="blank"></td>
        <td class="blank"></td>
      </tr>`
    )
    .join("");
  return `<!doctype html><html><head><meta charset="utf-8"><title>Stock Count Sheet</title>
  <style>
    *{box-sizing:border-box}body{font-family:Inter,"Segoe UI",Arial,sans-serif;color:#16202a;margin:14mm 11mm 16mm}
    h1{font-size:20px;margin:0 0 4px}
    .meta{font-size:12px;color:#5b6773;margin-bottom:16px}
    .meta span{margin-right:24px}
    table{width:100%;border-collapse:collapse;margin-top:8px}thead{display:table-header-group}tr{break-inside:avoid;page-break-inside:avoid}
    th,td{border:1px solid #b8c2cc;padding:6px 8px;font-size:12px;text-align:left}
    th{background:#f3f6f8}
    td.num{text-align:right}
    td.blank{height:24px;min-width:70px}
    .sign{display:flex;justify-content:space-between;margin-top:40px;font-size:13px}
    .sign div{width:45%;border-top:1px solid #16202a;padding-top:6px}
    footer{display:none}@page{size:landscape;margin:14mm 11mm 16mm}
    @media print{button{display:none}body{margin:0}footer{display:block;position:fixed;left:0;right:0;bottom:-7mm;color:#687680;font-size:9px}.page{float:right}.page:after{content:counter(page)}}
  </style></head><body>
  <button onclick="window.print()">Print / Save PDF</button>
  <h1>Physical Stock Count Sheet</h1>
  ${settings ? `<div class="meta"><strong>${escapeHtml(settings.company_name)}</strong>${settings.address ? ` · ${escapeHtml(settings.address)}` : ""}${settings.phone ? ` · ${escapeHtml(settings.phone)}` : ""}</div>` : ""}
  <div class="meta">
    <span><strong>Supplier:</strong> ${escapeHtml(supplierName)}</span>
    <span><strong>Category:</strong> ${escapeHtml(categoryName)}</span>
    <span><strong>Date generated:</strong> ${generated}</span>
    <span><strong>Items:</strong> ${rows.length}</span>
  </div>
  <table>
    <thead><tr>
      <th>Name</th><th>Thickness</th><th>Price</th><th>Supplier</th><th>Category</th><th>Location</th>
      <th>Unit</th><th>System Qty</th><th>Counted Qty</th><th>Difference</th>
    </tr></thead>
    <tbody>${body || '<tr><td colspan="10">No products.</td></tr>'}</tbody>
  </table>
  <div class="sign"><div>Prepared by: ______________________</div><div>Checked by: ______________________</div></div>
  <footer><span>Physical Stock Count Sheet</span><span class="page">Page </span></footer>
  </body></html>`;
}
