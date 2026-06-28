import { z } from "zod";

export const positiveMoney = z.number().int().min(1);
export const nonNegativeMoney = z.number().int().min(0);

export const dateString = z.string().regex(/^\d{4}-\d{2}-\d{2}$/);
