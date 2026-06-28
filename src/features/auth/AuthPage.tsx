import { FormEvent, useState } from "react";
import {
  Alert,
  Box,
  Button,
  CircularProgress,
  Paper,
  Stack,
  TextField,
  Typography
} from "@mui/material";
import LockOutlinedIcon from "@mui/icons-material/LockOutlined";

import { normalizeError } from "../../lib/tauri";
import { useAuth } from "./AuthContext";

export function AuthPage() {
  const { hasAdmin, setup, login } = useAuth();
  const isSetup = hasAdmin === false;
  const [fullName, setFullName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    setError(null);
    try {
      if (isSetup) {
        await setup({ full_name: fullName, email, password });
      } else {
        await login({ email, password });
      }
    } catch (err) {
      setError(normalizeError(err).message);
    } finally {
      setSaving(false);
    }
  }

  return (
    <Box
      sx={{
        minHeight: "100vh",
        display: "grid",
        placeItems: "center",
        bgcolor: "background.default",
        p: 3
      }}
    >
      <Paper
        component="form"
        onSubmit={handleSubmit}
        sx={{ width: 420, maxWidth: "100%", p: 4, border: "1px solid", borderColor: "divider" }}
      >
        <Stack spacing={2.25}>
          <Box sx={{ display: "flex", alignItems: "center", gap: 1.5 }}>
            <Box
              sx={{
                width: 42,
                height: 42,
                borderRadius: 1.5,
                bgcolor: "primary.main",
                color: "primary.contrastText",
                display: "grid",
                placeItems: "center"
              }}
            >
              <LockOutlinedIcon />
            </Box>
            <Box>
              <Typography variant="h5">{isSetup ? "Create admin account" : "Admin login"}</Typography>
              <Typography variant="body2" color="text.secondary">
                Steel Inventory Desktop System
              </Typography>
            </Box>
          </Box>

          {error ? <Alert severity="error">{error}</Alert> : null}

          {isSetup ? (
            <TextField
              label="Full name"
              value={fullName}
              required
              onChange={(event) => setFullName(event.target.value)}
            />
          ) : null}
          <TextField
            label="Email"
            type="email"
            value={email}
            required
            onChange={(event) => setEmail(event.target.value)}
          />
          <TextField
            label="Password or PIN"
            type="password"
            value={password}
            required
            onChange={(event) => setPassword(event.target.value)}
            helperText={isSetup ? "Use at least 4 characters. The credential is stored hashed." : undefined}
          />
          <Button type="submit" variant="contained" disabled={saving} startIcon={saving ? <CircularProgress size={16} /> : undefined}>
            {isSetup ? "Create admin" : "Log in"}
          </Button>
        </Stack>
      </Paper>
    </Box>
  );
}
