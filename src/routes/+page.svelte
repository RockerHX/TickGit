<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import BranchSwitcher from "$lib/components/BranchSwitcher.svelte";
  import CommitContextMenu from "$lib/components/CommitContextMenu.svelte";
  import CommitDetailPanel from "$lib/components/CommitDetailPanel.svelte";
  import CommitHistoryList from "$lib/components/CommitHistoryList.svelte";
  import DropOverlay from "$lib/components/DropOverlay.svelte";
  import PushToCommitOverlay from "$lib/components/PushToCommitOverlay.svelte";
  import ResizeHandle from "$lib/components/ResizeHandle.svelte";
  import RepositorySwitcher from "$lib/components/RepositorySwitcher.svelte";
  import StepPushOverlay from "$lib/components/StepPushOverlay.svelte";
  import ToastViewport from "$lib/components/ToastViewport.svelte";
  import { api } from "$lib/tauri/api";
  import {
    listenPushToCommitFailed,
    listenPushToCommitFinished,
    listenStepPushFailed,
    listenStepPushFinished,
    listenStepPushProgress,
  } from "$lib/tauri/events";
  import { fetchCommitDetails } from "$lib/tickgit/page-data";
  import {
    loadBootstrapRepositoryState,
    loadRepositoryIndex,
    loadRepositoryStateSnapshot,
    type RepositoryStateResult,
  } from "$lib/tickgit/repository-actions";
  import {
    buildStepPushHashes,
    createToastItem,
    getErrorMessage,
  } from "$lib/tickgit/page-helpers";
  import {
    canPushCurrentBranch,
    canRefreshBlockedBranchStatus,
    isBranchSwitcherDisabled,
    isContextMenuDisabled,
  } from "$lib/tickgit/page-state";
  import {
    dismissFailedOverlay,
    dismissOverlayIfJobMatches,
    formatPushTargetLabel,
    toFailedPushToCommitState,
    toFailedStepPushState,
    toFinishedPushToCommitState,
    toFinishedStepPushState,
    toRunningPushToCommitState,
    toRunningStepPushState,
  } from "$lib/tickgit/push-events";
  import {
    MAX_LEFT_PANE_WIDTH,
    MIN_BRANCH_PANE_WIDTH,
    MIN_LEFT_PANE_WIDTH,
    RESIZE_DIVIDER_LINE_WIDTH,
  } from "$lib/tickgit/layout";
  import type {
    BranchStatus,
    CommitMeta,
    CommitFileChange,
    CommitListItem,
    PushToCommitUiState,
    RepositorySummary,
    StepPushUiState,
    ToastItem,
  } from "$lib/types";

  const PAGE_SIZE = 50;
  const TOAST_TIMEOUT = 3400;
  const WINDOW_RESIZE_SAVE_DEBOUNCE_MS = 300;
  // 失败态既会自动消失，也允许用户手动关闭，避免错误浮层长时间阻塞界面。
  const PUSH_OVERLAY_DISMISS_MS = 3600;

  let repositories: RepositorySummary[] = [];
  let currentRepository: RepositorySummary | null = null;
  let branchStatus: BranchStatus | null = null;
  let localBranches: string[] = [];

  let commits: CommitListItem[] = [];
  let selectedCommit: CommitListItem | null = null;
  let selectedCommitMeta: CommitMeta | null = null;
  let commitFiles: CommitFileChange[] = [];
  let selectedFilePath: string | null = null;
  let diffText = "";
  let diffViewMode: "unified" | "split" = "unified";
  let hideWhitespaceInDiff = false;

  let nextSkip = 0;
  let hasMore = false;
  let loadingHistory = false;
  let loadingRepository = true;
  let loadingFiles = false;
  let loadingDiff = false;

  let dragActive = false;
  let isPushing = false;
  let switchingBranch = false;
  let activeResizeTarget: "header" | "history" | null = null;
  let leftPaneWidth = 360;

  let toasts: ToastItem[] = [];
  let toastId = 1;
  let pushToCommitState: PushToCommitUiState | null = null;
  let stepPushState: StepPushUiState | null = null;

  let contextMenu = {
    open: false,
    x: 0,
    y: 0,
    commit: null as CommitListItem | null,
  };

  let saveWindowSizeTimer: number | null = null;


  function applyRepositoryState(state: RepositoryStateResult) {
    const { snapshot, branches } = state;
    branchStatus = snapshot.branchStatus;
    localBranches = branches;
    commits = snapshot.commits;
    nextSkip = snapshot.nextSkip;
    hasMore = snapshot.hasMore;
    selectedCommit = snapshot.selectedCommit;
    selectedCommitMeta = snapshot.commitMeta;
    commitFiles = snapshot.commitFiles;
    selectedFilePath = snapshot.selectedFilePath;
    diffText = snapshot.diffText;
  }

  function notifyRemoteRefreshError(state: RepositoryStateResult) {
    if (state.remoteRefreshError) {
      notify(
        "同步远端状态失败",
        getErrorMessage(state.remoteRefreshError),
        "error",
      );
    }
  }

  function notify(
    title: string,
    message: string,
    tone: ToastItem["tone"] = "info",
  ) {
    const id = toastId++;
    toasts = [...toasts, createToastItem(id, title, message, tone)];
    window.setTimeout(() => {
      toasts = toasts.filter((item) => item.id !== id);
    }, TOAST_TIMEOUT);
  }

  async function bootstrap() {
    loadingRepository = true;

    try {
      const bootstrapState = await loadBootstrapRepositoryState(api, {
        pageSize: PAGE_SIZE,
        keepSelection: false,
        previousSelectedHash: null,
        ignoreWhitespace: hideWhitespaceInDiff,
      });
      repositories = bootstrapState.repositories;
      currentRepository = bootstrapState.currentRepository;

      if (bootstrapState.repositoryState) {
        notifyRemoteRefreshError(bootstrapState.repositoryState);
        applyRepositoryState(bootstrapState.repositoryState);
      }
    } catch (error) {
      notify("初始化失败", getErrorMessage(error), "error");
    } finally {
      loadingRepository = false;
    }
  }

  async function refreshRepositories() {
    const repositoryIndex = await loadRepositoryIndex(api);
    repositories = repositoryIndex.repositories;
    currentRepository = repositoryIndex.currentRepository;
  }

  async function switchRepository(path: string) {
    try {
      await api.setCurrentRepository(path);
      await refreshRepositories();

      if (currentRepository) {
        await loadRepositoryState(currentRepository.path);
      }
    } catch (error) {
      notify("切换仓库失败", getErrorMessage(error), "error");
    }
  }

  async function loadRepositoryState(path: string, keepSelection = false) {
    loadingRepository = true;

    try {
      // 这里依赖 loadRepositoryStateSnapshot 预先补齐全部未推送 commits；
      // 否则右键推送到某个 commit / 分步推送时，目标列表可能只拿到第一页。
      const repositoryState = await loadRepositoryStateSnapshot(api, path, {
        pageSize: PAGE_SIZE,
        keepSelection,
        previousSelectedHash: selectedCommit?.hash ?? null,
        ignoreWhitespace: hideWhitespaceInDiff,
      });

      notifyRemoteRefreshError(repositoryState);
      applyRepositoryState(repositoryState);
    } catch (error) {
      notify("读取仓库失败", getErrorMessage(error), "error");
    } finally {
      loadingRepository = false;
    }
  }

  async function switchBranch(branch: string) {
    if (
      !currentRepository ||
      switchingBranch ||
      isPushing ||
      stepPushState?.status === "running"
    ) {
      return;
    }

    if (branch === branchStatus?.branch) {
      return;
    }

    switchingBranch = true;
    loadingRepository = true;

    try {
      await api.checkoutBranch(currentRepository.path, branch);
      await loadRepositoryState(currentRepository.path);
      notify("分支已切换", `当前已切换到 ${branch}`, "success");
    } catch (error) {
      notify("切换分支失败", getErrorMessage(error), "error");
    } finally {
      loadingRepository = false;
      switchingBranch = false;
    }
  }

  async function refreshCurrentRepositoryOnFocus() {
    if (!currentRepository || loadingRepository || loadingHistory) {
      return;
    }

    await loadRepositoryState(currentRepository.path, true);
  }

  async function refreshBlockedBranchStatus() {
    if (!currentRepository || loadingRepository) {
      return;
    }

    await loadRepositoryState(currentRepository.path, true);
  }

  async function loadHistory(append: boolean) {
    if (!currentRepository || loadingHistory) {
      return;
    }

    loadingHistory = true;

    try {
      const page = await api.getCommitHistory(
        currentRepository.path,
        append ? nextSkip : 0,
        PAGE_SIZE,
      );
      commits = append ? [...commits, ...page.items] : page.items;
      nextSkip = page.nextSkip;
      hasMore = page.hasMore;
    } catch (error) {
      notify("加载历史失败", getErrorMessage(error), "error");
    } finally {
      loadingHistory = false;
    }
  }

  async function selectCommit(commit: CommitListItem) {
    selectedCommit = commit;
    await loadCommitFiles(commit.hash);
  }

  async function loadCommitFiles(hash: string) {
    if (!currentRepository) {
      return;
    }

    loadingFiles = true;
    selectedFilePath = null;
    diffText = "";

    try {
      const details = await fetchCommitDetails(
        api,
        currentRepository.path,
        hash,
        hideWhitespaceInDiff,
      );
      selectedCommitMeta = details.commitMeta;
      commitFiles = details.commitFiles;
      selectedFilePath = details.selectedFilePath;
      diffText = details.diffText;
    } catch (error) {
      selectedCommitMeta = null;
      notify("读取 Commit 详情失败", getErrorMessage(error), "error");
    } finally {
      loadingFiles = false;
    }
  }

  async function loadDiff(filePath: string) {
    if (!currentRepository || !selectedCommit) {
      return;
    }

    loadingDiff = true;
    selectedFilePath = filePath;

    try {
      const selectedFile = commitFiles.find((file) => file.path === filePath);
      diffText = await api.getCommitFileDiff(
        currentRepository.path,
        selectedCommit.hash,
        filePath,
        hideWhitespaceInDiff,
        selectedFile?.previousPath ?? null,
      );
    } catch (error) {
      diffText = "";
      notify("读取 Diff 失败", getErrorMessage(error), "error");
    } finally {
      loadingDiff = false;
    }
  }

  async function handleDrop(paths: string[]) {
    dragActive = false;
    if (paths.length === 0) {
      return;
    }

    let added = false;

    for (const path of paths) {
      try {
        await api.addRepository(path);
        await api.setCurrentRepository(path);
        added = true;
      } catch (error) {
        notify("添加仓库失败", getErrorMessage(error), "error");
      }
    }

    if (added) {
      await refreshRepositories();
      if (currentRepository) {
        await loadRepositoryState(currentRepository.path);
      }
      notify("仓库已添加", "新的 Git 仓库已加入 TickGit", "success");
    }
  }

  async function setHideWhitespaceInDiff(value: boolean) {
    if (hideWhitespaceInDiff === value) {
      return;
    }

    hideWhitespaceInDiff = value;

    if (!currentRepository || !selectedCommit || !selectedFilePath) {
      return;
    }

    // whitespace 过滤会改变 Git 返回的 patch 结果，不能只在前端做字符串过滤。
    await loadDiff(selectedFilePath);
  }

  async function pushCurrentBranch() {
    if (!currentRepository || !branchStatus?.pushAvailable || isPushing) {
      return;
    }

    isPushing = true;

    try {
      const started = await api.startPushCurrentBranch(
        currentRepository.path,
        branchStatus.branch,
      );
      pushToCommitState = toRunningPushToCommitState(started);
    } catch (error) {
      isPushing = false;
      notify("推送失败", getErrorMessage(error), "error");
    }
  }

  function openContextMenu(commit: CommitListItem, x: number, y: number) {
    contextMenu = { open: true, x, y, commit };
  }

  function closeContextMenu() {
    contextMenu = { open: false, x: 0, y: 0, commit: null };
  }

  async function pushToTargetCommit() {
    const commit = contextMenu.commit;
    closeContextMenu();

    if (
      !commit ||
      !currentRepository ||
      !branchStatus?.pushAvailable ||
      isPushing
    ) {
      return;
    }

    isPushing = true;

    try {
      const started = await api.startPushToCommit({
        repoPath: currentRepository.path,
        branch: branchStatus.branch,
        hash: commit.hash,
      });
      pushToCommitState = toRunningPushToCommitState(started);
    } catch (error) {
      isPushing = false;
      notify("推送失败", getErrorMessage(error), "error");
    }
  }

  async function startStepPush() {
    const commit = contextMenu.commit;
    closeContextMenu();

    if (
      !commit ||
      !currentRepository ||
      !branchStatus?.pushAvailable ||
      stepPushState?.status === "running"
    ) {
      return;
    }

    const hashes = buildStepPushHashes(commits, commit.hash);

    if (!hashes) {
      notify(
        "无法分步提交",
        commit.pushBlockedReason ?? "目标 Commit 不在可安全分步推送路径上",
        "error",
      );
      return;
    }

    try {
      const started = await api.startStepPush({
        repoPath: currentRepository.path,
        branch: branchStatus.branch,
        hashes,
        delayMs: 1500,
      });

      stepPushState = toRunningStepPushState({
        ...started,
        hash: hashes[0],
      });
    } catch (error) {
      notify("无法开始分步提交", getErrorMessage(error), "error");
    }
  }

  function clampLeftPaneWidth(value: number) {
    return Math.min(Math.max(value, MIN_LEFT_PANE_WIDTH), MAX_LEFT_PANE_WIDTH);
  }

  function startLayoutResize(target: "header" | "history", event: MouseEvent) {
    activeResizeTarget = target;
    applyLayoutResize(event.clientX);
  }

  function applyLayoutResize(pointerX: number) {
    leftPaneWidth = clampLeftPaneWidth(pointerX);
  }

  onMount(() => {
    const disposers: Array<() => void> = [];

    void bootstrap();

    void (async () => {
      const appWindow = getCurrentWindow();

      disposers.push(
        await appWindow.onDragDropEvent(({ payload }) => {
          if (payload.type === "enter" || payload.type === "over") {
            dragActive = true;
          } else if (payload.type === "leave") {
            dragActive = false;
          } else if (payload.type === "drop") {
            void handleDrop(payload.paths);
          }
        }),
      );

      disposers.push(
        await listenPushToCommitFinished((payload) => {
          const target = formatPushTargetLabel(
            payload.target,
            payload.targetKind,
          );
          pushToCommitState = toFinishedPushToCommitState(payload);
          isPushing = false;

          notify("推送成功", `已推送到 ${target.message}`, "success");

          if (currentRepository) {
            void loadRepositoryState(currentRepository.path, true);
          }

          window.setTimeout(() => {
            pushToCommitState = dismissOverlayIfJobMatches(
              pushToCommitState,
              payload.jobId,
            );
          }, 1800);
        }),
      );

      disposers.push(
        await listenPushToCommitFailed((payload) => {
          const target = formatPushTargetLabel(
            payload.target,
            payload.targetKind,
          );
          pushToCommitState = toFailedPushToCommitState(payload);
          isPushing = false;

          notify("推送失败", payload.message, "error");

          if (currentRepository) {
            void loadRepositoryState(currentRepository.path, true);
          }

          window.setTimeout(() => {
            pushToCommitState = dismissOverlayIfJobMatches(
              pushToCommitState,
              payload.jobId,
            );
          }, PUSH_OVERLAY_DISMISS_MS);
        }),
      );

      disposers.push(
        await listenStepPushProgress((payload) => {
          stepPushState = toRunningStepPushState(payload);
        }),
      );

      disposers.push(
        await listenStepPushFinished((payload) => {
          stepPushState = toFinishedStepPushState(payload, stepPushState);

          notify("分步提交完成", "所有目标 Commit 已按顺序推送", "success");

          if (currentRepository) {
            void loadRepositoryState(currentRepository.path, true);
          }

          window.setTimeout(() => {
            stepPushState = dismissOverlayIfJobMatches(
              stepPushState,
              payload.jobId,
            );
          }, 1800);
        }),
      );

      disposers.push(
        await listenStepPushFailed((payload) => {
          stepPushState = toFailedStepPushState(payload);

          notify("分步提交失败", payload.message, "error");

          if (currentRepository) {
            void loadRepositoryState(currentRepository.path, true);
          }

          window.setTimeout(() => {
            stepPushState = dismissOverlayIfJobMatches(
              stepPushState,
              payload.jobId,
            );
          }, PUSH_OVERLAY_DISMISS_MS);
        }),
      );

      disposers.push(
        await appWindow.onFocusChanged(({ payload: focused }) => {
          if (!focused) {
            return;
          }

          void refreshCurrentRepositoryOnFocus();
        }),
      );

      disposers.push(
        await appWindow.onResized(async () => {
          if (saveWindowSizeTimer) {
            window.clearTimeout(saveWindowSizeTimer);
          }

          saveWindowSizeTimer = window.setTimeout(async () => {
            try {
              const size = await appWindow.innerSize();
              await api.saveWindowSize(size.width, size.height);
            } catch {
              // Ignore window size persistence failures to avoid interrupting UI interaction.
            }
          }, WINDOW_RESIZE_SAVE_DEBOUNCE_MS);
        }),
      );
    })();

    const handlePointerMove = (event: MouseEvent) => {
      if (!activeResizeTarget) {
        return;
      }

      applyLayoutResize(event.clientX);
    };

    const stopLayoutResize = () => {
      activeResizeTarget = null;
    };

    window.addEventListener("mousemove", handlePointerMove);
    window.addEventListener("mouseup", stopLayoutResize);

    const closeMenu = () => closeContextMenu();
    window.addEventListener("click", closeMenu);

    return () => {
      for (const dispose of disposers) {
        dispose();
      }
      window.removeEventListener("mousemove", handlePointerMove);
      window.removeEventListener("mouseup", stopLayoutResize);
      window.removeEventListener("click", closeMenu);
      if (saveWindowSizeTimer) {
        window.clearTimeout(saveWindowSizeTimer);
      }
    };
  });
