import { useState } from "react";
import { Button } from "@mui/material";
import PrintIcon from "@mui/icons-material/Print";

import { PrintDialog } from "./PrintDialog";

type PrintButtonProps = {
  targetId: string;
  title: string;
  subtitle?: string;
  disabled?: boolean;
  label?: string;
  contentHasHeader?: boolean;
};

export function PrintButton({
  targetId,
  title,
  subtitle,
  disabled = false,
  label = "Print",
  contentHasHeader = false
}: PrintButtonProps) {
  const [html, setHtml] = useState("");

  function preparePrint() {
    const target = document.getElementById(targetId);
    if (!target) return;

    const content = target.cloneNode(true) as HTMLElement;
    content.querySelectorAll(".print-exclude, button").forEach((node) => node.remove());
    setHtml(buildPrintDocument(title, subtitle, content.innerHTML, contentHasHeader));
  }

  return (
    <>
      <Button
        className="print-exclude"
        startIcon={<PrintIcon />}
        variant="outlined"
        disabled={disabled}
        onClick={preparePrint}
      >
        {label}
      </Button>
      <PrintDialog open={Boolean(html)} html={html} onClose={() => setHtml("")} />
    </>
  );
}

function escapeHtml(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function buildPrintDocument(
  title: string,
  subtitle: string | undefined,
  content: string,
  contentHasHeader: boolean
) {
  const generated = new Date().toLocaleString();
  return `<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>${escapeHtml(title)}</title>
    <style>
      *{box-sizing:border-box}
      body{font-family:Arial,sans-serif;color:#16202a;margin:12mm;font-size:12px}
      h1{font-size:21px;margin:0 0 4px}
      .subtitle{color:#5b6773;margin:0 0 3px}
      .generated{color:#5b6773;font-size:10px;margin-bottom:16px}
      table{width:100%;border-collapse:collapse}
      th,td{border:1px solid #b8c2cc;padding:6px 8px;text-align:left;vertical-align:top}
      th{background:#f2f5f7;font-weight:700}
      td[align="right"],th[align="right"]{text-align:right}
      .MuiPaper-root,.MuiCard-root{box-shadow:none!important;border:0!important}
      .MuiGrid-container{display:grid;grid-template-columns:repeat(4,1fr);gap:10px;margin-bottom:16px}
      .MuiCard-root{border:1px solid #b8c2cc!important;padding:8px}
      .MuiCardContent-root{padding:6px!important}
      .MuiTypography-h5{font-size:16px;font-weight:700}
      .MuiTypography-h6{font-size:14px;font-weight:700;margin:14px 0 6px}
      .stock-report{font-family:Inter,"Segoe UI",Arial,sans-serif;color:#16202a}
      .stock-report__header{display:flex;align-items:flex-start;justify-content:space-between;gap:24px;padding:0 0 16px;border-bottom:2px solid #244f55}
      .stock-report__eyebrow{margin:0 0 4px;color:#1f6f78;font-size:9px;font-weight:800;letter-spacing:.12em;text-transform:uppercase}
      .stock-report__header h1{margin:0;font-size:23px;line-height:1.15;letter-spacing:-.025em}
      .stock-report__subtitle{margin:5px 0 0;color:#5b6773;font-size:10px}
      .stock-report__summary{display:flex;gap:18px;margin:0}
      .stock-report__summary div{min-width:58px}
      .stock-report__summary dt{color:#6b7782;font-size:8px;font-weight:800;letter-spacing:.08em;text-transform:uppercase}
      .stock-report__summary dd{margin:3px 0 0;font-size:10px;font-weight:750;font-variant-numeric:tabular-nums}
      .stock-report__groups{display:grid;gap:14px;margin-top:16px}
      .stock-group{break-inside:avoid;page-break-inside:avoid}
      .stock-group__heading{display:flex;align-items:end;justify-content:space-between;padding:7px 9px;background:#e8f0f1;border:1px solid #c9dadd}
      .stock-group__heading span{display:block;color:#557078;font-size:8px;font-weight:800;letter-spacing:.08em;text-transform:uppercase}
      .stock-group__heading h2{margin:1px 0 0;font-size:13px;line-height:1.15}
      .stock-group__heading p{margin:0;color:#52616d;font-size:9px}
      .stock-group table{width:100%;border-collapse:collapse;table-layout:fixed}
      .stock-group th,.stock-group td{border:0;border-bottom:1px solid #dbe3e6;padding:6px 9px;vertical-align:middle}
      .stock-group th{background:#f7f9fa;color:#66737e;font-size:8px;font-weight:800;letter-spacing:.045em;text-transform:uppercase}
      .stock-group td{font-size:10px}
      .stock-group th:first-child,.stock-group td:first-child{width:38%}
      .stock-group th:nth-child(2),.stock-group td:nth-child(2){width:15%}
      .stock-group th:nth-child(3),.stock-group td:nth-child(3){width:15%}
      .stock-group th:nth-child(4),.stock-group td:nth-child(4){width:16%}
      .stock-group th:nth-child(5),.stock-group td:nth-child(5){width:16%}
      .stock-group .num{text-align:right;font-variant-numeric:tabular-nums}
      .stock-group .product-name{font-weight:700}
      .stock-group .price{white-space:nowrap}
      .stock-quantity strong{font-size:11px}
      .stock-quantity span{margin-left:4px;color:#68747e;font-size:8px}
      button,.print-exclude{display:none!important}
      @page{size:auto;margin:10mm}
    </style>
  </head>
  <body>
    ${contentHasHeader ? "" : `<h1>${escapeHtml(title)}</h1>`}
    ${contentHasHeader || !subtitle ? "" : `<p class="subtitle">${escapeHtml(subtitle)}</p>`}
    ${contentHasHeader ? "" : `<div class="generated">Printed ${escapeHtml(generated)}</div>`}
    ${content}
  </body>
</html>`;
}
