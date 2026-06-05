import type { Locale } from "$lib/i18n";
import { translate } from "$lib/i18n";

export function formatRelativeDate(
  value: string | number,
  locale: Locale = "en-US",
) {
  const date = typeof value === "number" ? new Date(value) : new Date(value);

  if (Number.isNaN(date.getTime())) {
    return translate(locale, "date.unknownTime");
  }

  const diff = date.getTime() - Date.now();
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  const units = [
    { unit: "day", ms: 24 * 60 * 60 * 1000 },
    { unit: "hour", ms: 60 * 60 * 1000 },
    { unit: "minute", ms: 60 * 1000 },
  ] as const;

  for (const { unit, ms } of units) {
    if (Math.abs(diff) >= ms || unit === "minute") {
      return rtf.format(Math.round(diff / ms), unit);
    }
  }

  return date.toLocaleString(locale);
}

export function formatAbsoluteDate(
  value: string | number,
  locale: Locale = "en-US",
) {
  const date = typeof value === "number" ? new Date(value) : new Date(value);
  return Number.isNaN(date.getTime())
    ? translate(locale, "date.unknownTime")
    : date.toLocaleString(locale, {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "numeric",
        minute: "2-digit",
      });
}

export function getInitials(value: string) {
  const trimmed = value.trim();

  if (!trimmed) {
    return "?";
  }

  const words = trimmed
    .split(/\s+/)
    .map((part) => part[0])
    .filter(Boolean);

  if (words.length >= 2) {
    return `${words[0]}${words[1]}`.toUpperCase();
  }

  return trimmed.slice(0, 2).toUpperCase();
}

export function diffLineClass(line: string) {
  if (line.startsWith("+") && !line.startsWith("+++")) {
    return "bg-emerald-500/10 text-emerald-200";
  }

  if (line.startsWith("-") && !line.startsWith("---")) {
    return "bg-rose-500/10 text-rose-200";
  }

  if (line.startsWith("@@")) {
    return "bg-sky-500/10 text-sky-200";
  }

  return "text-slate-300";
}

export function statusTone(status: string) {
  switch (status[0]) {
    case "A":
      return "border-emerald-400/25 bg-emerald-400/10 text-emerald-200";
    case "M":
      return "border-amber-300/25 bg-amber-300/10 text-amber-100";
    case "D":
      return "border-rose-400/25 bg-rose-400/10 text-rose-200";
    case "R":
      return "border-sky-400/25 bg-sky-400/10 text-sky-200";
    default:
      return "border-slate-400/25 bg-slate-400/10 text-slate-200";
  }
}
