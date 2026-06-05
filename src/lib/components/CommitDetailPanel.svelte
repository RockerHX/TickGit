<script lang="ts">
  import { onDestroy } from "svelte";
  import { createEventDispatcher } from "svelte";
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
  import type {
    CommitFileChange,
    CommitFileDiffResult,
    CommitListItem,
    CommitMeta,
  } from "$lib/types";
  import { formatAbsoluteDate, getInitials, statusTone } from "$lib/utils";

  export let commit: CommitListItem | null = null;
  export let commitMeta: CommitMeta | null = null;
  export let files: CommitFileChange[] = [];
  export let selectedFilePath: string | null = null;
  export let diffResult: CommitFileDiffResult;
  export let loadingFiles = false;
  export let loadingDiff = false;
  export let diffViewMode: "unified" | "split" = "unified";
  export let hideWhitespaceInDiff = false;

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
  let copyResetTimer: ReturnType<typeof setTimeout> | null = null;
  let filePathCopyResetTimer: ReturnType<typeof setTimeout> | null = null;

  $: if (commit && copiedCommitHash !== commit.hash) {
    copiedCommitHash = null;
  }

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
      console.error("Failed to copy commit hash:", error);
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
      console.error("Failed to copy file path:", error);
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
  class="flex h-full min-h-0 flex-col overflow-hidden bg-[#2b3036]"
  bind:this={panelElement}
