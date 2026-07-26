import { ReactNode, useMemo, useState } from "react";
import {
  Box,
  Button,
  Checkbox,
  Divider,
  IconButton,
  ListItemIcon,
  ListItemText,
  Menu,
  MenuItem,
  Paper,
  Skeleton,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TablePagination,
  TableRow,
  TableSortLabel,
  TextField,
  Tooltip,
  Typography
} from "@mui/material";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import ViewColumnOutlinedIcon from "@mui/icons-material/ViewColumnOutlined";
import DownloadOutlinedIcon from "@mui/icons-material/DownloadOutlined";
import PrintOutlinedIcon from "@mui/icons-material/PrintOutlined";
import SearchIcon from "@mui/icons-material/Search";
import { useQuery } from "@tanstack/react-query";

import { settingsApi } from "../../lib/api";
import { PrintDialog } from "../print/PrintDialog";
import {
  buildTablePrintDocument,
  exportCsv,
  exportPdf,
  exportXlsx,
  type TableExportData
} from "./tableExport";

export type TableColumn<T> = {
  id: string;
  label: string;
  value: (row: T) => string | number | null | undefined;
  render?: (row: T) => ReactNode;
  align?: "left" | "right" | "center";
  minWidth?: number;
  width?: number;
  sortable?: boolean;
  hideable?: boolean;
  defaultHidden?: boolean;
};

export type RowAction<T> = {
  label: string;
  icon?: ReactNode;
  onClick: (row: T) => void;
  disabled?: boolean;
  destructive?: boolean;
  dividerBefore?: boolean;
};

type EnterpriseTableProps<T> = {
  rows: T[];
  columns: TableColumn<T>[];
  rowId: (row: T) => number | string;
  title: string;
  filename?: string;
  loading?: boolean;
  actions?: (row: T) => RowAction<T>[];
  emptyTitle?: string;
  emptyDescription?: string;
  searchPlaceholder?: string;
  searchable?: boolean;
  selectable?: boolean;
  maxHeight?: number | string;
  initialSort?: { column: string; direction: "asc" | "desc" };
  pageSize?: number;
  compact?: boolean;
  toolbarExtras?: ReactNode;
  fillHeight?: boolean;
};