</script>

<svelte:head>
  <title>TickGit</title>
</svelte:head>

<DropOverlay active={dragActive} />
<ToastViewport {toasts} />
<PushToCommitOverlay
  state={pushToCommitState}
  on:close={() => (pushToCommitState = dismissFailedOverlay(pushToCommitState))}
/>
<StepPushOverlay
  state={stepPushState}
  on:close={() => (stepPushState = dismissFailedOverlay(stepPushState))}
/>

<CommitContextMenu
  open={contextMenu.open}
  x={contextMenu.x}
  y={contextMenu.y}
  commit={contextMenu.commit}
  disabled={isContextMenuDisabled({ switchingBranch, isPushing, stepPushState })}
  pushToCommitDisabled={!contextMenu.commit?.isSafePushTarget}
  stepPushDisabled={!contextMenu.commit?.isSafePushTarget}
  pushToCommitReason={contextMenu.commit?.isSafePushTarget
    ? null
    : (contextMenu.commit?.pushBlockedReason ?? null)}
  stepPushReason={contextMenu.commit?.isSafePushTarget
    ? null
    : (contextMenu.commit?.pushBlockedReason ?? null)}
  on:pushToCommit={pushToTargetCommit}
  on:stepPush={startStepPush}
  on:close={closeContextMenu}
