<script lang="ts">
  import { onMount } from "svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import BranchSwitcher from "$lib/components/BranchSwitcher.svelte";
  import CommitContextMenu from "$lib/components/CommitContextMenu.svelte";
  import CommitDetailPanel from "$lib/components/CommitDetailPanel.svelte";
  import CommitHistoryList from "$lib/components/CommitHistoryList.svelte";
  import DropOverlay from "$lib/components/DropOverlay.svelte";
  import PushToCommitOverlay from "$lib/components/PushToCommitOverlay.svelte";
  import ResizeHandle from "$lib/components/ResizeHandle.svelte";
  import RepositorySwitcher from "$lib/components/RepositorySwitcher.svelte";
  import StepPushPlanDialog from "$lib/components/StepPushPlanDialog.svelte";
  import StepPushOverlay from "$lib/components/StepPushOverlay.svelte";
  import ToastViewport from "$lib/components/ToastViewport.svelte";
  import WorkspaceChangesPanel from "$lib/components/WorkspaceChangesPanel.svelte";
  import { api } from "$lib/tauri/api";
  import {
    listenPushToCommitFailed,
    listenPushToCommitFinished,
    listenStepPushFailed,
    listenStepPushFinished,
    listenStepPushProgress,
  } from "$lib/tauri/events";
  import {
    EMPTY_DIFF_RESULT,
    fetchCommitDetails,
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
  import { createToastItem, getErrorMessage } from "$lib/tickgit/page-helpers";
  import {
    canLoadCommitFiles,
    canLoadDiff,
    canLoadHistory,
    canPushCurrentBranch,
    canCreateWorkspaceCommit,
    canRefreshBlockedBranchStatus,
    canRefreshCurrentRepositoryOnFocus,
    canStartStepPush,
    canStartTargetCommitPush,
    canSwitchBranch,
    canWriteWorkspace,
    isBranchSwitcherDisabled,
    isContextMenuDisabled,
    isRepositoryAvailable,
    shouldClearRepositoryData,
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
    EMPTY_WORKSPACE_STATUS,
    fetchWorkspaceSnapshot,
    getWorkspaceCommitFailureEffect,
    getWorkspaceCommitSuccessEffect,
    workspaceFileKey,
    type WorkspaceSelection,
  } from "$lib/tickgit/workspace";
  import {
    MAX_LEFT_PANE_WIDTH,
    MIN_BRANCH_PANE_WIDTH,
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
    RepositorySummary,
    StepPushPlan,
    StepPushUiState,
    ToastItem,
    WorkspaceChangeSection,
    WorkspaceStatus,
  } from "$lib/types";

  const PAGE_SIZE = 50;
  const TOAST_TIMEOUT = 3400;
  const WINDOW_RESIZE_SAVE_DEBOUNCE_MS = 300;
  const HISTORY_FILTER_DEBOUNCE_MS = 300;
  // 失败态既会自动消失，也允许用户手动关闭，避免错误浮层长时间阻塞界面。
  const PUSH_OVERLAY_DISMISS_MS = 3600;

  let repositories: RepositorySummary[] = [];
  let currentRepository: RepositorySummary | null = null;
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
  let activeMainView: "history" | "changes" = "history";

  let workspaceStatus: WorkspaceStatus = EMPTY_WORKSPACE_STATUS;
  let selectedWorkspaceSection: WorkspaceChangeSection | null = null;
  let selectedWorkspaceFilePath: string | null = null;
  let workspaceDiffResult: CommitFileDiffResult = EMPTY_DIFF_RESULT;

  let nextSkip = 0;
  let hasMore = false;
  let loadingHistory = false;
  let loadingRepository = true;
  let loadingFiles = false;
  let loadingDiff = false;
  let loadingWorkspace = false;
  let loadingWorkspaceDiff = false;
  let workspaceActionFileKey: string | null = null;
  let workspaceCommitMessage = "";
  let committingWorkspace = false;

  let dragActive = false;
  let isPushing = false;
  let switchingBranch = false;
  let managingRepositoryPath: string | null = null;
  let activeResizeTarget: "header" | "history" | null = null;
  let leftPaneWidth = 360;

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
    diffResult = snapshot.diffResult;
  }

  function currentWorkspaceSelection(): WorkspaceSelection | null {
    if (!selectedWorkspaceSection || !selectedWorkspaceFilePath) {
      return null;
    }

    return {
      section: selectedWorkspaceSection,
      path: selectedWorkspaceFilePath,
    };
  }

  function resetWorkspaceState() {
    workspaceStatus = EMPTY_WORKSPACE_STATUS;
    selectedWorkspaceSection = null;
    selectedWorkspaceFilePath = null;
    workspaceDiffResult = EMPTY_DIFF_RESULT;
    workspaceActionFileKey = null;
    workspaceCommitMessage = "";
    committingWorkspace = false;
  }

  function applyWorkspaceSnapshot(
    snapshot: Awaited<ReturnType<typeof fetchWorkspaceSnapshot>>,
  ) {
    workspaceStatus = snapshot.status;
    selectedWorkspaceSection = snapshot.selectedSection;
    selectedWorkspaceFilePath = snapshot.selectedFilePath;
    workspaceDiffResult = snapshot.diffResult;
  }

  function canRunWorkspaceAction() {
    return canWriteWorkspace({
      currentRepository,
      loadingRepository,
      loadingWorkspace,
      switchingBranch,
      isPushing,
      stepPushState,
      workspaceActionFileKey,
      committingWorkspace,
    });
  }

  function canCommitWorkspaceChanges() {
    return canCreateWorkspaceCommit({
      currentRepository,
      loadingRepository,
      loadingWorkspace,
      switchingBranch,
      isPushing,
      stepPushState,
      workspaceActionFileKey,
      committingWorkspace,
      commitMessage: workspaceCommitMessage,
      stagedCount: workspaceStatus.staged.length,
    });
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
    loadingHistory = false;
    loadingFiles = false;
    loadingDiff = false;
    isPushing = false;
    switchingBranch = false;
    pushToCommitState = null;
    stepPushState = null;
    contextMenu = { open: false, x: 0, y: 0, commit: null };
    resetWorkspaceState();
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
    return (
      !managingRepositoryPath &&
      canManageRepositories({
        loadingRepository,
        switchingBranch,
        isPushing,
        stepPushState,
      })
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

    if (activeMainView === "changes") {
      await loadWorkspaceState(currentRepository.path, keepSelection);
    }
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

  function setHistoryFilters(filters: CommitHistoryFilters) {
    const nextFilters = normalizeHistoryFilters(filters);

    if (historyFiltersEqual(historyFilters, nextFilters)) {
      return;
    }

    historyFilters = nextFilters;
    scheduleHistoryFilterReload();
  }

  function clearHistoryFilters() {
    if (activeHistoryFilterCount === 0) {
      return;
    }

    historyFilters = { ...EMPTY_HISTORY_FILTERS };
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
    clearHistoryDetailState();

    historyFilterTimer = window.setTimeout(() => {
      historyFilterTimer = null;
      void loadHistory(false);
    }, HISTORY_FILTER_DEBOUNCE_MS);
  }

  async function switchMainView(view: "history" | "changes") {
    activeMainView = view;

    if (
      view === "changes" &&
      currentRepository &&
      isRepositoryAvailable(currentRepository)
    ) {
      await loadWorkspaceState(currentRepository.path, true);
    }
  }

  async function bootstrap() {
    loadingRepository = true;

    try {
      const bootstrapState = await loadBootstrapRepositoryState(api, {
        pageSize: PAGE_SIZE,
        keepSelection: false,
        previousSelectedHash: null,
        ignoreWhitespace: hideWhitespaceInDiff,
        filters: historyFilters,
        preferredFilePathFilter: historyFilters.filePath,
      });
      repositories = bootstrapState.repositories;
      currentRepository = bootstrapState.currentRepository;

      if (bootstrapState.repositoryState) {
        notifyRemoteRefreshError(bootstrapState.repositoryState);
        applyRepositoryState(bootstrapState.repositoryState);
      } else if (shouldClearRepositoryData(currentRepository)) {
        clearRepositoryData();
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
      resetWorkspaceState();

      await loadCurrentRepositoryState();
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
        filters: historyFilters,
        preferredFilePathFilter: historyFilters.filePath,
      });

      notifyRemoteRefreshError(repositoryState);
      applyRepositoryState(repositoryState);
    } catch (error) {
      notify("读取仓库失败", getErrorMessage(error), "error");
    } finally {
      loadingRepository = false;
    }
  }

  async function loadWorkspaceState(path: string, keepSelection = false) {
    loadingWorkspace = true;

    try {
      const snapshot = await fetchWorkspaceSnapshot(
        api,
        path,
        keepSelection,
        currentWorkspaceSelection(),
        hideWhitespaceInDiff,
      );
      applyWorkspaceSnapshot(snapshot);
    } catch (error) {
      resetWorkspaceState();
      notify("读取工作区失败", getErrorMessage(error), "error");
    } finally {
      loadingWorkspace = false;
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

    try {
      await api.checkoutBranch(repository.path, branch);
      await loadRepositoryState(repository.path);
      if (activeMainView === "changes") {
        await loadWorkspaceState(repository.path);
      }
      notify("分支已切换", `当前已切换到 ${branch}`, "success");
    } catch (error) {
      notify("切换分支失败", getErrorMessage(error), "error");
    } finally {
      loadingRepository = false;
      switchingBranch = false;
    }
  }

  async function refreshCurrentRepositoryOnFocus() {
    const repository = currentRepository;

    if (
      !repository ||
      !canRefreshCurrentRepositoryOnFocus({
        currentRepository: repository,
        loadingRepository,
        loadingHistory,
      })
    ) {
      return;
    }

    await loadCurrentRepositoryState(true);
  }

  async function refreshBlockedBranchStatus() {
    const repository = currentRepository;

    if (
      !repository ||
      !canRefreshBlockedBranchStatus({
        currentRepository: repository,
        loadingRepository,
        switchingBranch,
        isPushing,
        stepPushState,
      })
    ) {
      return;
    }

    await loadRepositoryState(repository.path, true);
  }

  async function loadHistory(append: boolean) {
    const repository = currentRepository;
    const requestId = ++historyRequestId;
    const filters = normalizeHistoryFilters(historyFilters);

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
        append ? nextSkip : 0,
        PAGE_SIZE,
        filters,
      );

      if (requestId !== historyRequestId) {
        return;
      }

      commits = append ? [...commits, ...page.items] : page.items;
      nextSkip = page.nextSkip;
      hasMore = page.hasMore;

      if (!append) {
        const selected = page.items[0] ?? null;
        if (selected) {
          await selectCommit(selected, filters.filePath);
        } else {
          clearHistoryDetailState();
        }
      }
    } catch (error) {
      if (requestId === historyRequestId) {
        notify("加载历史失败", getErrorMessage(error), "error");
      }
    } finally {
      if (requestId === historyRequestId) {
        loadingHistory = false;
      }
    }
  }

  async function selectCommit(
    commit: CommitListItem,
    preferredFilePathFilter: string | null = historyFilters.filePath ?? null,
  ) {
    selectedCommit = commit;
    await loadCommitFiles(commit.hash, preferredFilePathFilter);
  }

  async function loadCommitFiles(
    hash: string,
    preferredFilePathFilter: string | null = historyFilters.filePath ?? null,
  ) {
    const repository = currentRepository;

    if (!repository || !canLoadCommitFiles({ currentRepository: repository })) {
      return;
    }

    loadingFiles = true;
    selectedFilePath = null;
    diffResult = EMPTY_DIFF_RESULT;

    try {
      const details = await fetchCommitDetails(
        api,
        repository.path,
        hash,
        hideWhitespaceInDiff,
        preferredFilePathFilter,
      );
      selectedCommitMeta = details.commitMeta;
      commitFiles = details.commitFiles;
      selectedFilePath = details.selectedFilePath;
      diffResult = details.diffResult;
    } catch (error) {
      selectedCommitMeta = null;
      notify("读取 Commit 详情失败", getErrorMessage(error), "error");
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

    try {
      const selectedFile = commitFiles.find((file) => file.path === filePath);
      diffResult = await api.getCommitFileDiff(
        repository.path,
        commit.hash,
        filePath,
        hideWhitespaceInDiff,
        selectedFile?.previousPath ?? null,
      );
    } catch (error) {
      diffResult = EMPTY_DIFF_RESULT;
      notify("读取 Diff 失败", getErrorMessage(error), "error");
    } finally {
      loadingDiff = false;
    }
  }

  async function loadWorkspaceDiff(
    section: WorkspaceChangeSection,
    filePath: string,
  ) {
    const repository = currentRepository;

    if (!repository) {
      return;
    }

    loadingWorkspaceDiff = true;
    selectedWorkspaceSection = section;
    selectedWorkspaceFilePath = filePath;

    try {
      const selectedFile = [
        ...workspaceStatus.staged,
        ...workspaceStatus.unstaged,
      ].find((file) => file.section === section && file.path === filePath);
      workspaceDiffResult = await api.getWorkspaceFileDiff(
        repository.path,
        section,
        filePath,
        hideWhitespaceInDiff,
        selectedFile?.previousPath ?? null,
      );
    } catch (error) {
      workspaceDiffResult = EMPTY_DIFF_RESULT;
      notify("读取工作区 Diff 失败", getErrorMessage(error), "error");
    } finally {
      loadingWorkspaceDiff = false;
    }
  }

  async function stageWorkspaceFile(
    section: WorkspaceChangeSection,
    filePath: string,
  ) {
    const repository = currentRepository;

    if (!repository || !canRunWorkspaceAction()) {
      return;
    }

    workspaceActionFileKey = workspaceFileKey({ section, path: filePath });

    try {
      await api.stageWorkspaceFile(repository.path, filePath);
      await loadWorkspaceState(repository.path, true);
      notify("文件已暂存", filePath, "success");
    } catch (error) {
      notify("暂存文件失败", getErrorMessage(error), "error");
    } finally {
      workspaceActionFileKey = null;
    }
  }

  async function unstageWorkspaceFile(
    section: WorkspaceChangeSection,
    filePath: string,
  ) {
    const repository = currentRepository;

    if (!repository || !canRunWorkspaceAction()) {
      return;
    }

    workspaceActionFileKey = workspaceFileKey({ section, path: filePath });

    try {
      await api.unstageWorkspaceFile(repository.path, filePath);
      await loadWorkspaceState(repository.path, true);
      notify("文件已取消暂存", filePath, "success");
    } catch (error) {
      notify("取消暂存失败", getErrorMessage(error), "error");
    } finally {
      workspaceActionFileKey = null;
    }
  }

  async function commitWorkspaceChanges() {
    const repository = currentRepository;

    if (!repository || !canCommitWorkspaceChanges()) {
      return;
    }

    committingWorkspace = true;

    try {
      const created = await api.createCommit(
        repository.path,
        workspaceCommitMessage,
      );
      const effect = getWorkspaceCommitSuccessEffect();
      workspaceCommitMessage = effect.nextCommitMessage;

      if (effect.refreshWorkspace) {
        await loadWorkspaceState(repository.path);
      }

      if (effect.refreshRepository) {
        await loadRepositoryState(repository.path);
      }

      notify("提交成功", `${created.shortHash} ${created.summary}`, "success");
    } catch (error) {
      const effect = getWorkspaceCommitFailureEffect(workspaceCommitMessage);
      workspaceCommitMessage = effect.nextCommitMessage;
      notify("提交失败", getErrorMessage(error), "error");
    } finally {
      committingWorkspace = false;
    }
  }

  async function chooseRepositoryDirectory() {
    const selected = await openDialog({
      title: "选择新的仓库目录",
      directory: true,
      multiple: false,
    });

    if (!selected) {
      return null;
    }

    return Array.isArray(selected) ? (selected[0] ?? null) : selected;
  }

  async function removeRepositoryFromList(path: string) {
    if (!canRunRepositoryManagement()) {
      return;
    }

    managingRepositoryPath = path;

    try {
      await api.removeRepository(path);
      await refreshRepositories();
      await loadCurrentRepositoryState();
      notify("仓库已移除", "仅从 TickGit 列表移除，本地文件未删除", "success");
    } catch (error) {
      notify("移除仓库失败", getErrorMessage(error), "error");
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
      notify("仓库已重新定位", "已更新仓库路径并刷新状态", "success");
    } catch (error) {
      notify("重新定位失败", getErrorMessage(error), "error");
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
        notify("添加仓库失败", getErrorMessage(error), "error");
      }
    }

    if (added) {
      await refreshRepositories();
      if (currentRepository) {
        await loadCurrentRepositoryState();
      }
      notify("仓库已添加", "新的 Git 仓库已加入 TickGit", "success");
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

    if (activeMainView === "changes") {
      if (selectedWorkspaceSection && selectedWorkspaceFilePath) {
        await loadWorkspaceDiff(
          selectedWorkspaceSection,
          selectedWorkspaceFilePath,
        );
      }
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
      pushToCommitState = toRunningPushToCommitState(started);
    } catch (error) {
      isPushing = false;
      notify("推送失败", getErrorMessage(error), "error");
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
      const plan = await api.getStepPushPlan(repository.path, commit.hash);
      if (requestId !== stepPushPlanRequestId) {
        return;
      }

      stepPushPlan = plan;
    } catch (error) {
      if (requestId !== stepPushPlanRequestId) {
        return;
      }

      stepPushPlanErrorMessage = getErrorMessage(error);
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
      );
      stepPushPlanRequestId += 1;
      clearStepPushPlanDialog();
    } catch (error) {
      const message = getErrorMessage(error);
      stepPushPlanErrorMessage = message;
      notify("无法开始分步提交", message, "error");
    } finally {
      submittingStepPushPlan = false;
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
          const target = formatPushTargetLabel(
            payload.target,
            payload.targetKind,
          );
          pushToCommitState = toFailedPushToCommitState(payload);
          isPushing = false;

          notify("推送失败", payload.message, "error");

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

          notify("分步提交完成", "所有目标 Commit 已按顺序推送", "success");

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
          stepPushState = toFailedStepPushState(payload);

          notify("分步提交失败", payload.message, "error");

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
<StepPushPlanDialog
  open={stepPushPlanOpen}
  loading={loadingStepPushPlan || submittingStepPushPlan}
  plan={stepPushPlan}
  errorMessage={stepPushPlanErrorMessage}
  on:confirm={confirmStepPushPlan}
  on:cancel={closeStepPushPlanDialog}
/>

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
              managementDisabled={!canRunRepositoryManagement()}
              on:change={(event) => switchRepository(event.detail.path)}
              on:remove={(event) => removeRepositoryFromList(event.detail.path)}
              on:relocate={(event) => relocateRepositoryPath(event.detail.path)}
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

    <div class="border-t border-[#1f2328] bg-[#24292f] px-4 py-2">
      <div
        class="inline-flex overflow-hidden rounded-md border border-[#444c56] bg-[#2d333b] p-0.5"
      >
        {#each [{ id: "history", label: "History" }, { id: "changes", label: "Changes" }] as item}
          <button
            type="button"
            class={`rounded px-3 py-1.5 text-xs font-semibold transition ${
              activeMainView === item.id
                ? "bg-[#347dff]/20 text-[#f0f6fc]"
                : "text-slate-400 hover:bg-[#373e47] hover:text-slate-200"
            }`}
            on:click={() =>
              void switchMainView(item.id as "history" | "changes")}
          >
            {item.label}
          </button>
        {/each}
      </div>
    </div>
  </header>

  {#if shouldShowRepositoryUnavailableState(currentRepository)}
    <section
      class="flex min-h-0 flex-1 items-center justify-center bg-[#2b3036] px-6"
    >
      <div
        class="max-w-2xl rounded-lg border border-amber-300/25 bg-[#24292f] p-6 text-center shadow-lg shadow-black/20"
      >
        <div
          class="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-amber-400/12 text-xl font-bold text-amber-200"
        >
          !
        </div>
        <h2 class="mt-4 text-lg font-semibold text-[#f0f6fc]">
          仓库路径不可用
        </h2>
        <p class="mt-2 text-sm leading-6 text-slate-300">
          {currentRepository
            ? repositoryStatusMessage(currentRepository)
            : "当前仓库不可用"}
        </p>
        {#if currentRepository}
          <p class="mt-2 break-all font-mono text-xs text-slate-500">
            {currentRepository.path}
          </p>
          <div class="mt-5 flex flex-col justify-center gap-3 sm:flex-row">
            <button
              type="button"
              class="rounded-md border border-[#539bf5]/45 bg-[#347dff]/15 px-4 py-2 text-sm font-semibold text-[#cae8ff] transition hover:bg-[#347dff]/25 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={!canRunRepositoryManagement()}
              on:click={() =>
                currentRepository &&
                relocateRepositoryPath(currentRepository.path)}
            >
              重新定位
            </button>
            <button
              type="button"
              class="rounded-md border border-rose-400/35 bg-rose-500/10 px-4 py-2 text-sm font-semibold text-rose-100 transition hover:bg-rose-500/18 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={!canRunRepositoryManagement()}
              on:click={() =>
                currentRepository &&
                removeRepositoryFromList(currentRepository.path)}
            >
              从列表移除
            </button>
          </div>
        {/if}
      </div>
    </section>
  {:else if activeMainView === "history"}
    <section
      class="grid min-h-0 flex-1 bg-[#2b3036]"
      style={`grid-template-columns: minmax(${MIN_LEFT_PANE_WIDTH}px, ${leftPaneWidth}px) ${RESIZE_DIVIDER_LINE_WIDTH}px minmax(0,1fr);`}
    >
      <CommitHistoryList
        {commits}
        filters={historyFilters}
        activeFilterCount={activeHistoryFilterCount}
        selectedHash={selectedCommit?.hash ?? null}
        loading={loadingHistory || loadingRepository}
        {hasMore}
        {branchStatus}
        on:select={(event) => selectCommit(event.detail.commit)}
        on:loadMore={() => loadHistory(true)}
        on:openMenu={(event) =>
          openContextMenu(event.detail.commit, event.detail.x, event.detail.y)}
        on:filterChange={(event) => setHistoryFilters(event.detail.filters)}
        on:clearFilters={clearHistoryFilters}
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
        {diffResult}
        {diffViewMode}
        {hideWhitespaceInDiff}
        on:selectFile={(event) => loadDiff(event.detail.path)}
        on:diffModeChange={(event) => (diffViewMode = event.detail.mode)}
        on:hideWhitespaceChange={(event) =>
          setHideWhitespaceInDiff(event.detail.value)}
      />
    </section>
  {:else}
    <section class="min-h-0 flex-1 bg-[#2b3036]">
      <WorkspaceChangesPanel
        status={workspaceStatus}
        selectedSection={selectedWorkspaceSection}
        selectedFilePath={selectedWorkspaceFilePath}
        diffResult={workspaceDiffResult}
        {loadingWorkspace}
        loadingDiff={loadingWorkspaceDiff}
        {diffViewMode}
        {hideWhitespaceInDiff}
        workspaceActionsDisabled={!canRunWorkspaceAction()}
        {workspaceActionFileKey}
        commitMessage={workspaceCommitMessage}
        commitDisabled={!canCommitWorkspaceChanges()}
        {committingWorkspace}
        on:selectFile={(event) =>
          loadWorkspaceDiff(event.detail.section, event.detail.path)}
        on:stageFile={(event) =>
          stageWorkspaceFile(event.detail.section, event.detail.path)}
        on:unstageFile={(event) =>
          unstageWorkspaceFile(event.detail.section, event.detail.path)}
        on:commitMessageChange={(event) =>
          (workspaceCommitMessage = event.detail.value)}
        on:commit={commitWorkspaceChanges}
        on:diffModeChange={(event) => (diffViewMode = event.detail.mode)}
        on:hideWhitespaceChange={(event) =>
          setHideWhitespaceInDiff(event.detail.value)}
      />
    </section>
  {/if}
</main>
