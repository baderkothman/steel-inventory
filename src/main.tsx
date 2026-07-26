import React from "react";
import ReactDOM from "react-dom/client";
import { CssBaseline, ThemeProvider } from "@mui/material";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { HashRouter } from "react-router-dom";

import App from "./App";
import { AppErrorBoundary } from "./components/feedback/AppErrorBoundary";
import { AuthProvider } from "./features/auth/AuthContext";
import { appTheme } from "./app/theme";
import "./styles.css";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 20_000,
      refetchOnWindowFocus: false,
      retry: 1,
      throwOnError: true
    }
  }
});

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ThemeProvider theme={appTheme}>
      <CssBaseline />
      <QueryClientProvider client={queryClient}>
        <HashRouter>
          <AppErrorBoundary scope="app">
            <AuthProvider>
              <App />
            </AuthProvider>
          </AppErrorBoundary>
        </HashRouter>
      </QueryClientProvider>
    </ThemeProvider>
  </React.StrictMode>
);
