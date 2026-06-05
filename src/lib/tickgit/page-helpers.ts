import {
  FALLBACK_LOCALE,
  translate,
  translateErrorCode,
  type Locale,
} from "$lib/i18n";
import type { AppError, CommitListItem, ToastItem } from "$lib/types";

export function getErrorMessage(
  error: unknown,
  locale: Locale = FALLBACK_LOCALE,
) {
  if (typeof error === "string") {
    return error;
  }

  if (error && typeof error === "object") {
    const appError = error as Partial<AppError>;
    if (typeof appError.code === "string") {
      return translateErrorCode(locale, appError.code, appError.message);
    }

    if (typeof appError.message === "string") {
      return appError.message;
    }

    if ("toString" in error && typeof error.toString === "function") {
      return error.toString();
    }
  }

  return translate(locale, "common.unknownError");
}

export function pickSelectedCommit(
  commits: CommitListItem[],
  previousSelectedHash: string | null,
  keepSelection: boolean,
) {
  if (!keepSelection || !previousSelectedHash) {
    return commits[0] ?? null;
  }

  return (
    commits.find((item) => item.hash === previousSelectedHash) ??
    commits[0] ??
    null
  );
}

export function createToastItem(
  id: number,
  title: string,
  message: string,
  tone: ToastItem["tone"] = "info",
): ToastItem {
  return { id, title, message, tone };
}
