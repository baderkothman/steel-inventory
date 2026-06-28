import { Box, Button, Stack, Typography } from "@mui/material";
import LogoutIcon from "@mui/icons-material/Logout";

import { useAuth } from "../../features/auth/AuthContext";

export function Topbar() {
  const { admin, logout } = useAuth();

  return (
    <Box
      sx={{
        height: 64,
        px: 3,
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        borderBottom: "1px solid",
        borderColor: "divider",
        bgcolor: "background.paper"
      }}
    >
      <Typography variant="body2" color="text.secondary">
        Local database, single admin
      </Typography>
      <Stack direction="row" spacing={1.5} alignItems="center">
        <Typography variant="body2">{admin?.full_name}</Typography>
        <Button size="small" startIcon={<LogoutIcon />} onClick={() => void logout()}>
          Logout
        </Button>
      </Stack>
    </Box>
  );
}