export function EnterpriseTable<T>({
  rows,
  columns,
  rowId,
  title,
  filename = slug(title),
  loading = false,
  actions,
  emptyTitle = "No records yet",
  emptyDescription = "New records will appear here.",
  searchPlaceholder = `Search ${title.toLowerCase()}`,
  searchable = true,
  selectable = true,
  maxHeight = "min(62vh, 680px)",
  initialSort,
  pageSize = 10,
  compact = false,
  toolbarExtras,
  fillHeight = false
}: EnterpriseTableProps<T>) {
  const { data: settings } = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get });
  const [search, setSearch] = useState("");
  const [sortColumn, setSortColumn] = useState(initialSort?.column ?? "");
  const [sortDirection, setSortDirection] = useState<"asc" | "desc">(initialSort?.direction ?? "asc");
  const [page, setPage] = useState(0);
  const [rowsPerPage, setRowsPerPage] = useState(pageSize);
  const [selected, setSelected] = useState<Set<number | string>>(new Set());
  const [hiddenColumns, setHiddenColumns] = useState<Set<string>>(
    new Set(columns.filter((column) => column.defaultHidden).map((column) => column.id))
  );
  const [columnsAnchor, setColumnsAnchor] = useState<HTMLElement | null>(null);
  const [exportAnchor, setExportAnchor] = useState<HTMLElement | null>(null);
  const [printHtml, setPrintHtml] = useState("");

  const visibleColumns = useMemo(
    () => columns.filter((column) => !hiddenColumns.has(column.id)),
    [columns, hiddenColumns]
  );
  const filteredRows = useMemo(() => {
    const needle = search.trim().toLocaleLowerCase();
    if (!needle) return rows;
    return rows.filter((row) =>
      columns.some((column) => String(column.value(row) ?? "").toLocaleLowerCase().includes(needle))
    );
  }, [columns, rows, search]);
  const sortedRows = useMemo(() => {
    if (!sortColumn) return filteredRows;
    const column = columns.find((candidate) => candidate.id === sortColumn);
    if (!column) return filteredRows;
    return [...filteredRows].sort((left, right) => {
      const first = column.value(left);
      const second = column.value(right);
      const result =
        typeof first === "number" && typeof second === "number"
          ? first - second
          : String(first ?? "").localeCompare(String(second ?? ""), undefined, {
              numeric: true,
              sensitivity: "base"
            });
      return sortDirection === "asc" ? result : -result;
    });
  }, [columns, filteredRows, sortColumn, sortDirection]);
  const exportRows = selected.size
    ? sortedRows.filter((row) => selected.has(rowId(row)))
    : sortedRows;
  const safePage = Math.min(page, Math.max(Math.ceil(sortedRows.length / rowsPerPage) - 1, 0));
  const visiblePage = sortedRows.slice(
    safePage * rowsPerPage,
    safePage * rowsPerPage + rowsPerPage
  );
  const allPageSelected =
    visiblePage.length > 0 && visiblePage.every((row) => selected.has(rowId(row)));
  const exportData = (): TableExportData => ({
    title,
    filename,
    companyName: settings?.company_name,
    companyDetails: [settings?.address, settings?.phone, settings?.email].filter(Boolean).join(" · "),
    columns: visibleColumns.map((column) => ({
      label: column.label,
      values: exportRows.map((row) => String(column.value(row) ?? ""))
    }))
  });

  function toggleSort(column: TableColumn<T>) {
    if (column.sortable === false) return;
    if (sortColumn === column.id) {
      setSortDirection((current) => (current === "asc" ? "desc" : "asc"));
    } else {
      setSortColumn(column.id);
      setSortDirection("asc");
    }
    setPage(0);
  }

  function togglePageSelection() {
    const next = new Set(selected);
    visiblePage.forEach((row) => {
      const id = rowId(row);
      if (allPageSelected) next.delete(id);
      else next.add(id);
    });
    setSelected(next);
  }

  return (
    <Paper
      variant="outlined"
      sx={{
        overflow: "hidden",
        width: "100%",
        maxWidth: "100%",
        minWidth: 0,
        ...(fillHeight ? { height: "100%", display: "flex", flexDirection: "column" } : {})
      }}
    >
      <Stack
        direction={{ xs: "column", md: "row" }}
        alignItems={{ xs: "stretch", md: "center" }}
        justifyContent="space-between"
        gap={1.25}
        sx={{ px: 1.5, py: 1.25, borderBottom: "1px solid", borderColor: "divider" }}
        className="print-exclude"
      >
        <Stack direction="row" alignItems="center" spacing={1} sx={{ minWidth: 0, flex: 1 }}>
          {searchable ? (
            <TextField
              value={search}
              onChange={(event) => {
                setSearch(event.target.value);
                setPage(0);
              }}
              placeholder={searchPlaceholder}
              aria-label={searchPlaceholder}
              InputProps={{ startAdornment: <SearchIcon color="action" sx={{ mr: 1, fontSize: 20 }} /> }}
              sx={{ width: { xs: "100%", md: 300 } }}
            />
          ) : null}
          {selected.size ? (
            <Typography variant="body2" color="primary.main" fontWeight={700} whiteSpace="nowrap">
              {selected.size} selected
            </Typography>
          ) : null}
          {toolbarExtras}
        </Stack>
        <Stack direction="row" spacing={0.5} justifyContent="flex-end" flexWrap="wrap">
          <Tooltip title="Choose visible columns">
            <Button
              size="small"
              startIcon={<ViewColumnOutlinedIcon />}
              onClick={(event) => setColumnsAnchor(event.currentTarget)}
            >
              Columns
            </Button>
          </Tooltip>
          <Button
            size="small"
            startIcon={<DownloadOutlinedIcon />}
            disabled={!exportRows.length}
            onClick={(event) => setExportAnchor(event.currentTarget)}
          >
            Export
          </Button>
          <Button
            size="small"
            startIcon={<PrintOutlinedIcon />}
            disabled={!exportRows.length}
            onClick={() => setPrintHtml(buildTablePrintDocument(exportData()))}
          >
            Print
          </Button>
        </Stack>
      </Stack>

      <TableContainer sx={{ maxHeight, overflow: "auto", width: "100%", ...(fillHeight ? { flex: 1 } : {}) }}>
        <Table stickyHeader size={compact ? "small" : "medium"} aria-label={title} sx={{ width: "100%" }}>
          <TableHead>
            <TableRow>
              {selectable ? (
                <TableCell padding="checkbox" sx={{ width: 48 }}>
                  <Checkbox
                    size="small"
                    checked={allPageSelected}
                    indeterminate={!allPageSelected && visiblePage.some((row) => selected.has(rowId(row)))}
                    onChange={togglePageSelection}
                    inputProps={{ "aria-label": `Select visible ${title.toLowerCase()}` }}
                  />
                </TableCell>
              ) : null}
              {visibleColumns.map((column) => (
                <TableCell
                  key={column.id}
                  align={column.align}
                  sortDirection={sortColumn === column.id ? sortDirection : false}
                  sx={{ minWidth: column.minWidth, width: column.width }}
                >
                  {column.sortable === false ? (
                    column.label
                  ) : (
                    <TableSortLabel
                      active={sortColumn === column.id}
                      direction={sortColumn === column.id ? sortDirection : "asc"}
                      onClick={() => toggleSort(column)}
                    >
                      {column.label}
                    </TableSortLabel>
                  )}
                </TableCell>
              ))}
              {actions ? (
                <TableCell
                  align="right"
                  className="print-exclude sticky-actions"
                  sx={{
                    position: "sticky",
                    right: 0,
                    zIndex: 5,
                    width: 76,
                    minWidth: 76,
                    bgcolor: "#f2f6f7",
                    boxShadow: "-8px 0 10px -10px rgba(22,32,42,0.65)"
                  }}
                >
                  Actions
                </TableCell>
              ) : null}
            </TableRow>
          </TableHead>
          <TableBody>
            {loading
              ? Array.from({ length: Math.min(rowsPerPage, 8) }, (_, index) => (
                  <TableRow key={index}>
                    {selectable ? <TableCell padding="checkbox"><Skeleton variant="rounded" width={18} height={18} /></TableCell> : null}
                    {visibleColumns.map((column) => <TableCell key={column.id}><Skeleton /></TableCell>)}
                    {actions ? (
                      <TableCell
                        className="sticky-actions"
                        sx={{
                          position: "sticky",
                          right: 0,
                          zIndex: 1,
                          width: 76,
                          minWidth: 76,
                          bgcolor: "background.paper"
                        }}
                      >
                        <Skeleton />
                      </TableCell>
                    ) : null}
                  </TableRow>
                ))
              : visiblePage.map((row) => {
                  const id = rowId(row);
                  const isSelected = selected.has(id);
                  return (
                    <TableRow
                      key={id}
                      hover
                      selected={isSelected}
                      sx={{
                        "&:hover .sticky-actions": { bgcolor: "#f2f7f7" },
                        "&.Mui-selected .sticky-actions": { bgcolor: "#e5f0f1" }
                      }}
                    >
                      {selectable ? (
                        <TableCell padding="checkbox">
                          <Checkbox
                            size="small"
                            checked={isSelected}
                            onChange={() => {
                              const next = new Set(selected);
                              if (isSelected) next.delete(id);
                              else next.add(id);
                              setSelected(next);
                            }}
                            inputProps={{ "aria-label": `Select row ${id}` }}
                          />
                        </TableCell>
                      ) : null}
                      {visibleColumns.map((column) => (
                        <TableCell key={column.id} align={column.align}>
                          {column.render ? column.render(row) : String(column.value(row) ?? "—")}
                        </TableCell>
                      ))}
                      {actions ? (
                        <TableCell
                          align="right"
                          className="print-exclude sticky-actions"
                          sx={{
                            position: "sticky",
                            right: 0,
                            zIndex: 1,
                            width: 76,
                            minWidth: 76,
                            bgcolor: "background.paper",
                            boxShadow: "-8px 0 10px -10px rgba(22,32,42,0.65)"
                          }}
                        >
                          <RowActionsMenu row={row} actions={actions(row)} />
                        </TableCell>
                      ) : null}
                    </TableRow>
                  );
                })}
          </TableBody>
        </Table>
        {!loading && !sortedRows.length ? (
          <Box sx={{ px: 3, py: 7, textAlign: "center" }}>
            <Typography variant="h6">{search ? "No matching records" : emptyTitle}</Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.75 }}>
              {search ? "Try a different search term or clear the search field." : emptyDescription}
            </Typography>
          </Box>
        ) : null}
      </TableContainer>
      <TablePagination
        className="print-exclude"
        component="div"
        count={sortedRows.length}
        page={safePage}
        onPageChange={(_, nextPage) => setPage(nextPage)}
        rowsPerPage={rowsPerPage}
        onRowsPerPageChange={(event) => {
          setRowsPerPage(Number(event.target.value));
          setPage(0);
        }}
        rowsPerPageOptions={[10, 25, 50, 100]}
        labelRowsPerPage="Rows per page"
      />

      <Menu anchorEl={columnsAnchor} open={Boolean(columnsAnchor)} onClose={() => setColumnsAnchor(null)}>
        {columns.filter((column) => column.hideable !== false).map((column) => (
          <MenuItem
            key={column.id}
            onClick={() => {
              const next = new Set(hiddenColumns);
              if (next.has(column.id)) next.delete(column.id);
              else if (visibleColumns.length > 1) next.add(column.id);
              setHiddenColumns(next);
            }}
          >
            <Checkbox size="small" checked={!hiddenColumns.has(column.id)} />
            <ListItemText>{column.label}</ListItemText>
          </MenuItem>
        ))}
      </Menu>
      <Menu anchorEl={exportAnchor} open={Boolean(exportAnchor)} onClose={() => setExportAnchor(null)}>
        <MenuItem onClick={() => { exportCsv(exportData()); setExportAnchor(null); }}>CSV</MenuItem>
        <MenuItem onClick={() => { exportXlsx(exportData()); setExportAnchor(null); }}>Excel (.xlsx)</MenuItem>
        <MenuItem onClick={() => { void exportPdf(exportData()); setExportAnchor(null); }}>PDF</MenuItem>
      </Menu>
      <PrintDialog open={Boolean(printHtml)} html={printHtml} onClose={() => setPrintHtml("")} />
    </Paper>
  );
}

