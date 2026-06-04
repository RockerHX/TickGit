import type {
  AppError,
  CommitListItem,
  ToastItem,
} from "$lib/types";

export function getErrorMessage(error: unknown) {
  if (typeof error === "string") {
    return error;
  }

  if (error && typeof error === "object") {
    const appError = error as Partial<AppError>;
    if (typeof appError.message === "string") {
      return appError.message;
    }

    if ("toString" in error && typeof error.toString === "function") {
      return error.toString();
    }
  }

  return "未知错误";
}

export function buildStepPushHashes(
  commits: CommitListItem[],
  targetHash: string,
) {
  // 历史列表现在是全量口径；分步推送只能从后端标记过的 first-parent 安全路径里取 hash。
  const safeCommits = commits.filter((item) => item.isSafePushTarget);
  const targetIndex = safeCommits.findIndex((item) => item.hash === targetHash);

  if (targetIndex === -1) {
    return null;
  }

  return safeCommits
    .slice(targetIndex)
    .reverse()
    .map((item) => item.hash);
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
