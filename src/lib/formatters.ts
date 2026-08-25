// Static options, so this is built once for the process.
const decimalFormatter = new Intl.NumberFormat(undefined, {
  minimumFractionDigits: 2,
  maximumFractionDigits: 2
});

// Currency options vary per call, so formatters are cached by code instead of
// hoisted. A code the runtime rejects — e.g. a 3-decimal currency such as KWD,
// which cannot satisfy maximumFractionDigits: 2 — is cached as null so the
// RangeError is paid once rather than on every format.
const currencyFormatters = new Map<string, Intl.NumberFormat | null>();

function currencyFormatter(code: string) {
  const cached = currencyFormatters.get(code);
  if (cached !== undefined) return cached;

  let formatter: Intl.NumberFormat | null;
  try {
    formatter = new Intl.NumberFormat(undefined, {
      style: "currency",
      currency: code,
      maximumFractionDigits: 2
    });
  } catch {
    formatter = null;
  }
  currencyFormatters.set(code, formatter);
  return formatter;
}

export function money(cents?: number | null, currency = "USD") {
  const value = (cents ?? 0) / 100;
  const code = currency.trim().toUpperCase();

  const formatter = currencyFormatter(code || "USD");
  if (formatter) return formatter.format(value);

  const formatted = decimalFormatter.format(value);
  return code ? `${formatted} ${code}` : formatted;
}

export function isCurrencyCode(value: string) {
  return /^[A-Za-z]{3}$/.test(value.trim());
}

export function toCents(value: string | number) {
  const number = typeof value === "number" ? value : Number(value || 0);
  return Math.round(number * 100);
}

export function fromCents(value?: number | null) {
  return ((value ?? 0) / 100).toFixed(2);
}

export function quantity(value?: number | null) {
  return Number(value ?? 0).toLocaleString(undefined, {
    maximumFractionDigits: 3
  });
}

export function today() {
  return new Date().toISOString().slice(0, 10);
}
