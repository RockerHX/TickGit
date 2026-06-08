<script lang="ts">
  import { onDestroy } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import { createEventDispatcher } from "svelte";
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
  import type {
    BranchStatus,
    CommitFileChange,
    CommitFileDiffResult,
    CommitListItem,
    CommitMeta,
  } from "$lib/types";
  import { formatAbsoluteDate, getInitials } from "$lib/utils";

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
  let copyResetTimer: ReturnType<typeof setTimeout> | null = null;
  let filePathCopyResetTimer: ReturnType<typeof setTimeout> | null = null;

  $: if (commit && copiedCommitHash !== commit.hash) {
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
  class="flex h-full min-h-0 flex-col overflow-hidden bg-[#2b3036]"
  bind:this={panelElement}
>
  <div
    class="border-b border-[#1f2328]/80 bg-[#111827] px-5 py-4 shadow-[0_10px_30px_rgba(15,23,42,0.28)]"
  >
    {#if commit}
      <div
        class="min-w-0 rounded-xl border border-white/10 bg-[#18202d]/80 p-4 shadow-[0_18px_42px_rgba(8,13,24,0.22)]"
      >
        <div class="flex items-start justify-between gap-4">
          <div class="min-w-0 flex-1">
            <div
              class="truncate text-xl font-semibold leading-7 tracking-[-0.01em] text-slate-50"
              title={commit.summary}
            >
              {commit.summary}
            </div>
          </div>
          <span
            class="mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-white/10 bg-white/[0.04] text-slate-400"
            aria-hidden="true"
          >
            <svg
              viewBox="0 0 16 16"
              class="h-4 w-4 fill-current"
              aria-hidden="true"
            >
              <path
                d="M3.22 9.53a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1-1.06 1.06L8 5.81 4.28 9.53a.75.75 0 0 1-1.06 0Z"
              ></path>
            </svg>
          </span>
        </div>

        {#if commit.tags.length > 0 || !commit.isPushed}
          <div class="mt-2 flex flex-wrap items-center gap-1.5">
            {#if !commit.isPushed}
              <span
                class="inline-flex items-center gap-1 rounded-full border border-sky-400/25 bg-sky-400/10 px-2.5 py-0.5 text-[11px] font-medium text-sky-200"
                title={translate($locale, "commit.local")}
              >
                <svg
                  viewBox="0 0 16 16"
                  class="h-3.5 w-3.5 fill-current"
                  aria-hidden="true"
                >
                  <path
                    d="M8 12.75a.75.75 0 0 1-.75-.75V6.81L5.53 8.53a.75.75 0 1 1-1.06-1.06l3-3a.75.75 0 0 1 1.06 0l3 3a.75.75 0 0 1-1.06 1.06L8.75 6.81V12a.75.75 0 0 1-.75.75Z"
                  ></path>
                </svg>
                {translate($locale, "commit.local")}
              </span>
            {/if}
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

        <div
          class="mt-4 flex flex-wrap items-center justify-between gap-x-4 gap-y-3 text-[13px] text-slate-100"
        >
          <div class="flex min-w-0 flex-1 items-center gap-2.5">
            <div
              class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full border border-sky-300/25 bg-sky-400/15 text-[10px] font-semibold tracking-wide text-sky-100"
            >
              {getInitials(commit.authorName)}
            </div>
            <div
              class="min-w-0 flex-1"
              title={`${commit.authorName} <${commit.authorEmail}>`}
            >
              <div class="flex min-w-0 items-center gap-1.5">
                <span class="min-w-0 truncate font-medium text-slate-100">
                  {commit.authorName}
                </span>
                <span class="min-w-0 truncate text-slate-400">
                  &lt;{commit.authorEmail}&gt;
                </span>
              </div>
            </div>
          </div>

          <div
            class="ml-auto flex min-w-0 flex-wrap items-center justify-end gap-2 text-[12px]"
          >
            <button
              type="button"
              class="inline-flex h-8 max-w-full shrink-0 items-center gap-2 rounded-full border border-sky-300/20 bg-sky-400/10 px-3 font-mono font-medium text-sky-100 transition hover:border-sky-300/45 hover:bg-sky-400/15 focus:outline-none focus:ring-2 focus:ring-sky-400/40"
              title={commit.hash}
              aria-label={copiedCommitHash === commit.hash
                ? translate($locale, "commit.copiedHash")
                : translate($locale, "commit.copyHash")}
              on:click={() => copyCommitHash(commit.hash)}
            >
              <svg
                viewBox="0 0 16 16"
                class="h-3.5 w-3.5 shrink-0 fill-current text-sky-300"
                aria-hidden="true"
              >
                <path
                  d="M1.75 8a2.75 2.75 0 1 1 5.18 1.28h2.14a2.751 2.751 0 0 1 5.18-1.28 2.75 2.75 0 1 1-5.18 1.28H6.93A2.75 2.75 0 1 1 1.75 8Zm2.75-1.25a1.25 1.25 0 1 0 0 2.5 1.25 1.25 0 0 0 0-2.5Zm7 0a1.25 1.25 0 1 0 .001 2.501A1.25 1.25 0 0 0 11.5 6.75Z"
                ></path>
              </svg>
              <span class="truncate">{commit.shortHash}</span>
              {#if copiedCommitHash === commit.hash}
                <svg
                  viewBox="0 0 16 16"
                  class="h-3.5 w-3.5 shrink-0 fill-current text-emerald-300"
                  aria-hidden="true"
                >
                  <path
                    d="M13.78 4.97a.75.75 0 0 1 0 1.06L7.53 12.28a.75.75 0 0 1-1.06 0L2.22 8.03a.75.75 0 1 1 1.06-1.06L7 10.69l5.72-5.72a.75.75 0 0 1 1.06 0Z"
                  ></path>
                </svg>
              {:else}
                <svg
                  viewBox="0 0 16 16"
                  class="h-3.5 w-3.5 shrink-0 fill-current text-slate-300"
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
            <span class="text-right leading-5 text-slate-400">
              {formatAbsoluteDate(commit.committedAt, $locale)}
            </span>
          </div>
        </div>

        <div class="mt-3 flex flex-wrap items-center justify-between gap-2">
          {#if commitMeta}
            <div
              class="flex flex-wrap items-center gap-2 text-[13px] font-medium"
            >
              <span
                class="inline-flex items-center gap-1.5 rounded-full border border-emerald-300/20 bg-emerald-400/10 px-3 py-1 text-emerald-300"
              >
                <span class="font-mono text-sm leading-none">+</span>
                {translate($locale, "commit.addedLines", {
                  count: commitMeta.additions,
                })}
              </span>
              <span
                class="inline-flex items-center gap-1.5 rounded-full border border-rose-300/20 bg-rose-400/10 px-3 py-1 text-rose-300"
              >
                <span class="font-mono text-sm leading-none">-</span>
                {translate($locale, "commit.removedLines", {
                  count: commitMeta.deletions,
                })}
              </span>
            </div>
          {/if}

          <div class="flex flex-wrap justify-end gap-1.5">
            {#if commit.isSafePushTarget}
              <span
                class="rounded-full border border-emerald-300/20 bg-emerald-400/10 px-2.5 py-0.5 text-[11px] font-semibold text-emerald-300"
              >
                {translate($locale, "history.safeStepPush")}
              </span>
            {/if}
            <span
              class="rounded-full border border-sky-300/20 bg-sky-400/10 px-2.5 py-0.5 text-[11px] font-semibold text-sky-200"
            >
              {translate($locale, "commit.behindBadge", {
                count: branchStatus?.behindCount ?? 0,
              })}
            </span>
          </div>
        </div>

        {#if commitBody}
          <section
            class="mt-3 rounded-xl border border-white/10 bg-[#111827]/65 px-3 py-2.5"
            aria-label={translate($locale, "commit.messageTitle")}
          >
            <div
              class="mb-1.5 text-[11px] font-semibold uppercase tracking-[0.14em] text-slate-500"
            >
              {translate($locale, "commit.messageTitle")}
            </div>
            <div
              class="max-h-24 overflow-y-auto whitespace-pre-wrap break-words font-mono text-[12px] leading-5 text-slate-300"
            >
              {commitBody}
            </div>
          </section>
        {/if}
      </div>
    {:else}
      <div
        class="rounded-xl border border-dashed border-white/10 bg-[#18202d]/70 px-4 py-5 text-sm text-slate-400"
      >
        {translate($locale, "commit.selectPrompt")}
      </div>
    {/if}
  </div>

  <div
    class="grid min-h-0 flex-1"
    style={`grid-template-columns: minmax(${MIN_FILES_PANE_WIDTH}px, ${filesPaneWidth}px) ${RESIZE_DIVIDER_LINE_WIDTH}px minmax(${MIN_BRANCH_PANE_WIDTH}px,1fr);`}
  >
    <div class="flex min-h-0 flex-col bg-[#111827]">
      <div
        class="flex items-center justify-between gap-3 border-b border-white/10 bg-[#151e2b] px-4 py-3"
      >
        <div class="text-sm font-semibold text-slate-50">
          {translate($locale, "commit.changedFiles")}
        </div>
        <div
          class="rounded-full border border-white/10 bg-white/[0.06] px-2.5 py-0.5 text-xs font-semibold text-slate-300"
        >
          {files.length}
        </div>
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto bg-[#111827]">
        {#if loadingFiles}
          <div
            class="m-4 rounded-xl border border-white/10 bg-[#18202d]/80 px-4 py-5 text-sm text-slate-400"
          >
            {translate($locale, "commit.loadingFiles")}
          </div>
        {:else if files.length === 0}
          <div
            class="m-4 rounded-xl border border-dashed border-white/10 bg-[#18202d]/70 px-4 py-8 text-center text-sm text-slate-500"
          >
            {translate($locale, "commit.noFileChanges")}
          </div>
        {:else}
          <div class="space-y-2 p-3">
            {#each files as file (file.path + file.status)}
              <div
                class={`relative flex items-center gap-2 overflow-hidden rounded-xl border px-3 py-2.5 transition ${
                  selectedFilePath === file.path
                    ? "border-sky-300/25 bg-sky-400/15 shadow-[0_14px_30px_rgba(59,130,246,0.14)]"
                    : "border-white/10 bg-[#18202d]/70 hover:border-sky-300/20 hover:bg-white/[0.06]"
                }`}
              >
                {#if selectedFilePath === file.path}
                  <span
                    class="absolute bottom-2 left-0 top-2 w-1 rounded-r-full bg-sky-300"
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
                      class="min-w-0 flex-1 truncate text-[13px] leading-5 text-slate-200"
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
                  class="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg border border-transparent bg-white/[0.04] text-slate-300 transition hover:border-sky-300/30 hover:bg-sky-400/10 hover:text-slate-100 focus:outline-none focus:ring-2 focus:ring-sky-400/35"
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
