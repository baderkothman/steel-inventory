import { useEffect, useId, useState } from "react";
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
  const [frameReady, setFrameReady] = useState(false);
  const [printNotice, setPrintNotice] = useState<{
    message: string;
    severity: "info" | "error";
  } | null>(null);

  useEffect(() => {
    setFrameReady(false);
  }, [html, open]);

  async function handlePrint() {
    const frame = document.getElementById(frameId) as HTMLIFrameElement | null;
    if (!frame?.contentWindow) {
      setPrintNotice({
        severity: "error",
        message: "The print preview is not ready. Please wait a moment and try again."
      });
      return;
    }

    try {
      await waitForImages(frame.contentDocument);
      setPrintNotice({
        severity: "info",
        message: "Print request sent. Choose “Save as PDF” in the system dialog to create a PDF."
      });
      frame.contentWindow.focus();
      frame.contentWindow.print();
    } catch {
      setPrintNotice({
        severity: "error",
        message: "The print preview could not finish loading. Please try again."
      });
    }
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
            onLoad={() => setFrameReady(true)}
            style={{ border: 0, width: "100%", height: "100%" }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={onClose}>Close</Button>
          <Button startIcon={<PrintIcon />} variant="contained" onClick={() => void handlePrint()} disabled={!frameReady}>
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

async function waitForImages(document: Document | null) {
  if (!document) return;
  const images = Array.from(document.images);
  await Promise.all(images.map(async (image) => {
    if (!image.complete) {
      await new Promise<void>((resolve) => {
        const done = () => resolve();
        image.addEventListener("load", done, { once: true });
        image.addEventListener("error", done, { once: true });
        window.setTimeout(done, 3000);
      });
    }
    if (typeof image.decode === "function") {
      await image.decode().catch(() => undefined);
    }
  }));
  await document.fonts?.ready;
}