/>

<main
  class="flex h-screen min-h-0 flex-col overflow-hidden bg-[#2b3036] text-slate-200"
>
  <header class="shrink-0 border-b border-[#1f2328] bg-[#24292f]">
    <div
      class="grid items-stretch"
      style={`grid-template-columns: minmax(${MIN_LEFT_PANE_WIDTH}px, ${leftPaneWidth}px) ${RESIZE_DIVIDER_LINE_WIDTH}px minmax(${MIN_BRANCH_PANE_WIDTH}px,max-content) auto;`}
    >
      <div class="min-w-0 px-4 py-3">
        <div class="flex items-center gap-3">
          <div class="min-w-0 flex-1">
            <div
              class="mb-1 text-[11px] font-semibold uppercase tracking-[0.22em] text-slate-500"
            >
              Current Repository
            </div>
            <RepositorySwitcher
              {repositories}
              currentPath={currentRepository?.path ?? null}
              on:change={(event) => switchRepository(event.detail.path)}
            />
          </div>
        </div>
      </div>

      <ResizeHandle
        active={activeResizeTarget === "header" ||
          activeResizeTarget === "history"}
        ariaLabel="Resize repository and history panels"
        on:mousedown={(event) => startLayoutResize("header", event.detail)}
      />

      <div class="min-w-0 border-r border-[#1f2328] px-4 py-3">
        <div class="flex items-center gap-3">
          <div class="flex h-9 w-9 items-center justify-center text-slate-300">
            <svg
              viewBox="0 0 16 16"
              class="h-4.5 w-4.5 fill-current"
              aria-hidden="true"
            >
              <path
                d="M5.75 2a1.75 1.75 0 1 0 1.72 2.06l1.6.64a1.75 1.75 0 0 0 2.16 2.16l.64 1.6a1.75 1.75 0 1 0 1.38-.56 1.73 1.73 0 0 0-.31.03l-.64-1.6a1.75 1.75 0 0 0-2.16-2.16l-1.6-.64A1.75 1.75 0 0 0 5.75 2Zm0 1.5a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Zm4.5 2a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Zm3 4a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Z"
              ></path>
            </svg>
          </div>
          <div class="min-w-0 flex-1">
            <div
              class="text-[11px] font-semibold uppercase tracking-[0.22em] text-slate-500"
            >
              Current Branch
            </div>
            <div class="mt-1">
              <BranchSwitcher
                branches={localBranches}
                currentBranch={branchStatus?.branch ?? null}
                disabled={isBranchSwitcherDisabled({
                  currentRepository,
                  loadingRepository,
                  switchingBranch,
                  isPushing,
                  stepPushState,
                })}
                on:change={(event) => switchBranch(event.detail.branch)}
              />
            </div>
            <div class="mt-0.5 truncate text-xs text-slate-400">
              {branchStatus?.upstream ?? "No upstream configured"}
            </div>
          </div>
        </div>
      </div>

      <div class="flex items-center px-4 py-3">
        <button
          class="flex h-[54px] min-w-[188px] items-center gap-3 rounded-sm border border-[#1f2328] bg-[#24292f] px-4 text-left text-[#f0f6fc] transition hover:bg-[#2d333b] disabled:cursor-not-allowed disabled:text-slate-500"
          disabled={!canPushCurrentBranch({
            branchStatus,
            switchingBranch,
            isPushing,
            stepPushState,
          })}
          on:click={pushCurrentBranch}
        >
          <svg
            viewBox="0 0 16 16"
            class="h-5 w-5 shrink-0 fill-current text-[#f0f6fc] disabled:text-slate-500"
            aria-hidden="true"
          >
            <path
              d="M8 14.25a.75.75 0 0 1-.75-.75V5.81L5.03 8.03a.75.75 0 0 1-1.06-1.06l3.5-3.5a.75.75 0 0 1 1.06 0l3.5 3.5a.75.75 0 1 1-1.06 1.06L8.75 5.81v7.69a.75.75 0 0 1-.75.75Z"
            ></path>
          </svg>
          <span class="min-w-0 flex-1">
            <span class="block truncate text-[0.95rem] font-semibold">
              {switchingBranch
                ? "Switching…"
                : isPushing
                  ? "Pushing…"
                  : "Push origin"}
            </span>
            <span class="mt-0.5 block truncate text-xs text-slate-400">
              {branchStatus?.aheadCount
                ? `Ahead ${branchStatus.aheadCount} commits`
                : "Everything up to date"}
            </span>
          </span>
          <span
            class="flex shrink-0 items-center gap-1 rounded-full bg-[#6e7681] px-2.5 py-1 text-[11px] font-semibold text-[#f0f6fc]"
          >
            <span>{branchStatus?.aheadCount ?? 0}</span>
            <svg
              viewBox="0 0 16 16"
              class="h-3 w-3 fill-current"
              aria-hidden="true"
            >
              <path
                d="M8 12.75a.75.75 0 0 1-.75-.75V6.81L5.53 8.53a.75.75 0 1 1-1.06-1.06l3-3a.75.75 0 0 1 1.06 0l3 3a.75.75 0 0 1-1.06 1.06L8.75 6.81V12a.75.75 0 0 1-.75.75Z"
              ></path>
            </svg>
          </span>
        </button>
      </div>
    </div>

    {#if branchStatus && !branchStatus.pushAvailable}
      {#if branchStatus.behindCount > 0}
        <div
          class="border-t border-[#1f2328] bg-[#3b2a1f] px-4 py-3 text-sm text-amber-50"
        >
          <div
            class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between"
          >
            <div class="min-w-0">
              <div class="flex items-center gap-2 font-semibold">
                <span
                  class="flex h-6 w-6 items-center justify-center rounded-full bg-amber-400/15 text-amber-200"
                  aria-hidden="true"
                >
                  !
                </span>
                <span>远端已有更新，暂不能推送</span>
              </div>
              <p class="mt-1 max-w-3xl text-xs leading-5 text-amber-100/85">
                {branchStatus.disabledReason}
                TickGit 只会刷新远端跟踪状态，不会自动拉取、合并或变基代码。
              </p>
            </div>
            <button
              class="h-8 shrink-0 rounded-sm border border-amber-300/35 bg-amber-300/10 px-3 text-xs font-semibold text-amber-50 transition hover:bg-amber-300/18 disabled:cursor-not-allowed disabled:opacity-55"
              disabled={!canRefreshBlockedBranchStatus({
                currentRepository,
                loadingRepository,
                switchingBranch,
                isPushing,
                stepPushState,
              })}
              on:click={refreshBlockedBranchStatus}
            >
              {loadingRepository ? "刷新中…" : "刷新状态"}
            </button>
          </div>

          <div class="mt-3 grid gap-2 md:grid-cols-2">
            <div
              class="rounded-md border border-amber-200/15 bg-[#24292f]/60 p-3"
            >
              <div class="text-xs font-semibold text-amber-100">
                GitHub Desktop
              </div>
              <ol
                class="mt-2 list-decimal space-y-1 pl-4 text-xs text-amber-50/80"
              >
                <li>打开这个仓库</li>
                <li>点击顶部 Fetch origin</li>
                <li>如果出现 Pull origin，继续点击 Pull origin</li>
                <li>如有冲突，按工具提示处理后回 TickGit 刷新</li>
              </ol>
            </div>
            <div
              class="rounded-md border border-amber-200/15 bg-[#24292f]/60 p-3"
            >
              <div class="text-xs font-semibold text-amber-100">SourceTree</div>
              <ol
                class="mt-2 list-decimal space-y-1 pl-4 text-xs text-amber-50/80"
              >
                <li>打开这个仓库</li>
                <li>点击工具栏 Fetch</li>
                <li>点击 Pull 拉取远端更新</li>
                <li>如有冲突，按工具提示处理后回 TickGit 刷新</li>
              </ol>
            </div>
          </div>
        </div>
      {:else}
        <div
          class="border-t border-[#1f2328] bg-[#48322a] px-4 py-2 text-sm text-amber-100"
        >
          {branchStatus.disabledReason}
        </div>
      {/if}
    {/if}
  </header>

  <section
    class="grid min-h-0 flex-1 bg-[#2b3036]"
    style={`grid-template-columns: minmax(${MIN_LEFT_PANE_WIDTH}px, ${leftPaneWidth}px) ${RESIZE_DIVIDER_LINE_WIDTH}px minmax(0,1fr);`}
  >
    <CommitHistoryList
      {commits}
      selectedHash={selectedCommit?.hash ?? null}
      loading={loadingHistory || loadingRepository}
      {hasMore}
      {branchStatus}
      on:select={(event) => selectCommit(event.detail.commit)}
      on:loadMore={() => loadHistory(true)}
      on:openMenu={(event) =>
        openContextMenu(event.detail.commit, event.detail.x, event.detail.y)}
    />

    <ResizeHandle
      active={activeResizeTarget === "header" ||
        activeResizeTarget === "history"}
      ariaLabel="Resize history and details panels"
      on:mousedown={(event) => startLayoutResize("history", event.detail)}
    />

    <CommitDetailPanel
      commit={selectedCommit}
      commitMeta={selectedCommitMeta}
      files={commitFiles}
      {loadingFiles}
      {loadingDiff}
      {selectedFilePath}
      {diffText}
      {diffViewMode}
      {hideWhitespaceInDiff}
      on:selectFile={(event) => loadDiff(event.detail.path)}
      on:diffModeChange={(event) => (diffViewMode = event.detail.mode)}
      on:hideWhitespaceChange={(event) =>
        setHideWhitespaceInDiff(event.detail.value)}
    />
  </section>
</main>
