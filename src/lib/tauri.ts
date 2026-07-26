import { invoke } from "@tauri-apps/api/core";

export type AppError = {
  code: string;
  message: string;
};

export const SESSION_EXPIRED_EVENT = "steel-inventory:session-expired";

export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    const normalized = normalizeError(error);
    if (normalized.code === "UNAUTHORIZED") {
      window.dispatchEvent(new Event(SESSION_EXPIRED_EVENT));
    }
    throw normalized;
  }
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
