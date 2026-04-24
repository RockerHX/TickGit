export function formatRelativeDate(value: string | number) {
  const date = typeof value === "number" ? new Date(value) : new Date(value);

  if (Number.isNaN(date.getTime())) {
    return "Unknown time";
  }

  const diff = date.getTime() - Date.now();
  const rtf = new Intl.RelativeTimeFormat("zh-CN", { numeric: "auto" });
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

  return date.toLocaleString("zh-CN");
}

export function formatAbsoluteDate(value: string | number) {
  const date = typeof value === "number" ? new Date(value) : new Date(value);
  return Number.isNaN(date.getTime())
    ? "Unknown time"
    : date.toLocaleString("zh-CN");
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
      return "bg-emerald-500/15 text-emerald-300 border-emerald-500/30";
    case "M":
      return "bg-amber-500/15 text-amber-300 border-amber-500/30";
    case "D":
      return "bg-rose-500/15 text-rose-300 border-rose-500/30";
    case "R":
      return "bg-sky-500/15 text-sky-300 border-sky-500/30";
    default:
      return "bg-slate-500/15 text-slate-300 border-slate-500/30";
  }
}
