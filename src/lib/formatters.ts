export function money(cents?: number | null, currency = "USD") {
  const value = (cents ?? 0) / 100;
  return new Intl.NumberFormat(undefined, {
    style: "currency",
    currency,
    maximumFractionDigits: 2
  }).format(value);
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
