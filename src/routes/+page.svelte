<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import CommitContextMenu from "$lib/components/CommitContextMenu.svelte";
  import CommitDetailPanel from "$lib/components/CommitDetailPanel.svelte";
  import CommitHistoryList from "$lib/components/CommitHistoryList.svelte";
  import DropOverlay from "$lib/components/DropOverlay.svelte";
  import RepositorySwitcher from "$lib/components/RepositorySwitcher.svelte";
  import StepPushOverlay from "$lib/components/StepPushOverlay.svelte";
  import ToastViewport from "$lib/components/ToastViewport.svelte";
  import { api } from "$lib/tauri/api";
  import {
    listenStepPushFailed,
    listenStepPushFinished,
    listenStepPushProgress,
  } from "$lib/tauri/events";
  import {
    fetchCommitDetails,
    fetchRepositoryIndex,
    fetchRepositorySnapshot,
  } from "$lib/tickgit/page-data";
  import {
    buildStepPushHashes,
    createToastItem,
    getErrorMessage,
    toFailedStepPushState,
    toFinishedStepPushState,
    toRunningStepPushState,
  } from "$lib/tickgit/page-helpers";
  import type {
    BranchStatus,
    CommitFileChange,
    CommitListItem,
    RepositorySummary,
    StepPushUiState,
    ToastItem,
  } from "$lib/types";

  const PAGE_SIZE = 50;
  const TOAST_TIMEOUT = 3400;

  let repositories: RepositorySummary[] = [];
  let currentRepository: RepositorySummary | null = null;
  let branchStatus: BranchStatus | null = null;

  let commits: CommitListItem[] = [];
  let selectedCommit: CommitListItem | null = null;
  let commitFiles: CommitFileChange[] = [];
  let selectedFilePath: string | null = null;
  let diffText = "";

  let nextSkip = 0;
  let hasMore = false;
  let loadingHistory = false;
  let loadingRepository = true;
  let loadingFiles = false;
  let loadingDiff = false;

  let dragActive = false;
  let isPushing = false;

  let toasts: ToastItem[] = [];
  let toastId = 1;
  let stepPushState: StepPushUiState | null = null;

  let contextMenu = {
    open: false,
    x: 0,
    y: 0,
    commit: null as CommitListItem | null,
  };

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
      const repositoryIndex = await fetchRepositoryIndex(api);
      repositories = repositoryIndex.repositories;
      currentRepository = repositoryIndex.currentRepository;

      if (currentRepository) {
        await loadRepositoryState(currentRepository.path);
      }
    } catch (error) {
      notify("初始化失败", getErrorMessage(error), "error");
    } finally {
      loadingRepository = false;
    }
  }

  async function refreshRepositories() {
    const repositoryIndex = await fetchRepositoryIndex(api);
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
      const snapshot = await fetchRepositorySnapshot(
        api,
        path,
        PAGE_SIZE,
        keepSelection,
        selectedCommit?.hash ?? null,
      );

      branchStatus = snapshot.branchStatus;
      commits = snapshot.commits;
      nextSkip = snapshot.nextSkip;
      hasMore = snapshot.hasMore;
      selectedCommit = snapshot.selectedCommit;
      commitFiles = snapshot.commitFiles;
      selectedFilePath = snapshot.selectedFilePath;
      diffText = snapshot.diffText;
    } catch (error) {
      notify("读取仓库失败", getErrorMessage(error), "error");
    } finally {
      loadingRepository = false;
    }
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
      );
      commitFiles = details.commitFiles;
      selectedFilePath = details.selectedFilePath;
      diffText = details.diffText;
    } catch (error) {
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
      diffText = await api.getCommitFileDiff(
        currentRepository.path,
        selectedCommit.hash,
        filePath,
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

  async function pushCurrentBranch() {
    if (!currentRepository || !branchStatus?.pushAvailable || isPushing) {
      return;
    }

    isPushing = true;

    try {
      await api.pushCurrentBranch(currentRepository.path);
      notify("推送成功", "已执行常规 git push", "success");
      await loadRepositoryState(currentRepository.path, true);
    } catch (error) {
      notify("推送失败", getErrorMessage(error), "error");
    } finally {
      isPushing = false;
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
      await api.pushToCommit(
        currentRepository.path,
        branchStatus.branch,
        commit.hash,
      );
      notify("推送成功", `已推送到 Commit ${commit.shortHash}`, "success");
      await loadRepositoryState(currentRepository.path, true);
    } catch (error) {
      notify("推送失败", getErrorMessage(error), "error");
    } finally {
      isPushing = false;
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
      notify("无法分步提交", "目标 Commit 不在未推送列表中", "error");
      return;
    }

    try {
      const started = await api.startStepPush({
        repoPath: currentRepository.path,
        branch: branchStatus.branch,
        hashes,
        delayMs: 1500,
      });

      stepPushState = {
        jobId: started.jobId,
        current: 0,
        total: started.total,
        hash: hashes[0],
        status: "running",
      };
    } catch (error) {
      notify("无法开始分步提交", getErrorMessage(error), "error");
    }
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
            if (stepPushState?.jobId === payload.jobId) {
              stepPushState = null;
            }
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
        }),
      );
    })();

    const closeMenu = () => closeContextMenu();
    window.addEventListener("click", closeMenu);

    return () => {
      for (const dispose of disposers) {
        dispose();
      }
      window.removeEventListener("click", closeMenu);
    };
  });
