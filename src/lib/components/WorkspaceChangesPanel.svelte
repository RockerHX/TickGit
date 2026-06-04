<script lang="ts">
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

  const dispatch = createEventDispatcher<{
    selectFile: { section: WorkspaceChangeSection; path: string };
    diffModeChange: { mode: "unified" | "split" };
    hideWhitespaceChange: { value: boolean };
  }>();

  let isResizingFilesPane = false;
  let filesPaneWidth = DEFAULT_FILES_PANE_WIDTH;
  let panelElement: HTMLDivElement | null = null;

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
                <button
                  class={`w-full border-t border-[#373e47] px-4 py-2.5 text-left transition ${
                    isSelected(file)
                      ? "bg-[#347dff]/12"
                      : "bg-transparent hover:bg-[#373e47]/45"
                  }`}
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
              {/each}
            {/if}
          </div>
        {/each}
      {/if}
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
