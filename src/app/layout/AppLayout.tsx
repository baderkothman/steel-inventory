import { Box } from "@mui/material";
import { QueryErrorResetBoundary } from "@tanstack/react-query";
import { Outlet, useLocation } from "react-router-dom";

import { AppErrorBoundary } from "../../components/feedback/AppErrorBoundary";
import { Sidebar } from "./Sidebar";
import { Topbar } from "./Topbar";

export function AppLayout() {
  const location = useLocation();

  return (
    <Box sx={{ display: "flex", height: "100vh", bgcolor: "background.default" }}>
      <Sidebar />
      <Box sx={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column" }}>
        <Topbar />
        <Box component="main" sx={{ flex: 1, overflow: "auto", p: { xs: 1.5, md: 3 } }}>
          <QueryErrorResetBoundary>
            {({ reset }) => (
              <AppErrorBoundary key={location.pathname} scope="page" onReset={reset}>
                <Outlet />
              </AppErrorBoundary>
            )}
          </QueryErrorResetBoundary>
        </Box>
      </Box>
    </Box>
  );
}
