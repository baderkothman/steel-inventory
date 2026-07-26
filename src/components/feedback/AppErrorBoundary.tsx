import { Component, type ErrorInfo, type ReactNode } from "react";
import { Alert, Box, Button, Paper, Stack, Typography } from "@mui/material";
import HomeOutlinedIcon from "@mui/icons-material/HomeOutlined";
import RefreshOutlinedIcon from "@mui/icons-material/RefreshOutlined";
import ReplayOutlinedIcon from "@mui/icons-material/ReplayOutlined";

import { normalizeError } from "../../lib/tauri";

type AppErrorBoundaryProps = {
  children: ReactNode;
  scope?: "app" | "page";
  onReset?: () => void;
};

type AppErrorBoundaryState = {
  error: unknown;
};

export class AppErrorBoundary extends Component<
  AppErrorBoundaryProps,
  AppErrorBoundaryState
> {
  state: AppErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: unknown): AppErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Steel Inventory interface error", error, info.componentStack);
  }

  private retry = () => {
    this.props.onReset?.();
    this.setState({ error: null });
  };

  private goToDashboard = () => {
    window.location.hash = "#/";
    window.location.reload();
  };

  render() {
    if (!this.state.error) return this.props.children;

    const failure = normalizeError(this.state.error);
    const isPage = this.props.scope === "page";

    return (
      <Box
        sx={{
          minHeight: isPage ? "100%" : "100vh",
          display: "grid",
          placeItems: "center",
          p: 3,
          bgcolor: "background.default"
        }}
      >
        <Paper variant="outlined" sx={{ width: "min(100%, 680px)", p: { xs: 2.5, md: 3.5 } }}>
          <Stack spacing={2.5}>
            <Box>
              <Typography variant="h5">
                {isPage ? "This page couldn’t open" : "Steel Inventory couldn’t start"}
              </Typography>
              <Typography variant="body2" color="text.secondary" sx={{ mt: 0.75 }}>
                Your local business data has not been changed. Try the page again, or reload the
                application if the problem continues.
              </Typography>
            </Box>

            <Alert severity="error">
              {failure.message || "An unexpected interface error occurred."}
            </Alert>

            <Stack direction={{ xs: "column", sm: "row" }} spacing={1}>
              <Button variant="contained" startIcon={<ReplayOutlinedIcon />} onClick={this.retry}>
                Try again
              </Button>
              {isPage ? (
                <Button startIcon={<HomeOutlinedIcon />} onClick={this.goToDashboard}>
                  Go to dashboard
                </Button>
              ) : null}
              <Button
                startIcon={<RefreshOutlinedIcon />}
                onClick={() => window.location.reload()}
              >
                Reload application
              </Button>
            </Stack>
          </Stack>
        </Paper>
      </Box>
    );
  }
}
