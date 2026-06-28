import { invoke } from "@tauri-apps/api/core";

export type AppError = {
  code: string;
  message: string;
};

export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeError(error);
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
