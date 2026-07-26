import { useEffect, useState } from "react";
import {
  Alert,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  LinearProgress,
  Snackbar,
  Stack,
  Typography
} from "@mui/material";
import SystemUpdateAltIcon from "@mui/icons-material/SystemUpdateAlt";
import { isTauri } from "@tauri-apps/api/core";
import { resourceDir } from "@tauri-apps/api/path";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

let startupCheckStarted = false;

const READ_ONLY_INSTALL_INSTRUCTIONS =
  "Steel Inventory is running from a read-only disk image. Quit the app, drag Steel Inventory into Applications, eject the disk image, then reopen it from Applications and install the update. Your data will stay in place.";

export function AppUpdater() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [checking, setChecking] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [readOnlyInstall, setReadOnlyInstall] = useState(false);
  const [downloaded, setDownloaded] = useState(0);
  const [downloadSize, setDownloadSize] = useState<number | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (startupCheckStarted || !isTauri()) return;
    startupCheckStarted = true;
    void checkForUpdate(false);
  }, []);

  async function checkForUpdate(showCurrentMessage: boolean) {
    if (!isTauri()) {
      setMessage("Update checks are available in the installed desktop app.");
      return;
    }

    setChecking(true);
    setError(null);
    try {
      const availableUpdate = await check({ timeout: 30_000 });
      if (availableUpdate) {
        setReadOnlyInstall(await isReadOnlyMacInstall());
        setUpdate(availableUpdate);
      } else if (showCurrentMessage) {
        setMessage("Steel Inventory is up to date.");
      }
    } catch (reason) {
      if (showCurrentMessage) {
        setError(toMessage(reason));
      }
    } finally {
      setChecking(false);
    }
  }

  async function installUpdate() {
    if (!update) return;

    setInstalling(true);
    setError(null);
    setDownloaded(0);
    setDownloadSize(null);
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          setDownloadSize(event.data.contentLength ?? null);
        } else if (event.event === "Progress") {
          setDownloaded((current) => current + event.data.chunkLength);
        }
      });
      setMessage(`Version ${update.version} installed. Restarting…`);
      await relaunch();
    } catch (reason) {
      if (isReadOnlyFileSystemError(reason)) {
        setReadOnlyInstall(true);
        setError(null);
      } else {
        setError(toMessage(reason));
      }
      setInstalling(false);
    }
  }

  async function dismissUpdate() {
    if (installing) return;
    const current = update;
    setUpdate(null);
    setReadOnlyInstall(false);
    if (current) {
      await current.close().catch(() => undefined);
    }
  }

  const progress = downloadSize ? Math.min(100, (downloaded / downloadSize) * 100) : undefined;

  return (
    <>
      <Button
        size="small"
        startIcon={<SystemUpdateAltIcon />}
        disabled={checking || installing}
        onClick={() => void checkForUpdate(true)}
      >
        {checking ? "Checking…" : "Updates"}
      </Button>

      <Dialog open={Boolean(update)} onClose={() => void dismissUpdate()} maxWidth="sm" fullWidth>
        <DialogTitle>{readOnlyInstall ? "Move Steel Inventory to Applications" : "Update available"}</DialogTitle>
        <DialogContent>
          <Stack spacing={2}>
            <DialogContentText>
              {readOnlyInstall
                ? `Steel Inventory ${update?.version} is available, but it cannot be installed from the current location.`
                : `Steel Inventory ${update?.version} is ready to install. The app will restart after the update.`}
            </DialogContentText>
            {readOnlyInstall ? <Alert severity="warning">{READ_ONLY_INSTALL_INSTRUCTIONS}</Alert> : null}
            {update?.body ? (
              <Box>
                <Typography variant="subtitle2" gutterBottom>Release notes</Typography>
                <Typography variant="body2" color="text.secondary" sx={{ whiteSpace: "pre-wrap" }}>
                  {update.body}
                </Typography>
              </Box>
            ) : null}
            {installing ? (
              <Box>
                <LinearProgress variant={progress === undefined ? "indeterminate" : "determinate"} value={progress} />
                <Typography variant="caption" color="text.secondary" sx={{ display: "block", mt: 1 }}>
                  {progress === undefined ? "Downloading and installing…" : `Downloading… ${Math.round(progress)}%`}
                </Typography>
              </Box>
            ) : null}
            {error ? <Alert severity="error">{error}</Alert> : null}
          </Stack>
        </DialogContent>
        <DialogActions>
          {readOnlyInstall ? (
            <Button variant="contained" onClick={() => void dismissUpdate()}>Got it</Button>
          ) : (
            <>
              <Button disabled={installing} onClick={() => void dismissUpdate()}>Later</Button>
              <Button
                startIcon={<SystemUpdateAltIcon />}
                variant="contained"
                disabled={installing}
                onClick={() => void installUpdate()}
              >
                {installing ? "Installing…" : "Install and restart"}
              </Button>
            </>
          )}
        </DialogActions>
      </Dialog>

      <Snackbar
        open={Boolean(message)}
        autoHideDuration={5000}
        message={message}
        onClose={() => setMessage(null)}
      />
      <Snackbar open={Boolean(error) && !update} autoHideDuration={7000} onClose={() => setError(null)}>
        <Alert severity="error" onClose={() => setError(null)}>{error}</Alert>
      </Snackbar>
    </>
  );
}

function toMessage(reason: unknown) {
  if (reason instanceof Error) return reason.message;
  if (typeof reason === "string") return reason;
  return "Could not check for updates. Please try again.";
}

async function isReadOnlyMacInstall() {
  try {
    return isReadOnlyMacAppPath(await resourceDir());
  } catch {
    return false;
  }
}

function isReadOnlyMacAppPath(path: string) {
  const normalizedPath = path.replace(/\\/g, "/");
  return normalizedPath.startsWith("/Volumes/") || normalizedPath.includes("/AppTranslocation/");
}

function isReadOnlyFileSystemError(reason: unknown) {
  const message = toMessage(reason).toLowerCase();
  return message.includes("read-only file system") || message.includes("os error 30") || message.includes("erofs");
}
