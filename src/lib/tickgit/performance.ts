const PERF_DEBUG_STORAGE_KEY = "tickgit.perfDebug";

function perfDebugEnabled() {
  if (typeof window === "undefined") {
    return false;
  }

  return window.localStorage.getItem(PERF_DEBUG_STORAGE_KEY) === "1";
}

export function perfNow() {
  return typeof performance === "undefined" ? Date.now() : performance.now();
}

export function logPerfDuration(
  label: string,
  startedAt: number,
  detail?: Record<string, unknown>,
) {
  if (!perfDebugEnabled()) {
    return;
  }

  const durationMs = perfNow() - startedAt;
  console.debug("[tickgit:perf]", label, {
    durationMs: Number(durationMs.toFixed(1)),
    ...detail,
  });
}

export async function measureAsync<T>(
  label: string,
  task: () => Promise<T>,
  detail?: Record<string, unknown>,
) {
  const startedAt = perfNow();

  try {
    return await task();
  } finally {
    logPerfDuration(label, startedAt, detail);
  }
}
