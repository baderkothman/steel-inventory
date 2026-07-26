import { strToU8, zipSync } from "fflate";

export type ExportColumn = {
  label: string;
  values: string[];
};

export type TableExportData = {
  title: string;
  filename: string;
  columns: ExportColumn[];
  companyName?: string;
  companyDetails?: string;
};

export function exportCsv(data: TableExportData) {
  const rows = transpose(data.columns);
  const csv = [
    data.columns.map((column) => csvCell(column.label)).join(","),
    ...rows.map((row) => row.map(csvCell).join(","))
  ].join("\r\n");
  download(new Blob([`\uFEFF${csv}`], { type: "text/csv;charset=utf-8" }), `${data.filename}.csv`);
}

export function exportXlsx(data: TableExportData) {
  const rows = [data.columns.map((column) => column.label), ...transpose(data.columns)];
  const sheetRows = rows
    .map(
      (row, rowIndex) =>
        `<row r="${rowIndex + 1}">${row
          .map((value, columnIndex) => {
            const ref = `${columnName(columnIndex + 1)}${rowIndex + 1}`;
            return `<c r="${ref}" t="inlineStr"${rowIndex === 0 ? ' s="1"' : ""}><is><t xml:space="preserve">${xml(value)}</t></is></c>`;
          })
          .join("")}</row>`
    )
    .join("");
  const widths = data.columns
    .map((column, index) => {
      const longest = Math.max(column.label.length, ...column.values.map((value) => value.length));
      return `<col min="${index + 1}" max="${index + 1}" width="${Math.min(Math.max(longest + 2, 12), 42)}" customWidth="1"/>`;
    })
    .join("");
  const files: Record<string, Uint8Array> = {
    "[Content_Types].xml": strToU8(
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/><Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/></Types>`
    ),
    "_rels/.rels": strToU8(
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>`
    ),
    "xl/workbook.xml": strToU8(
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="${xml(data.title.slice(0, 31))}" sheetId="1" r:id="rId1"/></sheets></workbook>`
    ),
    "xl/_rels/workbook.xml.rels": strToU8(
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>`
    ),
    "xl/styles.xml": strToU8(
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><fonts count="2"><font><sz val="11"/><name val="Aptos"/></font><font><b/><color rgb="FFFFFFFF"/><sz val="11"/><name val="Aptos"/></font></fonts><fills count="3"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill><fill><patternFill patternType="solid"><fgColor rgb="FF245A61"/><bgColor indexed="64"/></patternFill></fill></fills><borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders><cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs><cellXfs count="2"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/><xf numFmtId="0" fontId="1" fillId="2" borderId="0" xfId="0" applyFont="1" applyFill="1"/></cellXfs></styleSheet>`
    ),
    "xl/worksheets/sheet1.xml": strToU8(
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetViews><sheetView workbookViewId="0"><pane ySplit="1" topLeftCell="A2" activePane="bottomLeft" state="frozen"/></sheetView></sheetViews><cols>${widths}</cols><sheetData>${sheetRows}</sheetData><autoFilter ref="A1:${columnName(data.columns.length)}${Math.max(rows.length, 1)}"/></worksheet>`
    )
  };
  const archive = zipSync(files, { level: 6 });
  download(
    new Blob([archive.buffer as ArrayBuffer], {
      type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    }),
    `${data.filename}.xlsx`
  );
}

