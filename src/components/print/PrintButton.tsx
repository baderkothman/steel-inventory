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
};

export function PrintButton({
  targetId,
  title,
  subtitle,
  disabled = false,
  label = "Print"
}: PrintButtonProps) {
  const [html, setHtml] = useState("");

  function preparePrint() {
    const target = document.getElementById(targetId);
    if (!target) return;

    const content = target.cloneNode(true) as HTMLElement;
    content.querySelectorAll(".print-exclude, button").forEach((node) => node.remove());
    setHtml(buildPrintDocument(title, subtitle, content.innerHTML));
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

function buildPrintDocument(title: string, subtitle: string | undefined, content: string) {
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
      button,.print-exclude{display:none!important}
      @page{size:auto;margin:10mm}
    </style>
  </head>
  <body>
    <h1>${escapeHtml(title)}</h1>
    ${subtitle ? `<p class="subtitle">${escapeHtml(subtitle)}</p>` : ""}
    <div class="generated">Printed ${escapeHtml(generated)}</div>
    ${content}
  </body>
</html>`;
}
