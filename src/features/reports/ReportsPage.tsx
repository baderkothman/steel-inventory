import { useMemo, useState } from "react";
import {
  Button,
  Box,
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
import { PrintDialog } from "../../components/print/PrintDialog";
import { PrintButton } from "../../components/print/PrintButton";
import { EmptyState, LoadingState } from "../../components/feedback/PageState";
import { reportOptions } from "../../lib/constants";
import { money, quantity, today } from "../../lib/formatters";
import { categoryApi, reportApi, settingsApi, supplierApi } from "../../lib/api";
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

  const columns = useMemo(() => Array.from(new Set(data.flatMap((row) => Object.keys(row)))), [data]);

  function printStockCount() {
    const supplierName = filters.supplier_id ? suppliers.find((s) => s.id === filters.supplier_id)?.name : "All suppliers";
    const categoryName = filters.category_id ? categories.find((c) => c.id === filters.category_id)?.name : "All categories";
    setCountSheetHtml(buildStockCountSheet(data, supplierName ?? "All suppliers", categoryName ?? "All categories"));
  }

  function exportCsv() {
    if (!settings) return;
    const csv = [
      columns.map((column) => csvCell(label(column))).join(","),
      ...data.map((row) =>
        columns
          .map((column) => csvCell(formatCell(column, row[column], settings.default_currency)))
          .join(",")
      )
    ].join("\n");
    const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `${report}.csv`;
    link.click();
    URL.revokeObjectURL(url);
  }

  return (
    <Stack spacing={2}>
      <PageHeader
        title="Reports"
        description="Generate operational, profit, stock, debt, expense, payment, and inventory reports."
        actions={
          <Stack direction="row" spacing={1}>
            {report === "stock_count" ? (
              <Button startIcon={<PrintIcon />} variant="outlined" disabled={!data.length} onClick={printStockCount}>Print count sheet</Button>
            ) : (
              <PrintButton
                targetId="report-table-print"
                title={reportOptions.find((option) => option.value === report)?.label ?? "Report"}
                disabled={!data.length || !settings}
                contentHasHeader={report === "stock"}
              />
            )}
            <Button startIcon={<DownloadIcon />} variant="contained" disabled={!data.length || !settings} onClick={exportCsv}>Export CSV</Button>
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

      <Paper id="report-table-print" variant="outlined">
        {isLoading || isSettingsLoading ? <LoadingState label="Loading report" /> : data.length === 0 ? <EmptyState label="No rows for this report." /> : report === "stock" && settings ? (
          <StockRemainingReport
            rows={data}
            currency={settings.default_currency}
            categoryName={
              filters.category_id
                ? categories.find((category) => category.id === filters.category_id)?.name
                : undefined
            }
          />
        ) : settings ? (
          <Table size="small">
            <TableHead><TableRow>{columns.map((column) => <TableCell key={column}>{label(column)}</TableCell>)}</TableRow></TableHead>
            <TableBody>{data.map((row, index) => <TableRow key={index}>{columns.map((column) => <TableCell key={column}>{formatCell(column, row[column], settings.default_currency)}</TableCell>)}</TableRow>)}</TableBody>
          </Table>
        ) : null}
      </Paper>

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
  return value.replace(/_/g, " ").replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function formatCell(column: string, value: unknown, currency: string) {
  if (column.endsWith("_cents") && typeof value === "number") return money(value, currency);
  if (value === null || value === undefined) return "";
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

type StockRow = {
  category: string;
  productType: string;
  productName: string;
  size: string;
  thicknessMm: number | null;
  sellingPriceCents: number;
  remainingQuantity: number;
  unit: string;
};

type StockGroup = {
  key: string;
  category: string;
  productType: string;
  rows: StockRow[];
};

function StockRemainingReport({
  rows,
  currency,
  categoryName
}: {
  rows: ReportRow[];
  currency: string;
  categoryName?: string;
}) {
  const groups = groupStockRows(rows);
  const generated = new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short"
  }).format(new Date());

  return (
    <Box className="stock-report">
      <header className="stock-report__header">
        <div>
          <p className="stock-report__eyebrow">Inventory availability</p>
          <h1>Stock remaining</h1>
          <p className="stock-report__subtitle">
            {categoryName ? `${categoryName} · ` : ""}
            Products are grouped by category and product type.
          </p>
        </div>
        <dl className="stock-report__summary">
          <div><dt>Products</dt><dd>{rows.length}</dd></div>
          <div><dt>Groups</dt><dd>{groups.length}</dd></div>
          <div><dt>Updated</dt><dd>{generated}</dd></div>
        </dl>
      </header>

      <div className="stock-report__groups">
        {groups.map((group) => (
          <section className="stock-group" key={group.key}>
            <div className="stock-group__heading">
              <div>
                <span>{group.category}</span>
                <h2>{titleCase(group.productType)}</h2>
              </div>
              <p>{group.rows.length} {group.rows.length === 1 ? "product" : "products"}</p>
            </div>
            <div className="stock-group__table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>Product name</th>
                    <th>Size</th>
                    <th>Thickness</th>
                    <th className="num">Selling price</th>
                    <th className="num">Remaining stock</th>
                  </tr>
                </thead>
                <tbody>
                  {group.rows.map((row, index) => (
                    <tr key={`${group.key}-${row.productName}-${row.size}-${row.thicknessMm ?? ""}-${index}`}>
                      <td className="product-name">{row.productName}</td>
                      <td>{row.size || "—"}</td>
                      <td>{row.thicknessMm === null ? "—" : `${quantity(row.thicknessMm)} mm`}</td>
                      <td className="num price">{money(row.sellingPriceCents, currency)}</td>
                      <td className="num stock-quantity">
                        <strong>{quantity(row.remainingQuantity)}</strong>
                        <span>{row.unit}</span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>
        ))}
      </div>
    </Box>
  );
}

function groupStockRows(rows: ReportRow[]): StockGroup[] {
  const groups = new Map<string, StockGroup>();
  for (const raw of rows) {
    const row: StockRow = {
      category: textValue(raw.category, "Uncategorized"),
      productType: textValue(raw.product_type, "Other"),
      productName: textValue(raw.product_name, "Unnamed product"),
      size: textValue(raw.size),
      thicknessMm: numberValue(raw.thickness_mm),
      sellingPriceCents: numberValue(raw.selling_price_cents) ?? 0,
      remainingQuantity: numberValue(raw.remaining_quantity) ?? 0,
      unit: textValue(raw.unit)
    };
    const key = `${row.category}\u0000${row.productType}`;
    const group = groups.get(key) ?? {
      key,
      category: row.category,
      productType: row.productType,
      rows: []
    };
    group.rows.push(row);
    groups.set(key, group);
  }

  return [...groups.values()]
    .map((group) => ({
      ...group,
      rows: group.rows.sort((left, right) =>
        left.productName.localeCompare(right.productName, undefined, { numeric: true })
      )
    }))
    .sort((left, right) =>
      left.category.localeCompare(right.category, undefined, { numeric: true }) ||
      left.productType.localeCompare(right.productType, undefined, { numeric: true })
    );
}

function textValue(value: unknown, fallback = "") {
  return typeof value === "string" && value.trim() ? value.trim() : fallback;
}

function numberValue(value: unknown) {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function titleCase(value: string) {
  return value.replace(/[_-]/g, " ").replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function csvCell(value: unknown) {
  const text = value === null || value === undefined ? "" : String(value);
  return `"${text.replace(/"/g, '""')}"`;
}

function escapeHtml(value: unknown) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

/** Builds a print-friendly physical stock-count sheet with blank counted/difference
 *  columns and prepared-by / checked-by fields for manual entry on paper. */
function buildStockCountSheet(rows: ReportRow[], supplierName: string, categoryName: string) {
  const generated = new Date().toISOString().slice(0, 10);
  const body = rows
    .map(
      (row) => `<tr>
        <td>${escapeHtml(row.product)}</td>
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
    body{font-family:Arial,sans-serif;color:#16202a;margin:24px}
    h1{font-size:20px;margin:0 0 4px}
    .meta{font-size:12px;color:#5b6773;margin-bottom:16px}
    .meta span{margin-right:24px}
    table{width:100%;border-collapse:collapse;margin-top:8px}
    th,td{border:1px solid #b8c2cc;padding:6px 8px;font-size:12px;text-align:left}
    th{background:#f3f6f8}
    td.num{text-align:right}
    td.blank{height:24px;min-width:70px}
    .sign{display:flex;justify-content:space-between;margin-top:40px;font-size:13px}
    .sign div{width:45%;border-top:1px solid #16202a;padding-top:6px}
    @media print{button{display:none}}
  </style></head><body>
  <button onclick="window.print()">Print / Save PDF</button>
  <h1>Physical Stock Count Sheet</h1>
  <div class="meta">
    <span><strong>Supplier:</strong> ${escapeHtml(supplierName)}</span>
    <span><strong>Category:</strong> ${escapeHtml(categoryName)}</span>
    <span><strong>Date generated:</strong> ${generated}</span>
    <span><strong>Items:</strong> ${rows.length}</span>
  </div>
  <table>
    <thead><tr>
      <th>Product</th><th>Supplier</th><th>Category</th><th>Location</th>
      <th>Unit</th><th>System Qty</th><th>Counted Qty</th><th>Difference</th>
    </tr></thead>
    <tbody>${body || '<tr><td colspan="8">No products.</td></tr>'}</tbody>
  </table>
  <div class="sign"><div>Prepared by: ______________________</div><div>Checked by: ______________________</div></div>
  </body></html>`;
}
