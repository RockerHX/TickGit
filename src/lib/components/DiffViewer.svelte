<script lang="ts" context="module">
  import type { FileTypeIconFile } from "$lib/components/FileTypeIcon.svelte";

  export type DiffViewerSelectedFile = FileTypeIconFile & {
    displayPath?: string;
  };
</script>

<script lang="ts">
  import { onDestroy } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import { createEventDispatcher } from "svelte";
  import FileTypeIcon from "$lib/components/FileTypeIcon.svelte";
  import {
    buildHunkCopyText,
    getDiffViewerState,
    getSplitDiffRowsForMode,
    hasPreviewableImageDiff,
    parseUnifiedDiff,
    type DiffLine,
    type ParsedTextDiff,
    type SplitDiffRow,
  } from "$lib/tickgit/diff";
  import { writeClipboardText } from "$lib/tickgit/clipboard";
  import { highlightDiffContent } from "$lib/tickgit/diff-highlight";
  import type { CommitFileDiffResult } from "$lib/types";

  export let title = "Diff";
  export let selectedFilePath: string | null = null;
  export let selectedFile: DiffViewerSelectedFile | null = null;
  export let diffResult: CommitFileDiffResult = {
    text: "",
    isBinary: false,
    isImage: false,
    isTooLarge: false,
    truncated: false,
    byteCount: 0,
    lineCount: 0,
    oldImageDataUrl: null,
    newImageDataUrl: null,
  };
  export let loadingDiff = false;
  export let mode: "unified" | "split" = "unified";
  export let hideWhitespaceInDiff = false;

  const dispatch = createEventDispatcher<{
    modeChange: { mode: "unified" | "split" };
    hideWhitespaceChange: { value: boolean };
  }>();

  const emptyStateClasses =
    "m-4 rounded-xl border border-dashed border-white/10 bg-[#18202d]/70 px-4 py-10 text-center text-sm text-slate-500";

  let openControl: "mode" | "settings" | "more" | null = null;
  let parsedDiff: ParsedTextDiff = parseUnifiedDiff("");
  let splitRows: SplitDiffRow[] = [];
  let copiedHunkIndex: number | null = null;
  let hunkCopyResetTimer: ReturnType<typeof setTimeout> | null = null;

  // Unified / Split 共用同一份解析结果，避免两套渲染路径各自维护 diff 语义。
  $: diffText = diffResult.text;
  $: parsedDiff = parseUnifiedDiff(diffText);
  $: splitRows = getSplitDiffRowsForMode(parsedDiff, mode);
  $: displayFilePath =
    selectedFile?.displayPath ??
    selectedFilePath ??
    title ??
    translate($locale, "diff.title");
  $: fileIcon =
    selectedFile ??
    (selectedFilePath
      ? ({
          path: selectedFilePath,
        } satisfies DiffViewerSelectedFile)
      : null);
  $: viewerState = getDiffViewerState({
    selectedFilePath,
    loadingDiff,
    diffText,
    isBinary: diffResult.isBinary,
    isImage: diffResult.isImage,
    isTooLarge: diffResult.isTooLarge,
    hideWhitespaceInDiff,
    parsedDiff,
  });

  function lineToneClasses(type: DiffLine["type"]) {
    switch (type) {
      case "add":
        return "bg-emerald-500/[0.08] text-emerald-100";
      case "delete":
        return "bg-rose-500/[0.08] text-rose-100";
      default:
        return "text-slate-300";
    }
  }

  function lineNumberToneClasses(type: DiffLine["type"]) {
    switch (type) {
      case "add":
        return "bg-emerald-500/[0.12] text-emerald-300";
      case "delete":
        return "bg-rose-500/[0.12] text-rose-300";
      default:
        return "bg-[#0f1724] text-slate-500";
    }
  }

  function formatLineNumber(value: number | null) {
    return value === null ? "" : String(value);
  }

  function toggleControl(control: "mode" | "settings" | "more") {
    openControl = openControl === control ? null : control;
  }

  function setMode(nextMode: "unified" | "split") {
    openControl = null;
    dispatch("modeChange", { mode: nextMode });
  }

  function setHideWhitespace(value: boolean) {
    dispatch("hideWhitespaceChange", { value });
  }

  function closeOptions() {
    openControl = null;
  }

  function formatDiffSize(value: number) {
    if (value >= 1024 * 1024) {
      return `${(value / 1024 / 1024).toFixed(1)} MiB`;
    }

    if (value >= 1024) {
      return `${(value / 1024).toFixed(1)} KiB`;
    }

    return `${value} B`;
  }

  function highlightedLineContent(content: string) {
    return highlightDiffContent(content || " ", selectedFilePath);
  }

  function imagePanelLabel(kind: "old" | "new") {
    return kind === "old"
      ? translate($locale, "diff.imageBefore")
      : translate($locale, "diff.imageAfter");
  }

  function copyHunkLabel(hunkIndex: number) {
    return copiedHunkIndex === hunkIndex
      ? translate($locale, "diff.copiedHunk")
      : translate($locale, "diff.copyHunk");
  }

  async function copyHunk(hunkIndex: number) {
    const hunk = parsedDiff.hunks[hunkIndex];

    if (!hunk) {
      return;
    }

    try {
      await writeClipboardText(buildHunkCopyText(hunk));
      copiedHunkIndex = hunkIndex;

      if (hunkCopyResetTimer) {
        clearTimeout(hunkCopyResetTimer);
      }

      hunkCopyResetTimer = setTimeout(() => {
        copiedHunkIndex = null;
      }, 1600);
    } catch (error) {
      console.error(translate($locale, "diff.copyHunkFailedLog"), error);
      copiedHunkIndex = null;
    }
  }

  onDestroy(() => {
    if (hunkCopyResetTimer) {
      clearTimeout(hunkCopyResetTimer);
    }
  });
