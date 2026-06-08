<script lang="ts">
  import { onDestroy } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import { createEventDispatcher } from "svelte";
  import ChangedFilesPanel from "$lib/components/ChangedFilesPanel.svelte";
  import CommitDetailHeader from "$lib/components/CommitDetailHeader.svelte";
  import CommitMessagePanel from "$lib/components/CommitMessagePanel.svelte";
  import DiffViewer from "$lib/components/DiffViewer.svelte";
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

  async function copyFilePath(path: string) {
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
    class="grid min-h-0 flex-1"
    style={`grid-template-columns: minmax(${MIN_FILES_PANE_WIDTH}px, ${filesPaneWidth}px) ${RESIZE_DIVIDER_LINE_WIDTH}px minmax(${MIN_BRANCH_PANE_WIDTH}px,1fr);`}
  >
    <div class="flex min-h-0 min-w-0 flex-col bg-tg-bg-panel">
      <div class="shrink-0 border-b border-tg-border-soft px-2.5 py-2">
        <CommitDetailHeader
          {commit}
          {commitMeta}
          collapsed={commitHeaderCollapsed}
          {copiedCommitHash}
          on:toggleCollapsed={() =>
            (commitHeaderCollapsed = !commitHeaderCollapsed)}
          on:copyHash={(event) => copyCommitHash(event.detail.hash)}
        >
          {#if commit && !commitHeaderCollapsed}
            <CommitMessagePanel {commit} body={commitBody} {branchStatus} />
          {/if}
        </CommitDetailHeader>
      </div>

      <ChangedFilesPanel
        {files}
        {selectedFilePath}
        {loadingFiles}
        {copiedFilePath}
        on:selectFile={(event) => dispatch("selectFile", event.detail)}
        on:copyPath={(event) => copyFilePath(event.detail.path)}
      />
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
