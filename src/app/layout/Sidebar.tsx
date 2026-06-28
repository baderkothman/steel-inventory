import { NavLink } from "react-router-dom";
import {
  Box,
  Divider,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Typography
} from "@mui/material";
import DashboardIcon from "@mui/icons-material/Dashboard";
import Inventory2Icon from "@mui/icons-material/Inventory2";
import CategoryIcon from "@mui/icons-material/Category";
import LocalShippingIcon from "@mui/icons-material/LocalShipping";
import PeopleIcon from "@mui/icons-material/People";
import ReceiptLongIcon from "@mui/icons-material/ReceiptLong";
import PointOfSaleIcon from "@mui/icons-material/PointOfSale";
import PaidIcon from "@mui/icons-material/Paid";
import PaymentsIcon from "@mui/icons-material/Payments";
import AssessmentIcon from "@mui/icons-material/Assessment";
import SettingsIcon from "@mui/icons-material/Settings";
import BackupIcon from "@mui/icons-material/Backup";

const items = [
  { to: "/", label: "Dashboard", icon: <DashboardIcon /> },
  { to: "/products", label: "Products", icon: <Inventory2Icon /> },
  { to: "/categories", label: "Categories", icon: <CategoryIcon /> },
  { to: "/suppliers", label: "Suppliers", icon: <LocalShippingIcon /> },
  { to: "/customers", label: "Customers", icon: <PeopleIcon /> },
  { to: "/purchases", label: "Purchases / Stock In", icon: <ReceiptLongIcon /> },
  { to: "/sales", label: "Sales Invoices", icon: <PointOfSaleIcon /> },
  { to: "/expenses", label: "Expenses", icon: <PaidIcon /> },
  { to: "/payments", label: "Payments", icon: <PaymentsIcon /> },
  { to: "/reports", label: "Reports", icon: <AssessmentIcon /> },
  { to: "/settings", label: "Settings", icon: <SettingsIcon /> },
  { to: "/backup", label: "Backup", icon: <BackupIcon /> }
];

export function Sidebar() {
  return (
    <Box
      sx={{
        width: 264,
        height: "100vh",
        bgcolor: "background.paper",
        borderRight: "1px solid",
        borderColor: "divider",
        display: "flex",
        flexDirection: "column"
      }}
    >
      <Box sx={{ px: 2.5, py: 2.5 }}>
        <Typography variant="h6">Steel Inventory</Typography>
        <Typography variant="caption" color="text.secondary">
          Offline desktop system
        </Typography>
      </Box>
      <Divider />
      <List sx={{ px: 1.25, py: 1.5 }}>
        {items.map((item) => (
          <ListItemButton
            key={item.to}
            component={NavLink}
            to={item.to}
            end={item.to === "/"}
            sx={{
              minHeight: 42,
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
            <ListItemIcon sx={{ minWidth: 36, color: "inherit" }}>{item.icon}</ListItemIcon>
            <ListItemText
              primary={item.label}
              primaryTypographyProps={{ fontSize: 14, fontWeight: "inherit" }}
            />
          </ListItemButton>
        ))}
      </List>
    </Box>
  );
}