</script>

<svelte:head>
  <title>TickGit</title>
</svelte:head>

<DropOverlay active={dragActive} />
<ToastViewport {toasts} />
<StepPushOverlay state={stepPushState} />

<CommitContextMenu
  open={contextMenu.open}
  x={contextMenu.x}
  y={contextMenu.y}
  commit={contextMenu.commit}
  disabled={isPushing || stepPushState?.status === "running"}
  on:pushToCommit={pushToTargetCommit}
  on:stepPush={startStepPush}
  on:close={closeContextMenu}
/>

<main class="flex h-screen flex-col bg-transparent px-5 py-5 text-slate-200">
  <header
    class="mb-5 rounded-[28px] border border-slate-800/80 bg-slate-950/70 px-5 py-4 shadow-2xl shadow-slate-950/50 backdrop-blur"
  >
    <div class="flex items-center justify-between gap-4">
      <div class="flex items-center gap-4">
        <div>
          <div
            class="text-[11px] font-semibold uppercase tracking-[0.28em] text-slate-500"
          >
            TickGit
          </div>
          <div class="mt-1 text-2xl font-semibold text-white">
            {currentRepository?.name ?? "未选择仓库"}
          </div>
        </div>

        <RepositorySwitcher
          {repositories}
          currentPath={currentRepository?.path ?? null}
          on:change={(event) => switchRepository(event.detail.path)}
        />
      </div>

      <div class="flex items-center gap-3">
        <div
          class="rounded-2xl border border-slate-800 bg-slate-900/70 px-4 py-3"
        >
          <div
            class="text-[11px] font-semibold uppercase tracking-[0.24em] text-slate-500"
          >
            Branch
          </div>
          <div class="mt-1 text-sm font-medium text-white">
            {branchStatus?.branch ?? "N/A"}
          </div>
          {#if branchStatus?.upstream}
            <div class="mt-1 text-xs text-slate-400">
              {branchStatus.upstream}
            </div>
          {/if}
        </div>

        <button
          class="rounded-2xl bg-sky-500 px-4 py-3 text-sm font-semibold text-slate-950 transition hover:bg-sky-400 disabled:cursor-not-allowed disabled:bg-slate-800 disabled:text-slate-500"
          disabled={!branchStatus?.pushAvailable ||
            branchStatus.aheadCount === 0 ||
            isPushing ||
            stepPushState?.status === "running"}
          on:click={pushCurrentBranch}
        >
          {isPushing
            ? "Pushing…"
            : `Push${branchStatus?.aheadCount ? ` (${branchStatus.aheadCount})` : ""}`}
        </button>
      </div>
    </div>

    {#if branchStatus && !branchStatus.pushAvailable}
      <div
        class="mt-3 rounded-2xl border border-amber-500/25 bg-amber-500/10 px-4 py-3 text-sm text-amber-100"
      >
        {branchStatus.disabledReason}
      </div>
    {/if}
  </header>

  <section class="grid min-h-0 flex-1 grid-cols-[420px_minmax(0,1fr)] gap-5">
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

    <CommitDetailPanel
      commit={selectedCommit}
      files={commitFiles}
      {loadingFiles}
      {loadingDiff}
      {selectedFilePath}
      {diffText}
      on:selectFile={(event) => loadDiff(event.detail.path)}
    />
  </section>
</main>
