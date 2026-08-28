import { reportOptions } from "../../lib/constants";

export type ReportKey = (typeof reportOptions)[number]["value"];

export function orderReportColumns(report: ReportKey, columns: string[], preferred?: readonly string[]) {
  if (report !== "stock" && report !== "stock_count") return columns;
  const productOrder = preferred ?? ["product_name", "thickness_mm", "selling_price_cents"];
  return [
    ...productOrder.filter((column) => columns.includes(column)),
    ...columns.filter((column) => !productOrder.includes(column))
  ];
}