export function RowActionsMenu<T>({ row, actions }: { row: T; actions: RowAction<T>[] }) {
  const [anchor, setAnchor] = useState<HTMLElement | null>(null);
  const validActions = actions.filter((action) => !action.disabled);
  if (!validActions.length) return null;
  return (
    <>
      <Tooltip title="Open actions">
        <IconButton
          size="small"
          aria-label="Open row actions"
          aria-haspopup="menu"
          onClick={(event) => setAnchor(event.currentTarget)}
        >
          <MoreHorizIcon />
        </IconButton>
      </Tooltip>
      <Menu anchorEl={anchor} open={Boolean(anchor)} onClose={() => setAnchor(null)}>
        {validActions.map((action, index) => (
          <Box key={`${action.label}-${index}`}>
            {action.dividerBefore || (action.destructive && index > 0) ? <Divider /> : null}
            <MenuItem
              onClick={() => {
                setAnchor(null);
                action.onClick(row);
              }}
              sx={action.destructive ? { color: "error.main" } : undefined}
            >
              {action.icon ? <ListItemIcon sx={action.destructive ? { color: "error.main" } : undefined}>{action.icon}</ListItemIcon> : null}
              <ListItemText>{action.label}</ListItemText>
            </MenuItem>
          </Box>
        ))}
      </Menu>
    </>
  );
}

function slug(value: string) {
  return value.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "");
}
