import { useState } from "react";
import { NavLink } from "react-router-dom";
import {
  Box,
  IconButton,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Tooltip,
  Typography
} from "@mui/material";
import DashboardIcon from "@mui/icons-material/Dashboard";
import Inventory2Icon from "@mui/icons-material/Inventory2";
import CategoryIcon from "@mui/icons-material/Category";
import LocalShippingIcon from "@mui/icons-material/LocalShipping";
import PeopleIcon from "@mui/icons-material/People";
import ReceiptLongIcon from "@mui/icons-material/ReceiptLong";
import PointOfSaleIcon from "@mui/icons-material/PointOfSale";
import RequestQuoteOutlinedIcon from "@mui/icons-material/RequestQuoteOutlined";
import PaidIcon from "@mui/icons-material/Paid";
import PaymentsIcon from "@mui/icons-material/Payments";
import AssessmentIcon from "@mui/icons-material/Assessment";
import SettingsIcon from "@mui/icons-material/Settings";
import BackupIcon from "@mui/icons-material/Backup";
import ChevronLeftIcon from "@mui/icons-material/ChevronLeft";
import ChevronRightIcon from "@mui/icons-material/ChevronRight";

const items = [
  { to: "/", label: "Dashboard", icon: <DashboardIcon /> },
  { to: "/products", label: "Products", icon: <Inventory2Icon /> },
  { to: "/categories", label: "Categories", icon: <CategoryIcon /> },
  { to: "/suppliers", label: "Suppliers", icon: <LocalShippingIcon /> },
  { to: "/customers", label: "Customers", icon: <PeopleIcon /> },
  { to: "/purchases", label: "Purchases", icon: <ReceiptLongIcon /> },
  { to: "/sales", label: "Sales Invoices", icon: <PointOfSaleIcon /> },
  { to: "/quotations", label: "Quotations", icon: <RequestQuoteOutlinedIcon /> },
  { to: "/expenses", label: "Expenses", icon: <PaidIcon /> },
  { to: "/payments", label: "Payments", icon: <PaymentsIcon /> },
  { to: "/reports", label: "Reports", icon: <AssessmentIcon /> },
  { to: "/settings", label: "Settings", icon: <SettingsIcon /> },
  { to: "/backup", label: "Backup", icon: <BackupIcon /> }
];

export function Sidebar() {
  const [collapsed, setCollapsed] = useState(() => {
    try {
      return window.localStorage.getItem("steel-inventory.sidebar-collapsed") === "true";
    } catch {
      return false;
    }
  });

  function toggleSidebar() {
    const next = !collapsed;
    setCollapsed(next);
    try {
      window.localStorage.setItem("steel-inventory.sidebar-collapsed", String(next));
    } catch {
      // The preference is optional when storage is unavailable.
    }
  }

  return (
    <Box
      component="aside"
      aria-label="Primary navigation"
      sx={{
        width: collapsed ? 76 : 264,
        flex: `0 0 ${collapsed ? 76 : 264}px`,
        height: "100vh",
        bgcolor: "background.paper",
        borderRight: "1px solid",
        borderColor: "divider",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
        transition: "width 200ms cubic-bezier(0.22, 1, 0.36, 1), flex-basis 200ms cubic-bezier(0.22, 1, 0.36, 1)"
      }}
    >
      <Box
        sx={{
          height: 64,
          minHeight: 64,
          px: collapsed ? 1.25 : 2.5,
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: 1,
          borderBottom: "1px solid",
          borderColor: "divider"
        }}
      >
        {collapsed ? (
          <Typography aria-label="Steel Inventory" variant="h6" color="primary.main" sx={{ width: 40, textAlign: "center" }}>
            SI
          </Typography>
        ) : (
          <Box sx={{ minWidth: 0 }}>
            <Typography variant="h6" noWrap>Steel Inventory</Typography>
            <Typography variant="caption" color="text.secondary" noWrap>
              Offline desktop system
            </Typography>
          </Box>
        )}
        {!collapsed ? (
          <Tooltip title="Collapse sidebar">
            <IconButton size="small" aria-label="Collapse sidebar" onClick={toggleSidebar}>
              <ChevronLeftIcon fontSize="small" />
            </IconButton>
          </Tooltip>
        ) : null}
      </Box>
      <List sx={{ px: collapsed ? 1 : 1.25, py: 1.5, flex: 1, overflowY: "auto", overflowX: "hidden" }}>
        {items.map((item) => (
          <Tooltip key={item.to} title={collapsed ? item.label : ""} placement="right" arrow>
            <ListItemButton
              component={NavLink}
              to={item.to}
              end={item.to === "/"}
              aria-label={item.label}
              sx={{
                minHeight: 44,
                justifyContent: collapsed ? "center" : "flex-start",
                px: collapsed ? 1 : 1.25,
                borderRadius: 1,
                mb: 0.25,
                color: "text.secondary",
                "&.active": {
                  bgcolor: "rgba(31,111,120,0.11)",
                  color: "primary.main",
                  fontWeight: 700
                }
              }}
            >
              <ListItemIcon sx={{ minWidth: collapsed ? 0 : 36, justifyContent: "center", color: "inherit" }}>{item.icon}</ListItemIcon>
              {!collapsed ? (
                <ListItemText
                  primary={item.label}
                  primaryTypographyProps={{ fontSize: 14, fontWeight: "inherit", noWrap: true }}
                />
              ) : null}
            </ListItemButton>
          </Tooltip>
        ))}
      </List>
      {collapsed ? (
        <Box sx={{ p: 1, borderTop: "1px solid", borderColor: "divider" }}>
          <Tooltip title="Expand sidebar" placement="right" arrow>
            <IconButton
              aria-label="Expand sidebar"
              onClick={toggleSidebar}
              sx={{ width: "100%", borderRadius: 1 }}
            >
              <ChevronRightIcon />
            </IconButton>
          </Tooltip>
        </Box>
      ) : null}
    </Box>
  );
}
