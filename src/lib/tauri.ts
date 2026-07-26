import { invoke } from "@tauri-apps/api/core";

export type AppError = {
  code: string;
  message: string;
};

export const SESSION_EXPIRED_EVENT = "steel-inventory:session-expired";

export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, normalizeCommandArgs(args));
  } catch (error) {
    const normalized = normalizeError(error);
    if (normalized.code === "UNAUTHORIZED") {
      window.dispatchEvent(new Event(SESSION_EXPIRED_EVENT));
    }
    throw normalized;
  }
}

/**
 * Tauri exposes Rust command parameters to JavaScript in camelCase. Keep the
 * domain-facing API free to use database-style snake_case while normalizing
 * only the command's top-level argument names here. Nested payload and filter
 * objects are Serde models and must retain their declared field names.
 */
function normalizeCommandArgs(args?: Record<string, unknown>) {
  if (!args) return undefined;

  return Object.fromEntries(
    Object.entries(args).map(([key, value]) => [
      key.replace(/_([a-z])/g, (_, letter: string) => letter.toUpperCase()),
      value
    ])
  );
}

export function normalizeError(error: unknown): AppError {
  if (typeof error === "object" && error !== null && "message" in error) {
    const maybe = error as Partial<AppError>;
    return {
      code: maybe.code ?? "APP_ERROR",
      message: maybe.message ?? "An unexpected error occurred."
    };
  }
  if (typeof error === "string") {
    return { code: "APP_ERROR", message: error };
  }
  return { code: "APP_ERROR", message: "An unexpected error occurred." };
}
