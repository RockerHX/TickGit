<script lang="ts">
  import { onMount, tick } from "svelte";
  import {
    locale,
    translate,
    translateBranchDisabledReason,
    translateErrorCode,
  } from "$lib/i18n";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import CommitContextMenu from "$lib/components/CommitContextMenu.svelte";
  import ConfirmRepositoryRemoveDialog from "$lib/components/ConfirmRepositoryRemoveDialog.svelte";
  import CommitDetailPanel from "$lib/components/CommitDetailPanel.svelte";
  import CommitHistoryList from "$lib/components/CommitHistoryList.svelte";
  import DropOverlay from "$lib/components/DropOverlay.svelte";
  import PushToCommitOverlay from "$lib/components/PushToCommitOverlay.svelte";
  import ResizeHandle from "$lib/components/ResizeHandle.svelte";
  import SettingsDialog from "$lib/components/SettingsDialog.svelte";
  import StepPushPlanDialog from "$lib/components/StepPushPlanDialog.svelte";
  import StepPushOverlay from "$lib/components/StepPushOverlay.svelte";
  import ToastViewport from "$lib/components/ToastViewport.svelte";
  import TopToolbar from "$lib/components/TopToolbar.svelte";
  import { api } from "$lib/tauri/api";
  import {
    listenPushToCommitFailed,
    listenPushToCommitFinished,
    listenPushToCommitProgress,
    listenStepPushFailed,
    listenStepPushFinished,
    listenStepPushProgress,
  } from "$lib/tauri/events";
  import {
    EMPTY_DIFF_RESULT,
    fetchCommitDetails,
    fetchCommitFileDiff,
    invalidateRepositoryCache,
    type CachedCommitDetails,
  } from "$lib/tickgit/page-data";
  import {
    loadBootstrapRepositoryState,
    loadRepositoryIndex,
    loadRepositoryStateSnapshot,
    type RepositoryStateResult,
  } from "$lib/tickgit/repository-actions";
  import {
    EMPTY_HISTORY_FILTERS,
    getActiveHistoryFilterCount,
    historyFiltersEqual,
    normalizeHistoryFilters,
  } from "$lib/tickgit/history";
  import {
    createToastItem,
    getBranchSwitchErrorMessage,
    getErrorMessage,
  } from "$lib/tickgit/page-helpers";
  import {
    canCheckRepositoryRevisionOnFocus,
    canLoadCommitFiles,
    canLoadDiff,
    canLoadHistory,
    canPushCurrentBranch,
    canRefreshBlockedBranchStatus,
    canRefreshCurrentRepositoryOnFocus,
    canStartStepPush,
    canStartTargetCommitPush,
    canSwitchBranch,
    isBranchSwitcherDisabled,
    isContextMenuDisabled,
    isRepositoryAvailable,
    shouldClearRepositoryData,
    shouldRefreshRepositoryForRevision,
    shouldShowRepositoryUnavailableState,
  } from "$lib/tickgit/page-state";
  import {
    canManageRepositories,
    repositoryStatusMessage,
  } from "$lib/tickgit/repositories";
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
  import { startStepPushFromPlan } from "$lib/tickgit/step-push-plan";
  import {
    HISTORY_PAGE_SIZE,
    getPaginationState,
  } from "$lib/tickgit/pagination";
  import { measureAsync } from "$lib/tickgit/performance";
  import { textSelectionEnabled } from "$lib/tickgit/preferences";
  import { writeClipboardText } from "$lib/tickgit/clipboard";
  import {
    MAX_LEFT_PANE_WIDTH,
    MIN_LEFT_PANE_WIDTH,
    RESIZE_DIVIDER_LINE_WIDTH,
  } from "$lib/tickgit/layout";
  import type {
    BranchStatus,
    CommitHistoryFilters,
    CommitMeta,
    CommitFileChange,
    CommitFileDiffResult,
    CommitListItem,
    PushToCommitUiState,
    RepositoryRevision,
    RepositorySummary,
    StepPushPlan,
    StepPushUiState,
    ToastItem,
  } from "$lib/types";

  const PAGE_SIZE = HISTORY_PAGE_SIZE;
  const TOAST_TIMEOUT = 3400;
  const WINDOW_RESIZE_SAVE_DEBOUNCE_MS = 300;
  const HISTORY_FILTER_DEBOUNCE_MS = 300;
  const FOCUS_REVISION_THROTTLE_MS = 60_000;
  // 失败态既会自动消失，也允许用户手动关闭，避免错误浮层长时间阻塞界面。
  const PUSH_OVERLAY_DISMISS_MS = 3600;

  let repositories: RepositorySummary[] = [];
  let currentRepository: RepositorySummary | null = null;
  let currentRepositoryRevision: RepositoryRevision | null = null;
  let lastFocusRevisionCheckedAt = 0;
  let branchStatus: BranchStatus | null = null;
  let localBranches: string[] = [];

  let commits: CommitListItem[] = [];
  let historyFilters: CommitHistoryFilters = { ...EMPTY_HISTORY_FILTERS };
  $: activeHistoryFilterCount = getActiveHistoryFilterCount(historyFilters);
  let selectedCommit: CommitListItem | null = null;
  let selectedCommitMeta: CommitMeta | null = null;
  let commitFiles: CommitFileChange[] = [];
  let selectedFilePath: string | null = null;
  let diffResult: CommitFileDiffResult = EMPTY_DIFF_RESULT;
  let diffViewMode: "unified" | "split" = "unified";
  let hideWhitespaceInDiff = false;

  let nextSkip = 0;
  let hasMore = false;
  let historyTotalCount = 0;
  let historyPageIndex = 0;
  let loadingHistory = false;
  let loadingRepository = true;
  let loadingFiles = false;
  let loadingDiff = false;
  let syncingRemoteStatus = false;

  let dragActive = false;
  let isPushing = false;
  let switchingBranch = false;
  let managingRepositoryPath: string | null = null;
  let repositoryPendingRemoval: RepositorySummary | null = null;
  let activeResizeTarget: "history" | null = null;
  let leftPaneWidth = 360;
  let settingsOpen = false;

  let toasts: ToastItem[] = [];
  let toastId = 1;
  let pushToCommitState: PushToCommitUiState | null = null;
  let stepPushState: StepPushUiState | null = null;
  let stepPushPlanOpen = false;
  let loadingStepPushPlan = false;
  let submittingStepPushPlan = false;
  let stepPushPlan: StepPushPlan | null = null;
  let stepPushPlanErrorMessage: string | null = null;
  let stepPushPlanRepoPath: string | null = null;
  let stepPushPlanRequestId = 0;

  let contextMenu = {
    open: false,
    x: 0,
    y: 0,
    commit: null as CommitListItem | null,
  };

  let saveWindowSizeTimer: number | null = null;
  let historyFilterTimer: number | null = null;
  let historyRequestId = 0;

  $: canRefreshRemoteStatus =
    canRefreshBlockedBranchStatus({
      currentRepository,
      loadingRepository,
      switchingBranch,
      isPushing,
      stepPushState,
    }) && !syncingRemoteStatus;
  $: canRunRepositoryManagementNow =
    !managingRepositoryPath &&
    canManageRepositories({
      loadingRepository,
      switchingBranch,
      isPushing,
      stepPushState,
    });
  $: canPushBranch = canPushCurrentBranch({
    branchStatus,
    switchingBranch,
    isPushing: isPushing || syncingRemoteStatus,
    stepPushState,
  });
  $: pushCardAheadCount = branchStatus?.aheadCount ?? 0;
  $: pushCardTitle = switchingBranch
    ? translate($locale, "push.switching")
    : isPushing
      ? translate($locale, "push.pushing")
      : translate($locale, "push.button");
  $: pushCardSubtitle = isPushing
    ? translate($locale, "push.uploading")
    : pushCardAheadCount > 0
      ? translate($locale, "push.aheadCommits", {
          count: pushCardAheadCount,
        })
      : translate($locale, "push.upToDate");
  $: pushCardBlockedReason = canPushBranch
    ? null
    : pushCardBlockedReasonMessage();
  $: pushCardAriaLabel = [
    pushCardTitle,
    pushCardSubtitle,
    pushCardBlockedReason,
  ]
    .filter(Boolean)
    .join(". ");

  function getCachedCommitDetails(
    preferredFilePathFilter: string | null = historyFilters.filePath ?? null,
  ): CachedCommitDetails | null {
    if (!selectedCommit || !selectedCommitMeta) {
      return null;
    }

    return {
      hash: selectedCommit.hash,
      ignoreWhitespace: hideWhitespaceInDiff,
      preferredFilePathFilter,
      commitMeta: selectedCommitMeta,
      commitFiles,
      selectedFilePath,
      diffResult,
    };
  }

  function resetRepositoryRevisionState() {
    currentRepositoryRevision = null;
    lastFocusRevisionCheckedAt = 0;
  }

  async function rememberCurrentRepositoryRevision(path: string) {
    try {
      const revision = await api.getRepositoryRevision(path);
      if (currentRepository?.path === path) {
        currentRepositoryRevision = revision;
      }
    } catch (error) {
      console.error("failed to read repository revision", error);
    }
  }

  function applyRepositoryState(state: RepositoryStateResult) {
    const { snapshot, branches } = state;
    branchStatus = snapshot.branchStatus;
    localBranches = branches;
    commits = snapshot.commits;
    nextSkip = snapshot.nextSkip;
    hasMore = snapshot.hasMore;
    historyTotalCount = snapshot.totalCount;
    selectedCommit = snapshot.selectedCommit;
    selectedCommitMeta = snapshot.commitMeta;
    commitFiles = snapshot.commitFiles;
    selectedFilePath = snapshot.selectedFilePath;
    diffResult = snapshot.diffResult;
  }

  function clearRepositoryData() {
    branchStatus = null;
    localBranches = [];
    commits = [];
    selectedCommit = null;
    selectedCommitMeta = null;
    commitFiles = [];
    selectedFilePath = null;
    diffResult = EMPTY_DIFF_RESULT;
    nextSkip = 0;
    hasMore = false;
    historyTotalCount = 0;
    historyPageIndex = 0;
    loadingHistory = false;
    loadingFiles = false;
    loadingDiff = false;
    isPushing = false;
    switchingBranch = false;
    pushToCommitState = null;
    stepPushState = null;
    contextMenu = { open: false, x: 0, y: 0, commit: null };
    repositoryPendingRemoval = null;
    resetRepositoryRevisionState();
  }

  function clearHistoryDetailState() {
    selectedCommit = null;
    selectedCommitMeta = null;
    commitFiles = [];
    selectedFilePath = null;
    diffResult = EMPTY_DIFF_RESULT;
    loadingFiles = false;
    loadingDiff = false;
    contextMenu = { open: false, x: 0, y: 0, commit: null };
  }

  function canRunRepositoryManagement() {
    return canRunRepositoryManagementNow;
  }

  function currentBranchDisabledReason() {
    return translateBranchDisabledReason(
      $locale,
      branchStatus?.disabledReasonCode,
      branchStatus?.disabledReason,
    );
  }

  function pushCardBlockedReasonMessage() {
    if (isPushing) {
      return translate($locale, "push.currentBusy");
    }

    if (syncingRemoteStatus) {
      return translate($locale, "common.refreshing");
    }

    if (switchingBranch) {
      return translate($locale, "push.switching");
    }

    if (stepPushState?.status === "running") {
      return translate($locale, "push.stepBusy");
    }

    if (!branchStatus) {
      return translate($locale, "branch.pushDisabledFallback");
    }

    if (!branchStatus.pushAvailable) {
      return currentBranchDisabledReason();
    }

    if (branchStatus.aheadCount <= 0) {
      return translate($locale, "push.upToDate");
    }

    return translate($locale, "branch.currentPushUnavailable");
  }

  function currentCommitPushBlockedReason() {
    const commit = contextMenu.commit;

    if (!commit) {
      return null;
    }

    return translateErrorCode(
      $locale,
      commit.pushBlockedReasonCode,
      commit.pushBlockedReason,
    );
  }

  async function loadCurrentRepositoryState(keepSelection = false) {
    if (shouldClearRepositoryData(currentRepository)) {
      clearRepositoryData();
      return;
    }

    if (!currentRepository) {
      return;
    }

    await loadRepositoryState(currentRepository.path, keepSelection);
  }

  function notifyRemoteRefreshError(state: RepositoryStateResult) {
    if (state.remoteRefreshError) {
      notify(
        translate($locale, "repository.remoteSyncFailedTitle"),
        getErrorMessage(state.remoteRefreshError, $locale),
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

  async function copyRepositoryName(repository: RepositorySummary) {
    try {
      await writeClipboardText(repository.name);
      notify(
        translate($locale, "repository.contextCopiedNameTitle"),
        repository.name,
        "success",
      );
    } catch (error) {
      notify(
        translate($locale, "repository.contextActionFailedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    }
  }

  async function copyRepositoryPath(repository: RepositorySummary) {
    try {
      await writeClipboardText(repository.path);
      notify(
        translate($locale, "repository.contextCopiedPathTitle"),
        repository.path,
        "success",
      );
    } catch (error) {
      notify(
        translate($locale, "repository.contextActionFailedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    }
  }

  async function viewRepositoryOnGithub(url: string) {
    try {
      await openUrl(url);
      notify(
        translate($locale, "repository.contextOpenedTitle"),
        translate($locale, "repository.contextOpenedGithub"),
        "success",
      );
    } catch (error) {
      notify(
        translate($locale, "repository.contextActionFailedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    }
  }

  async function runRepositoryAction(
    repository: RepositorySummary,
    action: (repoPath: string) => Promise<void>,
    successMessageKey:
      | "repository.contextOpenedTerminal"
      | "repository.contextRevealedFinder"
      | "repository.contextOpenedVSCode",
  ) {
    try {
      await action(repository.path);
      notify(
        translate($locale, "repository.contextOpenedTitle"),
        translate($locale, successMessageKey),
        "success",
      );
    } catch (error) {
      notify(
        translate($locale, "repository.contextActionFailedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    }
  }

  function setHistoryFilters(filters: CommitHistoryFilters) {
    const nextFilters = normalizeHistoryFilters(filters);

    if (historyFiltersEqual(historyFilters, nextFilters)) {
      return;
    }

    historyFilters = nextFilters;
    historyPageIndex = 0;
    scheduleHistoryFilterReload();
  }

  function clearHistoryFilters() {
    if (activeHistoryFilterCount === 0) {
      return;
    }

    historyFilters = { ...EMPTY_HISTORY_FILTERS };
    historyPageIndex = 0;
    scheduleHistoryFilterReload();
  }

  function scheduleHistoryFilterReload() {
    if (historyFilterTimer) {
      window.clearTimeout(historyFilterTimer);
    }

    historyRequestId += 1;
    loadingHistory = false;
    commits = [];
    nextSkip = 0;
    hasMore = false;
    historyTotalCount = 0;
    clearHistoryDetailState();

    historyFilterTimer = window.setTimeout(() => {
      historyFilterTimer = null;
      void loadHistory();
    }, HISTORY_FILTER_DEBOUNCE_MS);
  }

  async function bootstrap() {
    loadingRepository = true;

    try {
      const bootstrapState = await measureAsync(
        "page.bootstrap",
        () =>
          loadBootstrapRepositoryState(api, {
            pageSize: PAGE_SIZE,
            historySkip: historyPageIndex * PAGE_SIZE,
            keepSelection: false,
            previousSelectedHash: null,
            ignoreWhitespace: hideWhitespaceInDiff,
            filters: historyFilters,
            preferredFilePathFilter: historyFilters.filePath,
          }),
        { pageSize: PAGE_SIZE },
      );
      repositories = bootstrapState.repositories;
      currentRepository = bootstrapState.currentRepository;

      if (bootstrapState.repositoryState) {
        applyRepositoryState(bootstrapState.repositoryState);
        if (currentRepository) {
          void rememberCurrentRepositoryRevision(currentRepository.path);
        }
      } else if (shouldClearRepositoryData(currentRepository)) {
        clearRepositoryData();
      }
    } catch (error) {
      notify(
        translate($locale, "app.initFailedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    } finally {
      loadingRepository = false;
    }
  }

  async function refreshRepositories() {
    const repositoryIndex = await loadRepositoryIndex(api);
    const previousPath = currentRepository?.path ?? null;
    repositories = repositoryIndex.repositories;
    currentRepository = repositoryIndex.currentRepository;

    if ((currentRepository?.path ?? null) !== previousPath) {
      resetRepositoryRevisionState();
    }
  }

  async function switchRepository(path: string) {
    try {
      await api.setCurrentRepository(path);
      await refreshRepositories();
      historyPageIndex = 0;

      await loadCurrentRepositoryState();
    } catch (error) {
      notify(
        translate($locale, "repository.switchFailedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    }
  }

  async function loadRepositoryState(
    path: string,
    keepSelection = false,
    refreshRemoteTracking = false,
  ) {
    loadingRepository = true;

    try {
      // 这里依赖 loadRepositoryStateSnapshot 预先补齐全部未推送 commits；
      // 否则右键推送到某个 commit / 分步推送时，目标列表可能只拿到第一页。
      const repositoryState = await measureAsync(
        "page.loadRepositoryState",
        () =>
          loadRepositoryStateSnapshot(api, path, {
            pageSize: PAGE_SIZE,
            historySkip: historyPageIndex * PAGE_SIZE,
            keepSelection,
            previousSelectedHash: selectedCommit?.hash ?? null,
            ignoreWhitespace: hideWhitespaceInDiff,
            refreshRemoteTracking,
            filters: historyFilters,
            preferredFilePathFilter: historyFilters.filePath,
            cachedCommitDetails: getCachedCommitDetails(
              historyFilters.filePath,
            ),
          }),
        {
          keepSelection,
          refreshRemoteTracking,
          skip: historyPageIndex * PAGE_SIZE,
        },
      );

      notifyRemoteRefreshError(repositoryState);
      applyRepositoryState(repositoryState);
      void rememberCurrentRepositoryRevision(path);
    } catch (error) {
      notify(
        translate($locale, "repository.readFailedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    } finally {
      loadingRepository = false;
    }
  }

  async function switchBranch(branch: string) {
    const repository = currentRepository;

    if (
      !repository ||
      !canSwitchBranch(
        {
          currentRepository: repository,
          loadingRepository,
          switchingBranch,
          isPushing,
          stepPushState,
          branchStatus,
        },
        branch,
      )
    ) {
      return;
    }

    switchingBranch = true;
    loadingRepository = true;
    historyPageIndex = 0;

    try {
      await api.checkoutBranch(repository.path, branch);
      invalidateRepositoryCache(repository.path);
      await loadRepositoryState(repository.path);
      notify(
        translate($locale, "branch.switchedTitle"),
        translate($locale, "branch.switchedMessage", { branch }),
        "success",
      );
    } catch (error) {
      notify(
        translate($locale, "branch.switchFailedTitle"),
        getBranchSwitchErrorMessage(error, branch, $locale),
        "error",
      );
    } finally {
      loadingRepository = false;
      switchingBranch = false;
    }
  }

  async function refreshCurrentRepositoryOnFocus() {
    const repository = currentRepository;
    const now = Date.now();

    if (
      !repository ||
      !canCheckRepositoryRevisionOnFocus({
        currentRepository: repository,
        loadingRepository,
        loadingHistory,
        lastCheckedAt: lastFocusRevisionCheckedAt,
        now,
        throttleMs: FOCUS_REVISION_THROTTLE_MS,
      })
    ) {
      return;
    }

    lastFocusRevisionCheckedAt = now;

    try {
      const nextRevision = await api.getRepositoryRevision(repository.path);

      if (
        !shouldRefreshRepositoryForRevision(
          currentRepositoryRevision,
          nextRevision,
        )
      ) {
        currentRepositoryRevision = nextRevision;
        return;
      }

      currentRepositoryRevision = nextRevision;
      invalidateRepositoryCache(repository.path);
      await loadCurrentRepositoryState(true);
    } catch (error) {
      console.error("failed to refresh repository revision", error);
    }
  }

  async function fetchRemoteStatusManually() {
    const repository = currentRepository;

    if (
      !repository ||
      !canRefreshBlockedBranchStatus({
        currentRepository: repository,
        loadingRepository,
        switchingBranch,
        isPushing,
        stepPushState,
      }) ||
      syncingRemoteStatus
    ) {
      return;
    }

    syncingRemoteStatus = true;
    invalidateRepositoryCache(repository.path);

    try {
      const repositoryState = await loadRepositoryStateSnapshot(
        api,
        repository.path,
        {
          pageSize: PAGE_SIZE,
          historySkip: historyPageIndex * PAGE_SIZE,
          keepSelection: true,
          previousSelectedHash: selectedCommit?.hash ?? null,
          ignoreWhitespace: hideWhitespaceInDiff,
          refreshRemoteTracking: true,
          filters: historyFilters,
          preferredFilePathFilter: historyFilters.filePath,
          cachedCommitDetails: getCachedCommitDetails(historyFilters.filePath),
        },
      );

      notifyRemoteRefreshError(repositoryState);
      applyRepositoryState(repositoryState);
      void rememberCurrentRepositoryRevision(repository.path);
    } catch (error) {
      notify(
        translate($locale, "repository.readFailedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    } finally {
      syncingRemoteStatus = false;
    }
  }

  async function loadHistory(pageIndex = historyPageIndex) {
    const repository = currentRepository;
    const requestId = ++historyRequestId;
    const filters = normalizeHistoryFilters(historyFilters);
    const targetPageIndex = getPaginationState(
      historyTotalCount,
      pageIndex,
      PAGE_SIZE,
    ).pageIndex;

    if (
      !repository ||
      !canLoadHistory({ currentRepository: repository, loadingHistory })
    ) {
      return;
    }

    loadingHistory = true;

    try {
      const page = await api.getCommitHistory(
        repository.path,
        targetPageIndex * PAGE_SIZE,
        PAGE_SIZE,
        filters,
      );

      if (requestId !== historyRequestId) {
        return;
      }

      historyPageIndex = targetPageIndex;
      commits = page.items;
      nextSkip = page.nextSkip;
      hasMore = page.hasMore;
      historyTotalCount = page.totalCount;

      const selected = page.items[0] ?? null;
      if (selected) {
        await selectCommit(selected, filters.filePath);
      } else {
        clearHistoryDetailState();
      }
    } catch (error) {
      if (requestId === historyRequestId) {
        notify(
          translate($locale, "history.loadFailedTitle"),
          getErrorMessage(error, $locale),
          "error",
        );
      }
    } finally {
      if (requestId === historyRequestId) {
        loadingHistory = false;
      }
    }
  }

  function waitForAnimationFrame() {
    return new Promise<void>((resolve) => {
      if (typeof requestAnimationFrame === "function") {
        requestAnimationFrame(() => resolve());
        return;
      }

      setTimeout(resolve, 0);
    });
  }

  async function paintLoadingState() {
    await tick();
    await waitForAnimationFrame();
  }

  function changeHistoryPage(pageIndex: number) {
    if (loadingHistory || loadingRepository) {
      return;
    }

    const targetPageIndex = getPaginationState(
      historyTotalCount,
      pageIndex,
      PAGE_SIZE,
    ).pageIndex;

    if (targetPageIndex === historyPageIndex) {
      return;
    }

    historyPageIndex = targetPageIndex;
    void loadHistory(targetPageIndex);
  }

  async function selectCommit(
    commit: CommitListItem,
    preferredFilePathFilter: string | null = historyFilters.filePath ?? null,
  ) {
    selectedCommit = commit;
    await loadCommitFiles(
      commit.hash,
      preferredFilePathFilter,
      commit.parents[0] ?? null,
    );
  }

  async function loadCommitFiles(
    hash: string,
    preferredFilePathFilter: string | null = historyFilters.filePath ?? null,
    baseHash: string | null = selectedCommit?.parents[0] ?? null,
  ) {
    const repository = currentRepository;

    if (!repository || !canLoadCommitFiles({ currentRepository: repository })) {
      return;
    }

    loadingFiles = true;
    selectedFilePath = null;
    diffResult = EMPTY_DIFF_RESULT;
    await paintLoadingState();

    try {
      const details = await measureAsync(
        "page.loadCommitFiles",
        () =>
          fetchCommitDetails(
            api,
            repository.path,
            hash,
            hideWhitespaceInDiff,
            preferredFilePathFilter,
            baseHash,
          ),
        { hash, preferredFilePathFilter },
      );
      selectedCommitMeta = details.commitMeta;
      commitFiles = details.commitFiles;
      selectedFilePath = details.selectedFilePath;
      diffResult = details.diffResult;
    } catch (error) {
      selectedCommitMeta = null;
      notify(
        translate($locale, "commit.detailsFailedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    } finally {
      loadingFiles = false;
    }
  }

  async function loadDiff(filePath: string) {
    const repository = currentRepository;
    const commit = selectedCommit;

    if (
      !repository ||
      !commit ||
      !canLoadDiff({ currentRepository: repository, selectedCommit: commit })
    ) {
      return;
    }

    loadingDiff = true;
    selectedFilePath = filePath;
    await paintLoadingState();

    try {
      const selectedFile = commitFiles.find((file) => file.path === filePath);
      diffResult = await measureAsync(
        "page.loadDiff",
        () =>
          fetchCommitFileDiff(
            api,
            repository.path,
            commit.hash,
            filePath,
            hideWhitespaceInDiff,
            selectedFile?.previousPath ?? null,
            commit.parents[0] ?? null,
          ),
        { filePath, hideWhitespaceInDiff },
      );
    } catch (error) {
      diffResult = EMPTY_DIFF_RESULT;
      notify(
        translate($locale, "diff.readFailedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    } finally {
      loadingDiff = false;
    }
  }

  async function chooseRepositoryDirectory() {
    const selected = await openDialog({
      title: translate($locale, "repository.chooseDirectoryTitle"),
      directory: true,
      multiple: false,
    });

    if (!selected) {
      return null;
    }

    return Array.isArray(selected) ? (selected[0] ?? null) : selected;
  }

  function requestRepositoryRemoval(path: string) {
    if (!canRunRepositoryManagement()) {
      return;
    }

    repositoryPendingRemoval =
      repositories.find((repository) => repository.path === path) ??
      (currentRepository?.path === path ? currentRepository : null);
  }

  function cancelRepositoryRemoval() {
    if (managingRepositoryPath) {
      return;
    }

    repositoryPendingRemoval = null;
  }

  async function confirmRepositoryRemoval() {
    const repository = repositoryPendingRemoval;

    if (!repository || !canRunRepositoryManagement()) {
      return;
    }

    managingRepositoryPath = repository.path;

    try {
      await api.removeRepository(repository.path);
      repositoryPendingRemoval = null;
      await refreshRepositories();
      await loadCurrentRepositoryState();
      notify(
        translate($locale, "repository.removedTitle"),
        translate($locale, "repository.removedMessage"),
        "success",
      );
    } catch (error) {
      notify(
        translate($locale, "repository.removeFailedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    } finally {
      managingRepositoryPath = null;
    }
  }

  async function relocateRepositoryPath(path: string) {
    if (!canRunRepositoryManagement()) {
      return;
    }

    managingRepositoryPath = path;

    try {
      const newPath = await chooseRepositoryDirectory();

      if (!newPath) {
        return;
      }

      await api.relocateRepository(path, newPath);
      await refreshRepositories();
      await loadCurrentRepositoryState();
      notify(
        translate($locale, "repository.relocatedTitle"),
        translate($locale, "repository.relocatedMessage"),
        "success",
      );
    } catch (error) {
      notify(
        translate($locale, "repository.relocateFailedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    } finally {
      managingRepositoryPath = null;
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
        notify(
          translate($locale, "repository.addFailedTitle"),
          getErrorMessage(error, $locale),
          "error",
        );
      }
    }

    if (added) {
      await refreshRepositories();
      if (currentRepository) {
        await loadCurrentRepositoryState();
      }
      notify(
        translate($locale, "repository.addedTitle"),
        translate($locale, "repository.addedMessage"),
        "success",
      );
    }
  }

  async function setHideWhitespaceInDiff(value: boolean) {
    if (hideWhitespaceInDiff === value) {
      return;
    }

    hideWhitespaceInDiff = value;

    if (!currentRepository) {
      return;
    }

    if (!selectedCommit || !selectedFilePath) {
      return;
    }

    // whitespace 过滤会改变 Git 返回的 patch 结果，不能只在前端做字符串过滤。
    await loadDiff(selectedFilePath);
  }

  async function pushCurrentBranch() {
    const repository = currentRepository;
    const status = branchStatus;

    if (
      !repository ||
      !status ||
      !canPushCurrentBranch({
        branchStatus: status,
        switchingBranch,
        isPushing,
        stepPushState,
      })
    ) {
      return;
    }

    isPushing = true;

    try {
      const started = await api.startPushCurrentBranch(
        repository.path,
        status.branch,
      );
      if (pushToCommitState?.jobId !== started.jobId) {
        pushToCommitState = toRunningPushToCommitState({
          ...started,
          status: "preparing",
        });
      }
    } catch (error) {
      isPushing = false;
      notify(
        translate($locale, "push.failedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    }
  }

  function openContextMenu(commit: CommitListItem, x: number, y: number) {
    contextMenu = { open: true, x, y, commit };
  }

  function closeContextMenu() {
    contextMenu = { open: false, x: 0, y: 0, commit: null };
  }

  function clearStepPushPlanDialog() {
    stepPushPlanOpen = false;
    loadingStepPushPlan = false;
    submittingStepPushPlan = false;
    stepPushPlan = null;
    stepPushPlanErrorMessage = null;
    stepPushPlanRepoPath = null;
  }

  function closeStepPushPlanDialog() {
    if (submittingStepPushPlan) {
      return;
    }

    stepPushPlanRequestId += 1;
    clearStepPushPlanDialog();
  }

  async function pushToTargetCommit() {
    const commit = contextMenu.commit;
    const repository = currentRepository;
    const status = branchStatus;
    closeContextMenu();

    if (
      !commit ||
      !repository ||
      !status ||
      !canStartTargetCommitPush({
        commit,
        currentRepository: repository,
        branchStatus: status,
        switchingBranch,
        isPushing,
        stepPushState,
      })
    ) {
      return;
    }

    isPushing = true;

    try {
      const started = await api.startPushToCommit({
        repoPath: repository.path,
        branch: status.branch,
        hash: commit.hash,
      });
      if (pushToCommitState?.jobId !== started.jobId) {
        pushToCommitState = toRunningPushToCommitState({
          ...started,
          status: "preparing",
        });
      }
    } catch (error) {
      isPushing = false;
      notify(
        translate($locale, "push.failedTitle"),
        getErrorMessage(error, $locale),
        "error",
      );
    }
  }

  async function startStepPush() {
    const commit = contextMenu.commit;
    const repository = currentRepository;
    const status = branchStatus;
    closeContextMenu();

    if (
      !commit ||
      !repository ||
      !status ||
      !canStartStepPush({
        commit,
        currentRepository: repository,
        branchStatus: status,
        switchingBranch,
        isPushing,
        stepPushState,
      })
    ) {
      return;
    }

    stepPushPlanOpen = true;
    loadingStepPushPlan = true;
    submittingStepPushPlan = false;
    stepPushPlan = null;
    stepPushPlanErrorMessage = null;
    stepPushPlanRepoPath = repository.path;

    const requestId = ++stepPushPlanRequestId;

    try {
      await paintLoadingState();
      if (requestId !== stepPushPlanRequestId) {
        return;
      }

      const plan = await api.getStepPushPlan(repository.path, commit.hash);
      if (requestId !== stepPushPlanRequestId) {
        return;
      }

      stepPushPlan = plan;
    } catch (error) {
      if (requestId !== stepPushPlanRequestId) {
        return;
      }

      stepPushPlanErrorMessage = getErrorMessage(error, $locale);
    } finally {
      if (requestId === stepPushPlanRequestId) {
        loadingStepPushPlan = false;
      }
    }
  }

  async function confirmStepPushPlan() {
    if (!stepPushPlan || !stepPushPlanRepoPath || loadingStepPushPlan) {
      return;
    }

    submittingStepPushPlan = true;

    try {
      stepPushState = await startStepPushFromPlan(
        api,
        stepPushPlan,
        stepPushPlanRepoPath,
        1500,
        $locale,
      );
      stepPushPlanRequestId += 1;
      clearStepPushPlanDialog();
    } catch (error) {
      const message = getErrorMessage(error, $locale);
      stepPushPlanErrorMessage = message;
      notify(translate($locale, "stepPush.cannotStartTitle"), message, "error");
    } finally {
      submittingStepPushPlan = false;
    }
  }

  function clampLeftPaneWidth(value: number) {
    return Math.min(Math.max(value, MIN_LEFT_PANE_WIDTH), MAX_LEFT_PANE_WIDTH);
  }

  function startLayoutResize(target: "history", event: MouseEvent) {
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
        await listenPushToCommitProgress((payload) => {
          pushToCommitState = toRunningPushToCommitState(payload);
          isPushing = true;
        }),
      );

      disposers.push(
        await listenPushToCommitFinished((payload) => {
          const target = formatPushTargetLabel(
            payload.target,
            payload.targetKind,
            $locale,
          );
          pushToCommitState = toFinishedPushToCommitState(payload);
          isPushing = false;
          if (currentRepository) {
            invalidateRepositoryCache(currentRepository.path);
          }

          notify(
            translate($locale, "push.successTitle"),
            translate($locale, "push.successMessage", {
              target: target.message,
            }),
            "success",
          );

          void loadCurrentRepositoryState(true);

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
          pushToCommitState = toFailedPushToCommitState(payload, $locale);
          isPushing = false;
          if (currentRepository) {
            invalidateRepositoryCache(currentRepository.path);
          }

          notify(
            translate($locale, "push.failedTitle"),
            getErrorMessage(payload, $locale),
            "error",
          );

          void loadCurrentRepositoryState(true);

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
          if (currentRepository) {
            invalidateRepositoryCache(currentRepository.path);
          }

          notify(
            translate($locale, "stepPush.completeTitle"),
            translate($locale, "stepPush.completeMessage"),
            "success",
          );

          void loadCurrentRepositoryState(true);

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
          stepPushState = toFailedStepPushState(payload, $locale);
          if (currentRepository) {
            invalidateRepositoryCache(currentRepository.path);
          }

          notify(
            translate($locale, "stepPush.failedTitle"),
            getErrorMessage(payload, $locale),
            "error",
          );

          void loadCurrentRepositoryState(true);

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
      if (historyFilterTimer) {
        window.clearTimeout(historyFilterTimer);
      }
    };
  });
</script>

<svelte:head>
  <title>{translate($locale, "app.title")}</title>
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
<StepPushPlanDialog
  open={stepPushPlanOpen}
  loading={loadingStepPushPlan || submittingStepPushPlan}
  plan={stepPushPlan}
  errorMessage={stepPushPlanErrorMessage}
  on:confirm={confirmStepPushPlan}
  on:cancel={closeStepPushPlanDialog}
/>
<ConfirmRepositoryRemoveDialog
  repository={repositoryPendingRemoval}
  loading={Boolean(managingRepositoryPath)}
  on:confirm={confirmRepositoryRemoval}
  on:cancel={cancelRepositoryRemoval}
/>

<SettingsDialog open={settingsOpen} on:close={() => (settingsOpen = false)} />

<CommitContextMenu
  open={contextMenu.open}
  x={contextMenu.x}
  y={contextMenu.y}
  commit={contextMenu.commit}
  disabled={isContextMenuDisabled({
    switchingBranch,
    isPushing,
    stepPushState,
  })}
  pushToCommitDisabled={!contextMenu.commit?.isSafePushTarget}
  stepPushDisabled={!contextMenu.commit?.isSafePushTarget}
  pushToCommitReason={contextMenu.commit?.isSafePushTarget
    ? null
    : currentCommitPushBlockedReason()}
  stepPushReason={contextMenu.commit?.isSafePushTarget
    ? null
    : currentCommitPushBlockedReason()}
  on:pushToCommit={pushToTargetCommit}
  on:stepPush={startStepPush}
  on:close={closeContextMenu}
/>

<main
  class={`tg-app-shell flex h-screen min-h-0 flex-col overflow-hidden ${
    $textSelectionEnabled
      ? "tg-text-selection-enabled"
      : "tg-text-selection-disabled"
  }`}
>
  <header
    class="relative z-30 shrink-0 border-b border-tg-border-soft bg-tg-bg-panel backdrop-blur-xl"
  >
    <TopToolbar
      {repositories}
      {currentRepository}
      repositoryManagementDisabled={!canRunRepositoryManagementNow}
      {localBranches}
      {branchStatus}
      branchDisabled={isBranchSwitcherDisabled({
        currentRepository,
        loadingRepository: loadingRepository || syncingRemoteStatus,
        switchingBranch,
        isPushing,
        stepPushState,
      })}
      branchDisabledReason={currentBranchDisabledReason()}
      pushTitle={pushCardTitle}
      pushSubtitle={pushCardSubtitle}
      pushAheadCount={pushCardAheadCount}
      pushEnabled={canPushBranch}
      pushLoading={isPushing}
      pushBlockedReason={pushCardBlockedReason}
      pushAriaLabel={pushCardAriaLabel}
      refreshEnabled={canRefreshRemoteStatus}
      refreshLoading={syncingRemoteStatus}
      {settingsOpen}
      on:repositoryChange={(event) => switchRepository(event.detail.path)}
      on:repositoryRemove={(event) =>
        requestRepositoryRemoval(event.detail.path)}
      on:repositoryRelocate={(event) =>
        relocateRepositoryPath(event.detail.path)}
      on:repositoryCopyName={(event) =>
        copyRepositoryName(event.detail.repository)}
      on:repositoryCopyPath={(event) =>
        copyRepositoryPath(event.detail.repository)}
      on:repositoryViewGithub={(event) =>
        viewRepositoryOnGithub(event.detail.url)}
      on:repositoryOpenTerminal={(event) =>
        runRepositoryAction(
          event.detail.repository,
          api.openTerminalAtRepository,
          "repository.contextOpenedTerminal",
        )}
      on:repositoryRevealInFinder={(event) =>
        runRepositoryAction(
          event.detail.repository,
          api.revealRepositoryInFileManager,
          "repository.contextRevealedFinder",
        )}
      on:repositoryOpenInVSCode={(event) =>
        runRepositoryAction(
          event.detail.repository,
          api.openRepositoryInVSCode,
          "repository.contextOpenedVSCode",
        )}
      on:branchChange={(event) => switchBranch(event.detail.branch)}
      on:push={pushCurrentBranch}
      on:refresh={fetchRemoteStatusManually}
      on:openSettings={() => (settingsOpen = true)}
    />

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
                <span>{translate($locale, "remoteBlock.title")}</span>
              </div>
              <p class="mt-1 max-w-3xl text-xs leading-5 text-amber-100/85">
                {currentBranchDisabledReason()}
                {translate($locale, "remoteBlock.noAutoSync")}
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
              }) || syncingRemoteStatus}
              on:click={fetchRemoteStatusManually}
            >
              {syncingRemoteStatus
                ? translate($locale, "common.refreshing")
                : translate($locale, "remoteBlock.refreshStatus")}
            </button>
          </div>

          <div class="mt-3 grid gap-2 md:grid-cols-2">
            <div
              class="rounded-md border border-amber-200/15 bg-tg-bg-card/60 p-3"
            >
              <div class="text-xs font-semibold text-amber-100">
                {translate($locale, "remoteBlock.githubDesktop")}
              </div>
              <ol
                class="mt-2 list-decimal space-y-1 pl-4 text-xs text-amber-50/80"
              >
                <li>{translate($locale, "remoteBlock.githubStep1")}</li>
                <li>{translate($locale, "remoteBlock.githubStep2")}</li>
                <li>{translate($locale, "remoteBlock.githubStep3")}</li>
                <li>{translate($locale, "remoteBlock.githubStep4")}</li>
              </ol>
            </div>
            <div
              class="rounded-md border border-amber-200/15 bg-tg-bg-card/60 p-3"
            >
              <div class="text-xs font-semibold text-amber-100">
                {translate($locale, "remoteBlock.sourceTree")}
              </div>
              <ol
                class="mt-2 list-decimal space-y-1 pl-4 text-xs text-amber-50/80"
              >
                <li>{translate($locale, "remoteBlock.sourceTreeStep1")}</li>
                <li>{translate($locale, "remoteBlock.sourceTreeStep2")}</li>
                <li>{translate($locale, "remoteBlock.sourceTreeStep3")}</li>
                <li>{translate($locale, "remoteBlock.sourceTreeStep4")}</li>
              </ol>
            </div>
          </div>
        </div>
      {:else}
        <div
          class="border-t border-[#1f2328] bg-[#48322a] px-4 py-2 text-sm text-amber-100"
        >
          {currentBranchDisabledReason()}
        </div>
      {/if}
    {/if}
  </header>

  {#if shouldShowRepositoryUnavailableState(currentRepository)}
    <section
      class="flex min-h-0 flex-1 items-center justify-center bg-tg-bg-app px-6"
    >
      <div
        class="max-w-2xl rounded-tg-card border border-amber-300/25 bg-tg-bg-card p-6 text-center shadow-lg shadow-black/20 backdrop-blur-xl"
      >
        <div
          class="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-amber-400/12 text-xl font-bold text-amber-200"
        >
          !
        </div>
        <h2 class="mt-4 text-lg font-semibold text-tg-text-primary">
          {translate($locale, "repository.pathUnavailable")}
        </h2>
        <p class="mt-2 text-sm leading-6 text-slate-300">
          {currentRepository
            ? repositoryStatusMessage(currentRepository, $locale)
            : translate($locale, "repository.currentUnavailable")}
        </p>
        {#if currentRepository}
          <p class="mt-2 break-all font-mono text-xs text-slate-500">
            {currentRepository.path}
          </p>
          <div class="mt-5 flex flex-col justify-center gap-3 sm:flex-row">
            <button
              type="button"
              class="rounded-md border border-tg-blue-soft/45 bg-tg-blue/15 px-4 py-2 text-sm font-semibold text-sky-100 transition hover:bg-tg-blue/25 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={!canRunRepositoryManagementNow}
              on:click={() =>
                currentRepository &&
                relocateRepositoryPath(currentRepository.path)}
            >
              {translate($locale, "repository.relocate")}
            </button>
            <button
              type="button"
              class="rounded-md border border-rose-400/35 bg-rose-500/10 px-4 py-2 text-sm font-semibold text-rose-100 transition hover:bg-rose-500/18 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={!canRunRepositoryManagementNow}
              on:click={() =>
                currentRepository &&
                requestRepositoryRemoval(currentRepository.path)}
            >
              {translate($locale, "repository.removeFromList")}
            </button>
          </div>
        {/if}
      </div>
    </section>
  {:else}
    <section
      class="grid min-h-0 flex-1 bg-tg-bg-app"
      style={`grid-template-columns: minmax(${MIN_LEFT_PANE_WIDTH}px, ${leftPaneWidth}px) ${RESIZE_DIVIDER_LINE_WIDTH}px minmax(0,1fr);`}
    >
      <CommitHistoryList
        {commits}
        filters={historyFilters}
        activeFilterCount={activeHistoryFilterCount}
        selectedHash={selectedCommit?.hash ?? null}
        loading={loadingHistory || loadingRepository}
        totalCount={historyTotalCount}
        pageIndex={historyPageIndex}
        pageSize={PAGE_SIZE}
        {branchStatus}
        on:select={(event) => selectCommit(event.detail.commit)}
        on:pageChange={(event) => changeHistoryPage(event.detail.pageIndex)}
        on:openMenu={(event) =>
          openContextMenu(event.detail.commit, event.detail.x, event.detail.y)}
        on:filterChange={(event) => setHistoryFilters(event.detail.filters)}
        on:clearFilters={clearHistoryFilters}
      />

      <ResizeHandle
        active={activeResizeTarget === "history"}
        ariaLabel={translate($locale, "resize.historyAndDetails")}
        on:mousedown={(event) => startLayoutResize("history", event.detail)}
      />

      <CommitDetailPanel
        commit={selectedCommit}
        commitMeta={selectedCommitMeta}
        files={commitFiles}
        {loadingFiles}
        {loadingDiff}
        {selectedFilePath}
        {diffResult}
        {diffViewMode}
        {hideWhitespaceInDiff}
        {branchStatus}
        on:selectFile={(event) => loadDiff(event.detail.path)}
        on:diffModeChange={(event) => (diffViewMode = event.detail.mode)}
        on:hideWhitespaceChange={(event) =>
          setHideWhitespaceInDiff(event.detail.value)}
      />
    </section>
  {/if}
</main>
