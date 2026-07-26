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
      fontWeight: 650,
      letterSpacing: 0
    },
    body2: {
      lineHeight: 1.5
    }
  },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          borderRadius: 6,
          minHeight: 36,
          "&:focus-visible": {
            outline: "3px solid rgba(31,111,120,0.24)",
            outlineOffset: 2
          }
        }
      }
    },
    MuiCard: {
      styleOverrides: {
        root: {
          borderRadius: 10,
          boxShadow: "none"
        }
      }
    },
    MuiPaper: {
      styleOverrides: {
        outlined: {
          borderColor: "#d7e0e7"
        }
      }
    },
    MuiTableCell: {
      styleOverrides: {
        root: {
          padding: "12px 16px",
          borderBottomColor: "#e3e9ee",
          fontSize: "0.875rem",
          lineHeight: 1.4,
          "&[align='right']": {
            fontVariantNumeric: "tabular-nums"
          }
        },
        head: {
          fontWeight: 700,
          color: "#33414d",
          backgroundColor: "#f2f6f7",
          whiteSpace: "nowrap",
          zIndex: 2
        }
      }
    },
    MuiTableRow: {
      styleOverrides: {
        root: {
          transition: "background-color 140ms ease-out",
          "&.MuiTableRow-hover:hover": {
            backgroundColor: "#f2f7f7"
          },
          "&.Mui-selected, &.Mui-selected:hover": {
            backgroundColor: "#e5f0f1"
          },
          "&:focus-within": {
            backgroundColor: "#f2f7f7"
          }
        }
      }
    },
    MuiTableSortLabel: {
      styleOverrides: {
        root: {
          "&:focus-visible": {
            outline: "2px solid rgba(31,111,120,0.35)",
            outlineOffset: 2,
            borderRadius: 3
          }
        }
      }
    },
    MuiTablePagination: {
      styleOverrides: {
        toolbar: {
          minHeight: 52
        }
      }
    },
    MuiMenu: {
      defaultProps: {
        elevation: 3
      }
    },
    MuiMenuItem: {
      styleOverrides: {
        root: {
          minHeight: 40,
          fontSize: "0.875rem"
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
    },
    MuiIconButton: {
      styleOverrides: {
        root: {
          "&:focus-visible": {
            outline: "3px solid rgba(31,111,120,0.24)",
            outlineOffset: 2
          }
        }
      }
    }
  }
});