</script>

{#if openControl}
  <button
    class="fixed inset-0 z-20 cursor-default bg-transparent"
    aria-label={translate($locale, "diff.options")}
    on:click={closeOptions}
  ></button>
{/if}

<div class="flex min-h-0 flex-1 flex-col bg-[#111827]">
  <div
    class="relative flex items-center justify-between gap-3 border-b border-white/10 bg-[#151e2b] px-4 py-3 text-sm"
  >
    <div class="flex min-w-0 flex-1 items-center gap-3">
      {#if fileIcon}
        <FileTypeIcon file={fileIcon} />
      {/if}
      <div
        class="min-w-0 truncate font-semibold text-[#f0f6fc]"
        title={displayFilePath}
      >
        {displayFilePath}
      </div>
    </div>
    <div class="relative flex shrink-0 items-center gap-1.5">
      <button
        type="button"
        class="inline-flex h-8 items-center gap-1.5 rounded-full border border-sky-300/20 bg-sky-400/10 px-3 text-xs font-semibold text-sky-100 transition hover:border-sky-300/45 hover:bg-sky-400/15"
        aria-label={translate($locale, "diff.display")}
        aria-expanded={openControl === "mode"}
        on:click={() => toggleControl("mode")}
      >
        <span>
          {mode === "split"
            ? translate($locale, "diff.mode.split")
            : translate($locale, "diff.mode.unified")}
        </span>
        <svg
          viewBox="0 0 16 16"
          class="h-3 w-3 fill-current text-sky-200"
          aria-hidden="true"
        >
          <path
            d="M12.78 5.97a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06 0L3.22 7.03a.75.75 0 1 1 1.06-1.06L8 9.69l3.72-3.72a.75.75 0 0 1 1.06 0Z"
          ></path>
        </svg>
      </button>

      <button
        type="button"
        class={`inline-flex h-8 w-8 items-center justify-center rounded-lg border transition ${
          openControl === "settings"
            ? "border-sky-300/45 bg-sky-400/15 text-sky-100"
            : "border-white/10 bg-white/[0.04] text-slate-300 hover:border-sky-300/30 hover:bg-sky-400/10 hover:text-slate-100"
        }`}
        aria-label={translate($locale, "diff.options")}
        aria-expanded={openControl === "settings"}
        on:click={() => toggleControl("settings")}
      >
        <svg
          viewBox="0 0 16 16"
          class="h-4 w-4 fill-current"
          aria-hidden="true"
        >
          <path
            d="M6.5.75a.75.75 0 0 1 .75.75v.69c.367.07.718.193 1.047.36l.488-.488a.75.75 0 0 1 1.06 0l1.09 1.09a.75.75 0 0 1 0 1.06l-.488.488c.167.329.29.68.36 1.047h.69a.75.75 0 0 1 .75.75v1.54a.75.75 0 0 1-.75.75h-.69a4.457 4.457 0 0 1-.36 1.047l.488.488a.75.75 0 0 1 0 1.06l-1.09 1.09a.75.75 0 0 1-1.06 0l-.488-.488a4.457 4.457 0 0 1-1.047.36v.69a.75.75 0 0 1-.75.75H4.96a.75.75 0 0 1-.75-.75v-.69a4.457 4.457 0 0 1-1.047-.36l-.488.488a.75.75 0 0 1-1.06 0l-1.09-1.09a.75.75 0 0 1 0-1.06l.488-.488a4.457 4.457 0 0 1-.36-1.047h-.69a.75.75 0 0 1-.75-.75V6.5a.75.75 0 0 1 .75-.75h.69c.07-.367.193-.718.36-1.047l-.488-.488a.75.75 0 0 1 0-1.06l1.09-1.09a.75.75 0 0 1 1.06 0l.488.488c.329-.167.68-.29 1.047-.36V1.5a.75.75 0 0 1 .75-.75Zm-.77 4.5a2.75 2.75 0 1 0 0 5.5 2.75 2.75 0 0 0 0-5.5Z"
          ></path>
        </svg>
      </button>

      <button
        type="button"
        class={`inline-flex h-8 w-8 items-center justify-center rounded-lg border transition ${
          openControl === "more"
            ? "border-sky-300/45 bg-sky-400/15 text-sky-100"
            : "border-white/10 bg-white/[0.04] text-slate-300 hover:border-sky-300/30 hover:bg-sky-400/10 hover:text-slate-100"
        }`}
        aria-label={translate($locale, "diff.options")}
        aria-expanded={openControl === "more"}
        on:click={() => toggleControl("more")}
      >
        <svg
          viewBox="0 0 16 16"
          class="h-4 w-4 fill-current"
          aria-hidden="true"
        >
          <path
            d="M2 8a1.5 1.5 0 1 1 3 0 1.5 1.5 0 0 1-3 0Zm4.5 0a1.5 1.5 0 1 1 3 0 1.5 1.5 0 0 1-3 0ZM11 8a1.5 1.5 0 1 1 3 0 1.5 1.5 0 0 1-3 0Z"
          ></path>
        </svg>
      </button>

      {#if openControl}
        <div
          class="absolute right-0 top-[calc(100%+8px)] z-30 w-[240px] overflow-hidden rounded-xl border border-white/10 bg-[#151e2b] p-3 shadow-2xl shadow-black/35"
        >
          <div
            class="text-xs font-semibold uppercase tracking-[0.16em] text-slate-400"
          >
            {openControl === "mode"
              ? translate($locale, "diff.display")
              : translate($locale, "diff.options")}
          </div>

          {#if openControl === "mode"}
            <div class="mt-3 grid grid-cols-2 gap-2">
              <button
                type="button"
                class={`rounded-md border px-3 py-2 text-xs font-medium transition ${
                  mode === "unified"
                    ? "border-[#539bf5] bg-[#347dff]/18 text-[#f0f6fc]"
                    : "border-[#444c56] bg-[#373e47] text-slate-300 hover:border-[#539bf5]/40"
                }`}
                on:click={() => setMode("unified")}
              >
                Unified
              </button>
              <button
                type="button"
                class={`rounded-md border px-3 py-2 text-xs font-medium transition ${
                  mode === "split"
                    ? "border-[#539bf5] bg-[#347dff]/18 text-[#f0f6fc]"
                    : "border-[#444c56] bg-[#373e47] text-slate-300 hover:border-[#539bf5]/40"
                }`}
                on:click={() => setMode("split")}
              >
                Split
              </button>
            </div>
          {:else if openControl === "settings"}
            <label
              class="mt-3 flex cursor-pointer items-start gap-3 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2.5 transition hover:border-sky-300/35"
            >
              <input
                type="checkbox"
                class="mt-0.5 h-4 w-4 rounded border-[#6e7681] bg-[#24292f] text-[#347dff]"
                checked={hideWhitespaceInDiff}
                on:change={(event) =>
                  setHideWhitespace(event.currentTarget.checked)}
              />
              <div class="min-w-0">
                <div class="text-xs font-medium text-[#f0f6fc]">
                  {translate($locale, "diff.hideWhitespace")}
                </div>
                <div class="mt-1 text-[11px] leading-4 text-slate-400">
                  {translate($locale, "diff.hideWhitespaceDescription")}
                </div>
              </div>
            </label>
          {:else}
            <div class="mt-3 space-y-2 text-xs text-slate-300">
              {#if diffResult.byteCount > 0}
                <div class="rounded-lg bg-white/[0.04] px-3 py-2">
                  {translate($locale, "diff.patchSize", {
                    size: formatDiffSize(diffResult.byteCount),
                  })}
                </div>
              {/if}
              <div class="rounded-lg bg-white/[0.04] px-3 py-2">
                {translate($locale, "diff.changedLines", {
                  count: diffResult.lineCount,
                })}
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto overflow-x-hidden bg-[#111827]">
    {#if viewerState === "loading"}
      <div class={emptyStateClasses}>
        {translate($locale, "diff.loading")}
      </div>
    {:else if viewerState === "no-file"}
      <div class={emptyStateClasses}>
        {translate($locale, "diff.noFile")}
      </div>
    {:else if viewerState === "image"}
      {#if hasPreviewableImageDiff(diffResult)}
        <div class="grid gap-4 p-4 xl:grid-cols-2">
          {#each [{ kind: "old", url: diffResult.oldImageDataUrl }, { kind: "new", url: diffResult.newImageDataUrl }] as panel}
            <div
              class="min-w-0 overflow-hidden rounded-xl border border-white/10 bg-[#18202d]/80"
            >
              <div
                class="border-b border-white/10 bg-white/[0.03] px-3 py-2 text-xs font-semibold uppercase tracking-[0.16em] text-slate-400"
              >
                {imagePanelLabel(panel.kind as "old" | "new")}
              </div>
              {#if panel.url}
                <div
                  class="flex min-h-64 items-center justify-center bg-[#0f1724] p-4"
                >
                  <img
                    class="max-h-[52vh] max-w-full rounded-lg border border-white/10 bg-white/5 object-contain"
                    src={panel.url}
                    alt={`${imagePanelLabel(panel.kind as "old" | "new")} ${selectedFilePath ?? "image"}`}
                  />
                </div>
              {:else}
                <div
                  class="flex min-h-64 items-center justify-center px-4 text-center text-sm text-slate-500"
                >
                  {panel.kind === "old"
                    ? translate($locale, "diff.noOldImage")
                    : translate($locale, "diff.noNewImage")}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {:else}
        <div class={emptyStateClasses}>
          {translate($locale, "diff.imageUnavailable")}
        </div>
      {/if}
    {:else if viewerState === "binary"}
      <div class={emptyStateClasses}>
        {translate($locale, "diff.binaryUnavailable")}
      </div>
    {:else if viewerState === "too-large"}
      <div class={emptyStateClasses}>
        <div>{translate($locale, "diff.largeSkipped")}</div>
        <div class="mt-2 text-xs text-slate-500">
          {#if diffResult.byteCount > 0}
            {translate($locale, "diff.patchSize", {
              size: formatDiffSize(diffResult.byteCount),
            })} ·
          {/if}
          {translate($locale, "diff.changedLines", {
            count: diffResult.lineCount,
          })}
        </div>
      </div>
    {:else if viewerState === "only-whitespace"}
      <div class={emptyStateClasses}>
        {translate($locale, "diff.onlyWhitespace")}
      </div>
    {:else if viewerState === "no-content"}
      <div class={emptyStateClasses}>
        {translate($locale, "diff.noContent")}
      </div>
    {:else if viewerState === "parse-error"}
      <div class={emptyStateClasses}>
        {translate($locale, "diff.parseError")}
      </div>
    {:else if mode === "split"}
      <div>
        {#each splitRows as row, index (row.kind === "hunk" ? `${row.header}-${index}` : `${row.left?.originalLineNumber ?? "x"}-${row.right?.originalLineNumber ?? "y"}-${index}`)}
          {#if row.kind === "hunk"}
            <div
              class="flex items-center justify-between gap-3 border-y border-slate-500/10 bg-[#172033] px-4 py-2 font-mono text-[12px] text-sky-200"
            >
              <span class="min-w-0 truncate">{row.header}</span>
              <button
                type="button"
                class="inline-flex h-7 shrink-0 items-center gap-1.5 rounded-full border border-sky-300/25 bg-sky-300/10 px-2.5 text-[11px] font-medium text-sky-100 transition hover:border-sky-200/45 hover:bg-sky-300/18"
                title={copyHunkLabel(row.hunkIndex)}
                aria-label={copyHunkLabel(row.hunkIndex)}
                on:click={() => copyHunk(row.hunkIndex)}
              >
                {#if copiedHunkIndex === row.hunkIndex}
                  <svg
                    viewBox="0 0 16 16"
                    class="h-3.5 w-3.5 fill-current text-emerald-300"
                    aria-hidden="true"
                  >
                    <path
                      d="M13.78 4.97a.75.75 0 0 1 0 1.06L7.53 12.28a.75.75 0 0 1-1.06 0L2.22 8.03a.75.75 0 1 1 1.06-1.06L7 10.69l5.72-5.72a.75.75 0 0 1 1.06 0Z"
                    ></path>
                  </svg>
                  {translate($locale, "common.copied")}
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
                  {translate($locale, "diff.copyHunk")}
                {/if}
              </button>
            </div>
          {:else}
            <div
              class="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)] border-b border-slate-700/45"
            >
              {#each [row.left, row.right] as line, sideIndex}
                <div
                  class={`grid min-w-0 grid-cols-[4rem_4rem_minmax(0,1fr)] ${
                    sideIndex === 0 ? "border-r border-slate-700/60" : ""
                  } ${line ? lineToneClasses(line.type) : ""}`}
                >
                  <div
                    class={`border-r border-slate-700/55 px-2 py-0.5 text-right font-mono text-[11px] ${line ? lineNumberToneClasses(line.type) : "bg-[#0f1724] text-slate-600"}`}
                  >
                    {line ? formatLineNumber(line.oldLineNumber) : ""}
                  </div>
                  <div
                    class={`border-r border-slate-700/55 px-2 py-0.5 text-right font-mono text-[11px] ${line ? lineNumberToneClasses(line.type) : "bg-[#0f1724] text-slate-600"}`}
                  >
                    {line ? formatLineNumber(line.newLineNumber) : ""}
                  </div>
                  <div
                    class="overflow-x-hidden px-3 py-0.5 font-mono text-[12px] leading-6"
                  >
                    <span class="block whitespace-pre-wrap break-all">
                      {@html line ? highlightedLineContent(line.content) : " "}
                    </span>
                    {#if line?.noTrailingNewLine}
                      <div class="text-[11px] italic text-amber-200">
                        {translate($locale, "diff.noNewlineAtEnd")}
                      </div>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        {/each}
      </div>
    {:else}
      <div>
        {#each parsedDiff.hunks as hunk, hunkIndex}
          <div
            class="flex items-center justify-between gap-3 border-y border-slate-500/10 bg-[#172033] px-4 py-2 font-mono text-[12px] text-sky-200"
          >
            <span class="min-w-0 truncate">{hunk.header}</span>
            <button
              type="button"
              class="inline-flex h-7 shrink-0 items-center gap-1.5 rounded-full border border-sky-300/25 bg-sky-300/10 px-2.5 text-[11px] font-medium text-sky-100 transition hover:border-sky-200/45 hover:bg-sky-300/18"
              title={copyHunkLabel(hunkIndex)}
              aria-label={copyHunkLabel(hunkIndex)}
              on:click={() => copyHunk(hunkIndex)}
            >
              {#if copiedHunkIndex === hunkIndex}
                <svg
                  viewBox="0 0 16 16"
                  class="h-3.5 w-3.5 fill-current text-emerald-300"
                  aria-hidden="true"
                >
                  <path
                    d="M13.78 4.97a.75.75 0 0 1 0 1.06L7.53 12.28a.75.75 0 0 1-1.06 0L2.22 8.03a.75.75 0 1 1 1.06-1.06L7 10.69l5.72-5.72a.75.75 0 0 1 1.06 0Z"
                  ></path>
                </svg>
                {translate($locale, "common.copied")}
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
                {translate($locale, "diff.copyHunk")}
              {/if}
            </button>
          </div>
          {#each hunk.lines as line}
            <div
              class={`grid grid-cols-[4rem_4rem_minmax(0,1fr)] border-b border-slate-700/45 ${lineToneClasses(line.type)}`}
            >
              <div
                class={`border-r border-slate-700/55 px-2 py-0.5 text-right font-mono text-[11px] ${lineNumberToneClasses(line.type)}`}
              >
                {formatLineNumber(line.oldLineNumber)}
              </div>
              <div
                class={`border-r border-slate-700/55 px-2 py-0.5 text-right font-mono text-[11px] ${lineNumberToneClasses(line.type)}`}
              >
                {formatLineNumber(line.newLineNumber)}
              </div>
              <div
                class="overflow-x-hidden px-3 py-0.5 font-mono text-[12px] leading-6"
              >
                <span class="block whitespace-pre-wrap break-all">
                  {@html highlightedLineContent(line.content)}
                </span>
                {#if line.noTrailingNewLine}
                  <div class="text-[11px] italic text-amber-200">
                    {translate($locale, "diff.noNewlineAtEnd")}
                  </div>
                {/if}
              </div>
            </div>
          {/each}
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  :global(.hljs-keyword),
  :global(.hljs-selector-tag),
  :global(.hljs-built_in) {
    color: #ffb4d0;
  }

  :global(.hljs-string),
  :global(.hljs-attr),
  :global(.hljs-symbol) {
    color: #a5d6ff;
  }

  :global(.hljs-number),
  :global(.hljs-literal) {
    color: #79c0ff;
  }

  :global(.hljs-comment),
  :global(.hljs-quote) {
    color: #8b949e;
    font-style: italic;
  }

  :global(.hljs-title),
  :global(.hljs-name),
  :global(.hljs-section) {
    color: #d2a8ff;
  }
</style>
