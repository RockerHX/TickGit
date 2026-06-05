<script lang="ts">
  import { onDestroy } from "svelte";
  import { createEventDispatcher } from "svelte";
  import DiffViewer from "$lib/components/DiffViewer.svelte";
  import ResizeHandle from "$lib/components/ResizeHandle.svelte";
  import {
    DEFAULT_FILES_PANE_WIDTH,
    MAX_FILES_PANE_WIDTH,
    MIN_BRANCH_PANE_WIDTH,
    MIN_FILES_PANE_WIDTH,
    RESIZE_DIVIDER_LINE_WIDTH,
  } from "$lib/tickgit/layout";
  import { writeClipboardText } from "$lib/tickgit/clipboard";
  import { workspaceFileKey } from "$lib/tickgit/workspace";
  import type {
    CommitFileDiffResult,
    WorkspaceChangeSection,
    WorkspaceFileChange,
    WorkspaceStatus,
  } from "$lib/types";
  import { statusTone } from "$lib/utils";

  export let status: WorkspaceStatus = {
    staged: [],
    unstaged: [],
  };
  export let selectedSection: WorkspaceChangeSection | null = null;
  export let selectedFilePath: string | null = null;
  export let diffResult: CommitFileDiffResult;
  export let loadingWorkspace = false;
  export let loadingDiff = false;
  export let diffViewMode: "unified" | "split" = "unified";
  export let hideWhitespaceInDiff = false;
  export let workspaceActionsDisabled = false;
  export let workspaceActionFileKey: string | null = null;
  export let commitMessage = "";
  export let commitDisabled = false;
  export let committingWorkspace = false;

  const dispatch = createEventDispatcher<{
    selectFile: { section: WorkspaceChangeSection; path: string };
    stageFile: { section: WorkspaceChangeSection; path: string };
    unstageFile: { section: WorkspaceChangeSection; path: string };
    commitMessageChange: { value: string };
    commit: void;
    diffModeChange: { mode: "unified" | "split" };
    hideWhitespaceChange: { value: boolean };
  }>();

  let isResizingFilesPane = false;
  let filesPaneWidth = DEFAULT_FILES_PANE_WIDTH;
  let panelElement: HTMLDivElement | null = null;
  let copiedFilePath: string | null = null;
  let filePathCopyResetTimer: ReturnType<typeof setTimeout> | null = null;

  $: totalChanges = status.staged.length + status.unstaged.length;

  function clampFilesPaneWidth(value: number) {
    if (!panelElement) {
      return Math.min(
        Math.max(value, MIN_FILES_PANE_WIDTH),
        MAX_FILES_PANE_WIDTH,
      );
    }

    const maxWidth = Math.min(
      MAX_FILES_PANE_WIDTH,
      Math.max(
        MIN_FILES_PANE_WIDTH,
        panelElement.clientWidth - MIN_BRANCH_PANE_WIDTH,
      ),
    );

    return Math.min(Math.max(value, MIN_FILES_PANE_WIDTH), maxWidth);
  }

  function startFilesPaneResize(event: MouseEvent) {
    isResizingFilesPane = true;
    applyFilesPaneResize(event.clientX);
  }

  function applyFilesPaneResize(pointerX: number) {
    if (!panelElement) {
      return;
    }

    const bounds = panelElement.getBoundingClientRect();
    filesPaneWidth = clampFilesPaneWidth(pointerX - bounds.left);
  }

  function isSelected(file: WorkspaceFileChange) {
    return file.section === selectedSection && file.path === selectedFilePath;
  }

  function selectFile(file: WorkspaceFileChange) {
    dispatch("selectFile", { section: file.section, path: file.path });
  }

  function actionLabel(file: WorkspaceFileChange) {
    return file.section === "staged" ? "Unstage" : "Stage";
  }

  function isActionRunning(file: WorkspaceFileChange) {
    return workspaceActionFileKey === workspaceFileKey(file);
  }

  function runFileAction(file: WorkspaceFileChange) {
    const detail = { section: file.section, path: file.path };

    if (file.section === "staged") {
      dispatch("unstageFile", detail);
      return;
    }

    dispatch("stageFile", detail);
  }

  function setCommitMessage(value: string) {
    dispatch("commitMessageChange", { value });
  }

  async function copyFilePath(event: MouseEvent, path: string) {
    event.stopPropagation();

    try {
      await writeClipboardText(path);
      copiedFilePath = path;

      if (filePathCopyResetTimer) {
        clearTimeout(filePathCopyResetTimer);
      }

      filePathCopyResetTimer = setTimeout(() => {
        copiedFilePath = null;
      }, 1600);
    } catch (error) {
      console.error("Failed to copy file path:", error);
      copiedFilePath = null;
    }
  }

  onDestroy(() => {
    if (filePathCopyResetTimer) {
      clearTimeout(filePathCopyResetTimer);
    }
  });
