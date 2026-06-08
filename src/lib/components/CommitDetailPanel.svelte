<script lang="ts">
  import { onDestroy } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import { createEventDispatcher } from "svelte";
  import CommitDetailHeader from "$lib/components/CommitDetailHeader.svelte";
  import DiffViewer from "$lib/components/DiffViewer.svelte";
  import FileTypeIcon from "$lib/components/FileTypeIcon.svelte";
  import ResizeHandle from "$lib/components/ResizeHandle.svelte";
  import {
    DEFAULT_FILES_PANE_WIDTH,
    MAX_FILES_PANE_WIDTH,
    MIN_FILES_PANE_WIDTH,
    MIN_BRANCH_PANE_WIDTH,
    RESIZE_DIVIDER_LINE_WIDTH,
  } from "$lib/tickgit/layout";
  import { writeClipboardText } from "$lib/tickgit/clipboard";
  import { commitInfoDefaultCollapsed } from "$lib/tickgit/preferences";
  import type {
    BranchStatus,
    CommitFileChange,
    CommitFileDiffResult,
    CommitListItem,
    CommitMeta,
  } from "$lib/types";

  export let commit: CommitListItem | null = null;
  export let commitMeta: CommitMeta | null = null;
  export let files: CommitFileChange[] = [];
  export let selectedFilePath: string | null = null;
  export let diffResult: CommitFileDiffResult;
  export let loadingFiles = false;
  export let loadingDiff = false;
  export let diffViewMode: "unified" | "split" = "unified";
  export let hideWhitespaceInDiff = false;
  export let branchStatus: BranchStatus | null = null;

  const dispatch = createEventDispatcher<{
    selectFile: { path: string };
    diffModeChange: { mode: "unified" | "split" };
    hideWhitespaceChange: { value: boolean };
  }>();

  let isResizingFilesPane = false;
  let filesPaneWidth = DEFAULT_FILES_PANE_WIDTH;
  let panelElement: HTMLDivElement | null = null;
  let copiedCommitHash: string | null = null;
  let copiedFilePath: string | null = null;
  let commitHeaderCollapsed = false;
  let previousCommitHash: string | null = null;
  let copyResetTimer: ReturnType<typeof setTimeout> | null = null;
  let filePathCopyResetTimer: ReturnType<typeof setTimeout> | null = null;

  $: if (commit?.hash !== previousCommitHash) {
    previousCommitHash = commit?.hash ?? null;
    commitHeaderCollapsed = $commitInfoDefaultCollapsed;
    copiedCommitHash = null;
  }

  $: selectedFile =
    files.find((file) => file.path === selectedFilePath) ?? null;
  $: commitBody = commitMeta?.body.trim() ?? "";

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

  async function copyCommitHash(hash: string) {
    try {
      await writeClipboardText(hash);
      copiedCommitHash = hash;

      if (copyResetTimer) {
        clearTimeout(copyResetTimer);
      }

      copyResetTimer = setTimeout(() => {
        copiedCommitHash = null;
      }, 2000);
    } catch (error) {
      console.error(translate($locale, "commit.copyHashFailedLog"), error);
      copiedCommitHash = null;
    }
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
      console.error(translate($locale, "file.copyPathFailedLog"), error);
      copiedFilePath = null;
    }
  }

  onDestroy(() => {
    if (copyResetTimer) {
      clearTimeout(copyResetTimer);
    }
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
  class="flex h-full min-h-0 flex-col overflow-hidden bg-tg-bg-app"
  bind:this={panelElement}
>
  <div
    class="border-b border-tg-border-soft bg-tg-bg-panel px-4 py-3 shadow-[0_10px_30px_rgba(15,23,42,0.22)]"
  >
    <CommitDetailHeader
      {commit}
      {commitMeta}
      {branchStatus}
      collapsed={commitHeaderCollapsed}
      {copiedCommitHash}
      on:toggleCollapsed={() =>
        (commitHeaderCollapsed = !commitHeaderCollapsed)}
      on:copyHash={(event) => copyCommitHash(event.detail.hash)}
    >
      {#if commit && !commitHeaderCollapsed && commitBody}
        <section
          class="mt-2.5 rounded-lg border border-tg-border-soft bg-tg-bg-panel px-3 py-2"
          aria-label={translate($locale, "commit.messageTitle")}
        >
          <div
            class="mb-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-tg-text-muted"
          >
            {translate($locale, "commit.messageTitle")}
          </div>
          <div
            class="max-h-20 overflow-y-auto whitespace-pre-wrap break-words font-mono text-[11px] leading-5 text-tg-text-secondary"
          >
            {commitBody}
          </div>
        </section>
      {/if}
    </CommitDetailHeader>
  </div>

  <div
    class="grid min-h-0 flex-1"
    style={`grid-template-columns: minmax(${MIN_FILES_PANE_WIDTH}px, ${filesPaneWidth}px) ${RESIZE_DIVIDER_LINE_WIDTH}px minmax(${MIN_BRANCH_PANE_WIDTH}px,1fr);`}
  >
    <div class="flex min-h-0 flex-col bg-tg-bg-panel">
      <div
        class="flex items-center justify-between gap-3 border-b border-tg-border-soft bg-tg-bg-card px-4 py-3"
      >
        <div class="text-sm font-semibold text-tg-text-primary">
          {translate($locale, "commit.changedFiles")}
        </div>
        <div
          class="rounded-full border border-tg-border-soft bg-white/[0.06] px-2.5 py-0.5 text-xs font-semibold text-tg-text-secondary"
        >
          {files.length}
        </div>
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto bg-tg-bg-panel">
        {#if loadingFiles}
          <div class="tg-card m-4 px-4 py-5 text-sm text-tg-text-secondary/80">
            {translate($locale, "commit.loadingFiles")}
          </div>
        {:else if files.length === 0}
          <div
            class="m-4 rounded-xl border border-dashed border-tg-border-soft bg-tg-bg-card/70 px-4 py-8 text-center text-sm text-tg-text-muted"
          >
            {translate($locale, "commit.noFileChanges")}
          </div>
        {:else}
          <div class="space-y-2 p-3">
            {#each files as file (file.path + file.status)}
              <div
                class={`relative flex items-center gap-2 overflow-hidden rounded-xl border px-3 py-2.5 transition ${
                  selectedFilePath === file.path
                    ? "border-tg-blue-soft/25 bg-tg-blue-soft/15 shadow-[0_14px_30px_rgba(59,130,246,0.14)]"
                    : "border-tg-border-soft bg-tg-bg-card/70 hover:border-tg-blue-soft/20 hover:bg-white/[0.06]"
                }`}
              >
                {#if selectedFilePath === file.path}
                  <span
                    class="absolute bottom-2 left-0 top-2 w-1 rounded-r-full bg-tg-blue-soft"
                    aria-hidden="true"
                  ></span>
                {/if}
                <button
                  type="button"
                  class="min-w-0 flex-1 text-left"
                  title={file.displayPath}
                  on:click={() => dispatch("selectFile", { path: file.path })}
                >
                  <div class="flex min-w-0 items-center gap-3">
                    <FileTypeIcon {file} />
                    <span
                      class="min-w-0 flex-1 truncate text-[13px] leading-5 text-tg-text-secondary"
                    >
                      {file.displayPath}
                    </span>
                  </div>
                </button>
                {#if typeof file.additions === "number" && typeof file.deletions === "number"}
                  <div
                    class="flex shrink-0 items-center gap-1.5 text-[11px] font-semibold tabular-nums"
                    aria-label={`+${file.additions} -${file.deletions}`}
                  >
                    <span class="text-emerald-300">+{file.additions}</span>
                    <span class="text-rose-300">-{file.deletions}</span>
                  </div>
                {/if}
                <button
                  type="button"
                  class="tg-control tg-focus-ring flex h-7 w-7 shrink-0 items-center justify-center border-transparent"
                  title={copiedFilePath === file.path
                    ? translate($locale, "file.copiedPath")
                    : translate($locale, "file.copyPath")}
                  aria-label={copiedFilePath === file.path
                    ? translate($locale, "file.copiedPath")
                    : translate($locale, "file.copyPath")}
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
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <ResizeHandle
      active={isResizingFilesPane}
      ariaLabel={translate($locale, "resize.changedFilesAndDiff")}
      on:mousedown={(event) => startFilesPaneResize(event.detail)}
    />

    <div class="flex min-h-0 min-w-0 flex-col">
      <DiffViewer
        title={translate($locale, "diff.title")}
        {selectedFilePath}
        {selectedFile}
        {diffResult}
        {loadingDiff}
        mode={diffViewMode}
        {hideWhitespaceInDiff}
        on:modeChange={(event) => dispatch("diffModeChange", event.detail)}
        on:hideWhitespaceChange={(event) =>
          dispatch("hideWhitespaceChange", event.detail)}
      />
    </div>
  </div>
</div>