>
  <div class="border-b border-[#1f2328] bg-[#2d333b] px-5 py-3">
    {#if commit}
      <div class="min-w-0">
        <div class="flex items-start gap-3">
          <div class="min-w-0 flex-1">
            <div class="truncate text-[1.15rem] font-semibold text-[#f0f6fc]">
              {commit.summary}
            </div>
            {#if commit.tags.length > 0}
              <div class="mt-2 flex flex-wrap items-center gap-1.5">
                {#each commit.tags as tag}
                  <span
                    class="max-w-full truncate rounded-full border border-amber-400/30 bg-amber-400/10 px-2.5 py-0.5 text-[11px] font-medium text-amber-200"
                    title={tag}
                  >
                    {tag}
                  </span>
                {/each}
              </div>
            {/if}
            {#if commitMeta?.body}
              <div
                class="mt-1 whitespace-pre-wrap text-[0.95rem] leading-6 text-slate-200"
              >
                {commitMeta.body}
              </div>
            {/if}
          </div>
          {#if !commit.isPushed}
            <span
              class="mt-0.5 flex shrink-0 items-center text-[#f0f6fc]"
              title="Local commit"
            >
              <svg
                viewBox="0 0 16 16"
                class="h-4 w-4 fill-current"
                aria-hidden="true"
              >
                <path
                  d="M8 12.75a.75.75 0 0 1-.75-.75V6.81L5.53 8.53a.75.75 0 1 1-1.06-1.06l3-3a.75.75 0 0 1 1.06 0l3 3a.75.75 0 0 1-1.06 1.06L8.75 6.81V12a.75.75 0 0 1-.75.75Z"
                ></path>
              </svg>
            </span>
          {/if}
        </div>

        <div class="mt-3 flex items-center gap-2 text-[13px] text-[#f0f6fc]">
          <div
            class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-[#444c56] bg-[#373e47] text-[10px] font-semibold"
          >
            {getInitials(commit.authorName)}
          </div>
          <span class="truncate font-medium">{commit.authorName}</span>
          <span class="truncate text-slate-300"
            >&lt;{commit.authorEmail}&gt;</span
          >
        </div>

        <div
          class="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 text-[13px] text-[#f0f6fc]"
        >
          <svg
            viewBox="0 0 16 16"
            class="h-4 w-4 shrink-0 fill-current"
            aria-hidden="true"
          >
            <path
              d="M1.75 8a2.75 2.75 0 1 1 5.18 1.28h2.14a2.751 2.751 0 0 1 5.18-1.28 2.75 2.75 0 1 1-5.18 1.28H6.93A2.75 2.75 0 1 1 1.75 8Zm2.75-1.25a1.25 1.25 0 1 0 0 2.5 1.25 1.25 0 0 0 0-2.5Zm7 0a1.25 1.25 0 1 0 .001 2.501A1.25 1.25 0 0 0 11.5 6.75Z"
            ></path>
          </svg>
          <span class="font-mono">{commit.hash}</span>
          <button
            type="button"
            class="ml-1 inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-md border border-[#444c56] bg-[#373e47] text-[#f0f6fc] transition hover:border-[#539bf5]/50 hover:bg-[#347dff]/15"
            title={copiedCommitHash === commit.hash
              ? "已复制 Commit Hash"
              : "复制 Commit Hash"}
            aria-label={copiedCommitHash === commit.hash
              ? "已复制 Commit Hash"
              : "复制 Commit Hash"}
            on:click={() => copyCommitHash(commit.hash)}
          >
            {#if copiedCommitHash === commit.hash}
              <svg
                viewBox="0 0 16 16"
                class="h-4 w-4 fill-current text-emerald-300"
                aria-hidden="true"
              >
                <path
                  d="M13.78 4.97a.75.75 0 0 1 0 1.06L7.53 12.28a.75.75 0 0 1-1.06 0L2.22 8.03a.75.75 0 1 1 1.06-1.06L7 10.69l5.72-5.72a.75.75 0 0 1 1.06 0Z"
                ></path>
              </svg>
            {:else}
              <svg
                viewBox="0 0 16 16"
                class="h-4 w-4 fill-current"
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
          <span class="text-slate-400"
            >{formatAbsoluteDate(commit.committedAt)}</span
          >
        </div>

        {#if commitMeta}
          <div
            class="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-[13px]"
          >
            <svg
              viewBox="0 0 16 16"
              class="h-4 w-4 shrink-0 fill-[#f0f6fc]"
              aria-hidden="true"
            >
              <path
                d="M7.25 1.75a.75.75 0 0 1 1.5 0v5.5h5.5a.75.75 0 0 1 0 1.5h-5.5v5.5a.75.75 0 0 1-1.5 0v-5.5h-5.5a.75.75 0 0 1 0-1.5h5.5Z"
              ></path>
            </svg>
            <span class="font-medium text-emerald-300">
              {commitMeta.additions} added lines
            </span>
            <span class="font-medium text-rose-300">
              {commitMeta.deletions} removed lines
            </span>
          </div>
        {/if}
      </div>
    {:else}
      <div class="text-sm text-slate-500">
        Select a commit to inspect its details
      </div>
    {/if}
  </div>

  <div
    class="grid min-h-0 flex-1"
    style={`grid-template-columns: minmax(${MIN_FILES_PANE_WIDTH}px, ${filesPaneWidth}px) ${RESIZE_DIVIDER_LINE_WIDTH}px minmax(${MIN_BRANCH_PANE_WIDTH}px,1fr);`}
  >
    <div class="flex min-h-0 flex-col bg-[#2d333b]">
      <div
        class="flex items-center justify-between gap-3 border-b border-[#1f2328] px-4 py-3"
      >
        <div class="text-sm font-semibold text-[#f0f6fc]">changed files</div>
        <div class="text-xs font-medium text-slate-400">{files.length}</div>
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto bg-[#2d333b]">
        {#if loadingFiles}
          <div class="px-4 py-4 text-sm text-slate-400">
            Loading changed files…
          </div>
        {:else if files.length === 0}
          <div
            class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2b3036] px-4 py-8 text-center text-sm text-slate-500"
          >
            No file changes available for this commit
          </div>
        {:else}
          <div>
            {#each files as file (file.path + file.status)}
              <div
                class={`flex items-center gap-2 border-b border-[#373e47] px-4 py-2.5 transition ${
                  selectedFilePath === file.path
                    ? "bg-[#347dff]/12"
                    : "bg-transparent hover:bg-[#373e47]/45"
                }`}
              >
                <button
                  type="button"
                  class="min-w-0 flex-1 text-left"
                  title={file.displayPath}
                  on:click={() => dispatch("selectFile", { path: file.path })}
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
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <ResizeHandle
      active={isResizingFilesPane}
      ariaLabel="Resize changed files and diff panels"
      on:mousedown={(event) => startFilesPaneResize(event.detail)}
    />

    <DiffViewer
      title="Diff"
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
</div>
