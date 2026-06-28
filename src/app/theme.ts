import { createTheme } from "@mui/material/styles";

export const appTheme = createTheme({
  palette: {
    mode: "light",
    background: {
      default: "#f6f8fa",
      paper: "#ffffff"
    },
    primary: {
      main: "#1f6f78",
      contrastText: "#ffffff"
    },
    secondary: {
      main: "#b7832f"
    },
    success: {
      main: "#26734d"
    },
    warning: {
      main: "#b26a00"
    },
    error: {
      main: "#b42318"
    },
    text: {
      primary: "#16202a",
      secondary: "#5b6773"
    },
    divider: "#dbe3ea"
  },
  shape: {
    borderRadius: 8
  },
  typography: {
    fontFamily: [
      "Inter",
      "Segoe UI",
      "Roboto",
      "Arial",
      "sans-serif"
    ].join(","),
    h4: {
      fontSize: "1.55rem",
      fontWeight: 700,
      letterSpacing: 0
    },
    h5: {
      fontSize: "1.25rem",
      fontWeight: 700,
      letterSpacing: 0
    },
    h6: {
      fontSize: "1rem",
      fontWeight: 700,
      letterSpacing: 0
    },
    button: {
      textTransform: "none",
      fontWeight: 700,
      letterSpacing: 0
    }
  },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          borderRadius: 6
        }
      }
    },
    MuiCard: {
      styleOverrides: {
        root: {
          borderRadius: 8,
          boxShadow: "0 1px 2px rgba(16, 24, 40, 0.06)"
        }
      }
    },
    MuiTableCell: {
      styleOverrides: {
        head: {
          fontWeight: 700,
          color: "#2f3b47",
          backgroundColor: "#f2f5f7"
        }
      }
    },
    MuiTextField: {
      defaultProps: {
        size: "small"
      }
    },
    MuiFormControl: {
      defaultProps: {
        size: "small"
      }
    }
  }
});