</script>

<svelte:window
  on:mousemove={(event) => {
    if (isResizingFilesPane) {
      applyFilesPaneResize(event.clientX);
    }
  }}
  on:mouseup={() => {
    isResizingFilesPane = false;
  }}
/>

<div
  class="grid h-full min-h-0 overflow-hidden bg-[#2b3036]"
  style={`grid-template-columns: minmax(${MIN_FILES_PANE_WIDTH}px, ${filesPaneWidth}px) ${RESIZE_DIVIDER_LINE_WIDTH}px minmax(${MIN_BRANCH_PANE_WIDTH}px,1fr);`}
  bind:this={panelElement}
>
  <div class="flex min-h-0 flex-col bg-[#2d333b]">
    <div
      class="flex items-center justify-between gap-3 border-b border-[#1f2328] px-4 py-3"
    >
      <div>
        <div class="text-sm font-semibold text-[#f0f6fc]">Changes</div>
        <div class="mt-1 text-xs text-slate-400">
          {status.staged.length} staged · {status.unstaged.length} unstaged
        </div>
      </div>
      <div class="text-xs font-medium text-slate-400">{totalChanges}</div>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto bg-[#2d333b]">
      {#if loadingWorkspace}
        <div class="px-4 py-4 text-sm text-slate-400">
          Loading workspace changes…
        </div>
      {:else if totalChanges === 0}
        <div
          class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2b3036] px-4 py-8 text-center text-sm text-slate-500"
        >
          No workspace changes
        </div>
      {:else}
        {#each [{ title: "Staged changes", files: status.staged }, { title: "Unstaged changes", files: status.unstaged }] as group}
          <div class="border-b border-[#1f2328]">
            <div
              class="flex items-center justify-between gap-3 bg-[#24292f] px-4 py-2 text-xs font-semibold uppercase tracking-[0.16em] text-slate-400"
            >
              <span>{group.title}</span>
              <span>{group.files.length}</span>
            </div>

            {#if group.files.length === 0}
              <div class="px-4 py-3 text-xs text-slate-500">No files</div>
            {:else}
              {#each group.files as file (workspaceFileKey(file))}
                <div
                  class={`flex items-center gap-2 border-t border-[#373e47] px-4 py-2.5 transition ${
                    isSelected(file)
                      ? "bg-[#347dff]/12"
                      : "bg-transparent hover:bg-[#373e47]/45"
                  }`}
                >
                  <button
                    type="button"
                    class="min-w-0 flex-1 text-left"
                    title={file.displayPath}
                    on:click={() => selectFile(file)}
                  >
                    <div class="flex items-center gap-3">
                      <span
                        class={`flex h-6 min-w-6 items-center justify-center rounded-full border px-1.5 text-[10px] font-semibold uppercase ${statusTone(file.status)}`}
                      >
                        {file.status}
                      </span>
                      <span
                        class="min-w-0 flex-1 truncate text-[13px] leading-5 text-slate-200"
                      >
                        {file.displayPath}
                      </span>
                    </div>
                  </button>
                  <button
                    type="button"
                    class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md border border-[#444c56] bg-[#373e47] text-slate-200 transition hover:border-[#539bf5]/50 hover:bg-[#347dff]/15"
                    title={copiedFilePath === file.path
                      ? "已复制文件路径"
                      : "复制文件路径"}
                    aria-label={copiedFilePath === file.path
                      ? "已复制文件路径"
                      : "复制文件路径"}
                    on:click={(event) => copyFilePath(event, file.path)}
                  >
                    {#if copiedFilePath === file.path}
                      <svg
                        viewBox="0 0 16 16"
                        class="h-3.5 w-3.5 fill-current text-emerald-300"
                        aria-hidden="true"
                      >
                        <path
                          d="M13.78 4.97a.75.75 0 0 1 0 1.06L7.53 12.28a.75.75 0 0 1-1.06 0L2.22 8.03a.75.75 0 1 1 1.06-1.06L7 10.69l5.72-5.72a.75.75 0 0 1 1.06 0Z"
                        ></path>
                      </svg>
                    {:else}
                      <svg
                        viewBox="0 0 16 16"
                        class="h-3.5 w-3.5 fill-current"
                        aria-hidden="true"
                      >
                        <path
                          d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25Z"
                        ></path>
                        <path
                          d="M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"
                        ></path>
                      </svg>
                    {/if}
                  </button>
                  <button
                    type="button"
                    class="h-7 shrink-0 rounded-md border border-[#444c56] bg-[#373e47] px-2.5 text-[11px] font-semibold text-slate-200 transition hover:border-[#539bf5]/50 hover:bg-[#347dff]/15 disabled:cursor-not-allowed disabled:opacity-50"
                    disabled={workspaceActionsDisabled || isActionRunning(file)}
                    on:click={() => runFileAction(file)}
                  >
                    {isActionRunning(file) ? "..." : actionLabel(file)}
                  </button>
                </div>
              {/each}
            {/if}
          </div>
        {/each}
      {/if}
    </div>

    <div class="border-t border-[#1f2328] bg-[#24292f] p-4">
      <label
        class="text-xs font-semibold uppercase tracking-[0.16em] text-slate-400"
        for="workspace-commit-message"
      >
        Commit message
      </label>
      <textarea
        id="workspace-commit-message"
        class="mt-2 min-h-20 w-full resize-none rounded-md border border-[#444c56] bg-[#2d333b] px-3 py-2 text-sm leading-5 text-[#f0f6fc] outline-none transition placeholder:text-slate-500 focus:border-[#539bf5]/70 disabled:cursor-not-allowed disabled:opacity-60"
        placeholder="Summary of staged changes"
        value={commitMessage}
        disabled={committingWorkspace}
        on:input={(event) => setCommitMessage(event.currentTarget.value)}
      ></textarea>
      <button
        type="button"
        class="mt-3 h-9 w-full rounded-md bg-[#238636] px-3 text-sm font-semibold text-white transition hover:bg-[#2ea043] disabled:cursor-not-allowed disabled:bg-[#2d333b] disabled:text-slate-500"
        disabled={commitDisabled}
        on:click={() => dispatch("commit")}
      >
        {committingWorkspace
          ? "Committing…"
          : `Commit ${status.staged.length} staged file${
              status.staged.length === 1 ? "" : "s"
            }`}
      </button>
      <div class="mt-2 text-xs leading-5 text-slate-500">
        Only staged files will be committed. Unstaged files remain in the
        workspace.
      </div>
    </div>
  </div>

  <ResizeHandle
    active={isResizingFilesPane}
    ariaLabel="Resize workspace files and diff panels"
    on:mousedown={(event) => startFilesPaneResize(event.detail)}
  />

  <DiffViewer
    title="Workspace Diff"
    {selectedFilePath}
    {diffResult}
    {loadingDiff}
    mode={diffViewMode}
    {hideWhitespaceInDiff}
    on:modeChange={(event) => dispatch("diffModeChange", event.detail)}
    on:hideWhitespaceChange={(event) =>
      dispatch("hideWhitespaceChange", event.detail)}
  />
</div>
