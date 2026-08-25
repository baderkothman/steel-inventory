import { useId, useState } from "react";
import {
  Alert,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Snackbar
} from "@mui/material";
import PrintIcon from "@mui/icons-material/Print";

type PrintDialogProps = {
  open: boolean;
  html: string;
  onClose: () => void;
};

export function PrintDialog({ open, html, onClose }: PrintDialogProps) {
  const frameId = useId();
  const [printNotice, setPrintNotice] = useState<{
    message: string;
    severity: "info" | "error";
  } | null>(null);

  function handlePrint() {
    const frame = document.getElementById(frameId) as HTMLIFrameElement | null;
    if (!frame?.contentWindow) {
      setPrintNotice({
        severity: "error",
        message: "The print preview is not ready. Please wait a moment and try again."
      });
      return;
    }

    setPrintNotice({
      severity: "info",
      message: "Print request sent. Choose “Save as PDF” in the system dialog to create a PDF."
    });
    window.setTimeout(() => {
      try {
        frame.contentWindow?.focus();
        frame.contentWindow?.print();
      } catch {
        setPrintNotice({
          severity: "error",
          message: "The print dialog could not be opened. Please try again."
        });
      }
    }, 150);
  }

  return (
    <>
      <Dialog open={open} onClose={onClose} fullWidth maxWidth="lg">
        <DialogTitle>Print preview</DialogTitle>
        <DialogContent sx={{ height: "72vh", p: 0 }}>
          <iframe
            id={frameId}
            title="Print preview"
            srcDoc={html}
            // The preview is app-generated markup with no scripts. `allow-same-origin`
            // lets this window reach `contentWindow.print()`; `allow-modals` keeps the
            // sandboxed-modals flag from turning that call into a no-op. Scripting stays
            // disabled, so the frame cannot act on the same-origin grant.
            sandbox="allow-same-origin allow-modals"
            style={{ border: 0, width: "100%", height: "100%" }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={onClose}>Close</Button>
          <Button startIcon={<PrintIcon />} variant="contained" onClick={handlePrint}>
            Print / Save PDF
          </Button>
        </DialogActions>
      </Dialog>
      <Snackbar
        open={Boolean(printNotice)}
        autoHideDuration={6000}
        onClose={() => setPrintNotice(null)}
        anchorOrigin={{ vertical: "bottom", horizontal: "center" }}
      >
        <Alert
          severity={printNotice?.severity ?? "info"}
          variant="filled"
          onClose={() => setPrintNotice(null)}
          sx={{ width: "100%" }}
        >
          {printNotice?.message}
        </Alert>
      </Snackbar>
    </>
  );
}