export async function exportPdf(data: TableExportData) {
  const [{ jsPDF }, { default: autoTable }] = await Promise.all([
    import("jspdf"),
    import("jspdf-autotable")
  ]);
  const landscape = data.columns.length > 6;
  const doc = new jsPDF({ orientation: landscape ? "landscape" : "portrait", unit: "mm", format: "a4" });
  const pageWidth = doc.internal.pageSize.getWidth();
  const pageHeight = doc.internal.pageSize.getHeight();
  const generated = new Date().toLocaleString();

  autoTable(doc, {
    head: [data.columns.map((column) => column.label)],
    body: transpose(data.columns),
    startY: 31,
    margin: { top: 31, right: 10, bottom: 15, left: 10 },
    theme: "grid",
    styles: { fontSize: landscape ? 7.5 : 8.5, cellPadding: 2.2, textColor: [24, 35, 45], overflow: "linebreak" },
    headStyles: { fillColor: [36, 90, 97], textColor: 255, fontStyle: "bold" },
    alternateRowStyles: { fillColor: [246, 249, 250] },
    didDrawPage: ({ pageNumber }) => {
      doc.setTextColor(22, 32, 42);
      doc.setFontSize(15);
      doc.setFont("helvetica", "bold");
      doc.text(data.title, 10, 13);
      doc.setFont("helvetica", "normal");
      doc.setFontSize(8);
      doc.setTextColor(91, 103, 115);
      if (data.companyName) doc.text(data.companyName, 10, 19);
      if (data.companyDetails) doc.text(data.companyDetails, 10, 24);
      doc.text(`Generated ${generated}`, pageWidth - 10, 19, { align: "right" });
      doc.text(`Page ${pageNumber}`, pageWidth - 10, pageHeight - 7, { align: "right" });
    }
  });
  doc.save(`${data.filename}.pdf`);
}

export function buildTablePrintDocument(data: TableExportData) {
  const header = data.columns.map((column) => `<th>${html(column.label)}</th>`).join("");
  const body = transpose(data.columns)
    .map((row) => `<tr>${row.map((value) => `<td>${html(value)}</td>`).join("")}</tr>`)
    .join("");
  return `<!doctype html><html><head><meta charset="utf-8"><title>${html(data.title)}</title><style>
  *{box-sizing:border-box}body{margin:0;font-family:Inter,"Segoe UI",Arial,sans-serif;color:#18232d;font-size:10px}
  header{margin-bottom:8mm;padding-bottom:4mm;border-bottom:1.5px solid #245a61}
  h1{margin:0;font-size:18px;letter-spacing:-.01em}header p{margin:2px 0 0;color:#5b6773}
  table{width:100%;border-collapse:collapse;table-layout:auto}thead{display:table-header-group}
  th{background:#e9f0f1;color:#20383c;font-weight:700;text-align:left}
  th,td{padding:5px 6px;border-bottom:1px solid #dbe3ea;vertical-align:top;overflow-wrap:anywhere}
  tbody tr:nth-child(even){background:#f8fafb}tr{break-inside:avoid;page-break-inside:avoid}
  footer{position:fixed;bottom:-5mm;left:0;right:0;color:#687680;font-size:8px}
  footer .page:after{content:counter(page)}footer span:last-child{float:right}
  @page{size:auto;margin:14mm 11mm 16mm}
  </style></head><body><header><h1>${html(data.title)}</h1>
  ${data.companyName ? `<p>${html(data.companyName)}</p>` : ""}
  ${data.companyDetails ? `<p>${html(data.companyDetails)}</p>` : ""}
  <p>${data.columns[0]?.values.length ?? 0} records · Generated ${html(new Date().toLocaleString())}</p>
  </header><table><thead><tr>${header}</tr></thead><tbody>${body}</tbody></table>
  <footer><span>Page <span class="page"></span></span><span>${html(data.title)}</span></footer></body></html>`;
}

function transpose(columns: ExportColumn[]) {
  const count = columns[0]?.values.length ?? 0;
  return Array.from({ length: count }, (_, rowIndex) =>
    columns.map((column) => column.values[rowIndex] ?? "")
  );
}

function csvCell(value: string) {
  return `"${value.replace(/"/g, '""')}"`;
}

function columnName(index: number) {
  let name = "";
  for (let value = index; value > 0; value = Math.floor((value - 1) / 26)) {
    name = String.fromCharCode(65 + ((value - 1) % 26)) + name;
  }
  return name;
}

function xml(value: string) {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

function html(value: string) {
  return xml(value).replace(/\n/g, "<br>");
}

function download(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}
