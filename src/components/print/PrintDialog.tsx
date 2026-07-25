import { useId } from "react";
import { Button, Dialog, DialogActions, DialogContent, DialogTitle } from "@mui/material";
import PrintIcon from "@mui/icons-material/Print";

type PrintDialogProps = {
  open: boolean;
  html: string;
  onClose: () => void;
};

export function PrintDialog({ open, html, onClose }: PrintDialogProps) {
  const frameId = useId();

  function handlePrint() {
    const frame = document.getElementById(frameId) as HTMLIFrameElement | null;
    frame?.contentWindow?.print();
  }

  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="lg">
      <DialogTitle>Print preview</DialogTitle>
      <DialogContent sx={{ height: "72vh", p: 0 }}>
        <iframe
          id={frameId}
          title="Print preview"
          srcDoc={html}
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
  );
}
