<script lang="ts" context="module">
  import type { FileTypeIconFile } from "$lib/components/FileTypeIcon.svelte";

  export type DiffViewerSelectedFile = FileTypeIconFile & {
    displayPath?: string;
  };
</script>

<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import { createEventDispatcher } from "svelte";
  import FileTypeIcon from "$lib/components/FileTypeIcon.svelte";
  import {
    buildHunkCopyText,
    getDiffViewerState,
    hasPreviewableImageDiff,
    parseUnifiedDiff,
    type DiffLine,
  } from "$lib/tickgit/diff";
  import { writeClipboardText } from "$lib/tickgit/clipboard";
  import { highlightDiffContent } from "$lib/tickgit/diff-highlight";
  import type {
    DiffWorkerRequest,
    DiffWorkerResponse,
    HighlightedDiffLine,
    HighlightedParsedTextDiff,
    HighlightedSplitDiffRow,
  } from "$lib/tickgit/diff-worker";
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
    "m-3 rounded-xl border border-dashed border-tg-border-soft bg-tg-bg-card/70 px-3 py-8 text-center text-xs text-tg-text-muted";

  let openControl: "mode" | "settings" | "more" | null = null;
  let parsedDiff: HighlightedParsedTextDiff = {
    ...parseUnifiedDiff(""),
    hunks: [],
  };
  let splitRows: HighlightedSplitDiffRow[] = [];
  let copiedHunkIndex: number | null = null;
  let hunkCopyResetTimer: ReturnType<typeof setTimeout> | null = null;
  let diffWorker: Worker | null = null;
  let diffWorkerRequestId = 0;

  // Unified / Split 共用同一份解析结果，避免两套渲染路径各自维护 diff 语义。
  $: diffText = diffResult.text;
  $: scheduleDiffProcessing(diffText, selectedFilePath, mode);
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

  function highlightLine(
    line: DiffLine,
    filePath: string | null,
  ): HighlightedDiffLine {
    return {
      ...line,
      html: highlightDiffContent(line.content || " ", filePath),
    };
  }

  function processDiffOnMainThread(
    nextDiffText: string,
    filePath: string | null,
    nextMode: "unified" | "split",
  ) {
    const parsed = parseUnifiedDiff(nextDiffText);
    parsedDiff = {
      ...parsed,
      hunks: parsed.hunks.map((hunk) => ({
        ...hunk,
        lines: hunk.lines.map((line) => highlightLine(line, filePath)),
      })),
    };
    splitRows =
      nextMode === "split"
        ? parsedDiff.hunks.flatMap((hunk, hunkIndex) => [
            { kind: "hunk" as const, header: hunk.header, hunkIndex },
            ...hunk.lines.map((line) => ({
              kind: "line" as const,
              left: line.type === "add" ? null : line,
              right: line.type === "delete" ? null : line,
            })),
          ])
        : [];
  }

  function scheduleDiffProcessing(
    nextDiffText: string,
    filePath: string | null,
    nextMode: "unified" | "split",
  ) {
    const requestId = ++diffWorkerRequestId;
    if (!diffWorker) {
      processDiffOnMainThread(nextDiffText, filePath, nextMode);
      return;
    }

    diffWorker.postMessage({
      id: requestId,
      diffText: nextDiffText,
      filePath,
      mode: nextMode,
    } satisfies DiffWorkerRequest);
  }

  function lineToneClasses(type: DiffLine["type"]) {
    switch (type) {
      case "add":
        return "bg-emerald-500/[0.08] text-emerald-100";
      case "delete":
        return "bg-rose-500/[0.08] text-rose-100";
      default:
        return "text-tg-text-secondary";
    }
  }

  function lineNumberToneClasses(type: DiffLine["type"]) {
    switch (type) {
      case "add":
        return "bg-emerald-500/[0.12] text-emerald-300";
      case "delete":
        return "bg-rose-500/[0.12] text-rose-300";
      default:
        return "bg-tg-bg-app text-tg-text-muted";
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

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape" || !openControl) {
      return;
    }

    closeOptions();
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

  onMount(() => {
    diffWorker = new Worker(
      new URL("$lib/tickgit/diff-worker.ts", import.meta.url),
      { type: "module" },
    );
    diffWorker.onmessage = (event: MessageEvent<DiffWorkerResponse>) => {
      if (event.data.id !== diffWorkerRequestId) {
        return;
      }

      parsedDiff = event.data.parsedDiff;
      splitRows = event.data.splitRows;
    };
    scheduleDiffProcessing(diffText, selectedFilePath, mode);
  });

  onDestroy(() => {
    if (hunkCopyResetTimer) {
      clearTimeout(hunkCopyResetTimer);
    }
    diffWorker?.terminate();
  });
</script>

<svelte:window on:keydown={handleWindowKeydown} />

{#if openControl}
  <button
    class="fixed inset-0 z-20 cursor-default bg-transparent"
    aria-label={translate($locale, "diff.options")}
    on:click={closeOptions}
  ></button>
{/if}

<div class="flex min-h-0 flex-1 flex-col bg-tg-bg-panel">
  <div
    class="relative flex items-center justify-between gap-2 border-b border-tg-border-soft bg-tg-bg-card px-2.5 py-1.5 text-[12px]"
  >
    <div class="flex min-w-0 flex-1 items-center gap-2">
      {#if fileIcon}
        <FileTypeIcon file={fileIcon} />
      {/if}
      <div
        class="min-w-0 truncate font-semibold text-tg-text-primary"
        title={displayFilePath}
      >
        {displayFilePath}
      </div>
    </div>
    <div class="relative flex shrink-0 items-center gap-1">
      <button
        type="button"
        class="tg-focus-ring inline-flex h-6 items-center gap-1 rounded-lg border border-tg-blue-soft/20 bg-tg-blue-soft/10 px-2 text-[10px] font-semibold text-sky-100 transition hover:border-tg-blue-soft/45 hover:bg-tg-blue-soft/15"
        aria-label={translate($locale, "diff.display")}
        aria-controls="diff-control-popover"
        aria-expanded={openControl === "mode"}
        aria-haspopup="menu"
        aria-pressed={openControl === "mode"}
        on:click={() => toggleControl("mode")}
      >
        <span>
          {mode === "split"
            ? translate($locale, "diff.mode.split")
            : translate($locale, "diff.mode.unified")}
        </span>
        <svg
          viewBox="0 0 16 16"
          class="h-2.5 w-2.5 fill-current text-sky-200"
          aria-hidden="true"
        >
          <path
            d="M12.78 5.97a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06 0L3.22 7.03a.75.75 0 1 1 1.06-1.06L8 9.69l3.72-3.72a.75.75 0 0 1 1.06 0Z"
          ></path>
        </svg>
      </button>

      <button
        type="button"
        class={`inline-flex h-6 w-6 items-center justify-center rounded-lg border transition ${
          openControl === "settings"
            ? "border-tg-blue-soft/45 bg-tg-blue-soft/15 text-sky-100"
            : "border-tg-border-soft bg-white/[0.04] text-tg-text-secondary hover:border-tg-blue-soft/30 hover:bg-tg-blue-soft/10 hover:text-tg-text-primary"
        }`}
        aria-label={translate($locale, "diff.options")}
        aria-controls="diff-control-popover"
        aria-expanded={openControl === "settings"}
        aria-haspopup="menu"
        aria-pressed={openControl === "settings"}
        on:click={() => toggleControl("settings")}
      >
        <svg
          viewBox="0 0 16 16"
          class="h-3 w-3 fill-current"
          aria-hidden="true"
        >
          <path
            d="M6.5.75a.75.75 0 0 1 .75.75v.69c.367.07.718.193 1.047.36l.488-.488a.75.75 0 0 1 1.06 0l1.09 1.09a.75.75 0 0 1 0 1.06l-.488.488c.167.329.29.68.36 1.047h.69a.75.75 0 0 1 .75.75v1.54a.75.75 0 0 1-.75.75h-.69a4.457 4.457 0 0 1-.36 1.047l.488.488a.75.75 0 0 1 0 1.06l-1.09 1.09a.75.75 0 0 1-1.06 0l-.488-.488a4.457 4.457 0 0 1-1.047.36v.69a.75.75 0 0 1-.75.75H4.96a.75.75 0 0 1-.75-.75v-.69a4.457 4.457 0 0 1-1.047-.36l-.488.488a.75.75 0 0 1-1.06 0l-1.09-1.09a.75.75 0 0 1 0-1.06l.488-.488a4.457 4.457 0 0 1-.36-1.047h-.69a.75.75 0 0 1-.75-.75V6.5a.75.75 0 0 1 .75-.75h.69c.07-.367.193-.718.36-1.047l-.488-.488a.75.75 0 0 1 0-1.06l1.09-1.09a.75.75 0 0 1 1.06 0l.488.488c.329-.167.68-.29 1.047-.36V1.5a.75.75 0 0 1 .75-.75Zm-.77 4.5a2.75 2.75 0 1 0 0 5.5 2.75 2.75 0 0 0 0-5.5Z"
          ></path>
        </svg>
      </button>

      <button
        type="button"
        class={`inline-flex h-6 w-6 items-center justify-center rounded-lg border transition ${
          openControl === "more"
            ? "border-tg-blue-soft/45 bg-tg-blue-soft/15 text-sky-100"
            : "border-tg-border-soft bg-white/[0.04] text-tg-text-secondary hover:border-tg-blue-soft/30 hover:bg-tg-blue-soft/10 hover:text-tg-text-primary"
        }`}
        aria-label={translate($locale, "diff.options")}
        aria-controls="diff-control-popover"
        aria-expanded={openControl === "more"}
        aria-haspopup="menu"
        aria-pressed={openControl === "more"}
        on:click={() => toggleControl("more")}
      >
        <svg
          viewBox="0 0 16 16"
          class="h-3 w-3 fill-current"
          aria-hidden="true"
        >
          <path
            d="M2 8a1.5 1.5 0 1 1 3 0 1.5 1.5 0 0 1-3 0Zm4.5 0a1.5 1.5 0 1 1 3 0 1.5 1.5 0 0 1-3 0ZM11 8a1.5 1.5 0 1 1 3 0 1.5 1.5 0 0 1-3 0Z"
          ></path>
        </svg>
      </button>

      {#if openControl}
        <div
          id="diff-control-popover"
          class="tg-panel absolute right-0 top-[calc(100%+6px)] z-30 w-[210px] overflow-hidden rounded-xl p-2 shadow-2xl shadow-black/35"
          role="group"
          aria-label={openControl === "mode"
            ? translate($locale, "diff.display")
            : translate($locale, "diff.options")}
        >
          <div
            class="text-[10px] font-semibold uppercase tracking-[0.16em] text-tg-text-secondary/80"
          >
            {openControl === "mode"
              ? translate($locale, "diff.display")
              : translate($locale, "diff.options")}
          </div>

          {#if openControl === "mode"}
            <div class="mt-2 grid grid-cols-2 gap-1">
              <button
                type="button"
                class={`rounded-md border px-2 py-1 text-[10px] font-medium transition ${
                  mode === "unified"
                    ? "border-tg-blue-soft bg-tg-blue/18 text-tg-text-primary"
                    : "border-tg-border-strong bg-tg-bg-card text-tg-text-secondary hover:border-tg-blue-soft/40"
                }`}
                aria-pressed={mode === "unified"}
                on:click={() => setMode("unified")}
              >
                {translate($locale, "diff.mode.unified")}
              </button>
              <button
                type="button"
                class={`rounded-md border px-2 py-1 text-[10px] font-medium transition ${
                  mode === "split"
                    ? "border-tg-blue-soft bg-tg-blue/18 text-tg-text-primary"
                    : "border-tg-border-strong bg-tg-bg-card text-tg-text-secondary hover:border-tg-blue-soft/40"
                }`}
                aria-pressed={mode === "split"}
                on:click={() => setMode("split")}
              >
                {translate($locale, "diff.mode.split")}
              </button>
            </div>
          {:else if openControl === "settings"}
            <label
              class="mt-2 flex cursor-pointer items-start gap-2 rounded-lg border border-tg-border-soft bg-white/[0.04] px-2 py-1.5 transition hover:border-tg-blue-soft/35"
            >
              <input
                type="checkbox"
                class="mt-0.5 h-3 w-3 rounded border-tg-border-strong bg-tg-bg-card text-tg-blue"
                checked={hideWhitespaceInDiff}
                on:change={(event) =>
                  setHideWhitespace(event.currentTarget.checked)}
              />
              <div class="min-w-0">
                <div class="text-[10px] font-medium text-tg-text-primary">
                  {translate($locale, "diff.hideWhitespace")}
                </div>
                <div
                  class="mt-0.5 text-[9px] leading-4 text-tg-text-secondary/80"
                >
                  {translate($locale, "diff.hideWhitespaceDescription")}
                </div>
              </div>
            </label>
          {:else}
            <div class="mt-2 space-y-1 text-[10px] text-tg-text-secondary">
              {#if diffResult.byteCount > 0}
                <div class="rounded-lg bg-white/[0.04] px-2 py-1">
                  {translate($locale, "diff.patchSize", {
                    size: formatDiffSize(diffResult.byteCount),
                  })}
                </div>
              {/if}
              <div class="rounded-lg bg-white/[0.04] px-2 py-1">
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

  <div class="min-h-0 flex-1 overflow-y-auto overflow-x-hidden bg-tg-bg-panel">
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
            <div class="tg-card min-w-0 overflow-hidden">
              <div
                class="border-b border-tg-border-soft bg-white/[0.03] px-3 py-2 text-xs font-semibold uppercase tracking-[0.16em] text-tg-text-secondary/80"
              >
                {imagePanelLabel(panel.kind as "old" | "new")}
              </div>
              {#if panel.url}
                <div
                  class="flex min-h-64 items-center justify-center bg-tg-bg-app p-4"
                >
                  <img
                    class="max-h-[52vh] max-w-full rounded-lg border border-white/10 bg-white/5 object-contain"
                    src={panel.url}
                    alt={`${imagePanelLabel(panel.kind as "old" | "new")} ${selectedFilePath ?? "image"}`}
                  />
                </div>
              {:else}
                <div
                  class="flex min-h-64 items-center justify-center px-4 text-center text-sm text-tg-text-muted"
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
        <div class="mt-2 text-xs text-tg-text-muted">
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
              class="flex items-center justify-between gap-2 border-y border-tg-border-soft bg-tg-bg-elevated px-2.5 py-1 font-mono text-[10px] text-sky-100"
            >
              <span class="min-w-0 truncate">{row.header}</span>
              <button
                type="button"
                class="tg-focus-ring inline-flex h-6 shrink-0 items-center gap-1 rounded-full border border-tg-blue-soft/25 bg-tg-blue-soft/10 px-1.5 text-[9px] font-medium text-sky-100 transition hover:border-tg-blue-soft/45 hover:bg-tg-blue-soft/18"
                title={copyHunkLabel(row.hunkIndex)}
                aria-label={copyHunkLabel(row.hunkIndex)}
                on:click={() => copyHunk(row.hunkIndex)}
              >
                {#if copiedHunkIndex === row.hunkIndex}
                  <svg
                    viewBox="0 0 16 16"
                    class="h-2.5 w-2.5 fill-current text-emerald-300"
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
                    class="h-2.5 w-2.5 fill-current"
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
                  class={`grid min-w-0 grid-cols-[3rem_3rem_minmax(0,1fr)] ${
                    sideIndex === 0 ? "border-r border-slate-700/60" : ""
                  } ${line ? lineToneClasses(line.type) : ""}`}
                >
                  <div
                    class={`border-r border-slate-700/55 px-1.5 py-0 text-right font-mono text-[10px] leading-5 ${line ? lineNumberToneClasses(line.type) : "bg-tg-bg-app text-tg-text-muted/70"}`}
                  >
                    {line ? formatLineNumber(line.oldLineNumber) : ""}
                  </div>
                  <div
                    class={`border-r border-slate-700/55 px-1.5 py-0 text-right font-mono text-[10px] leading-5 ${line ? lineNumberToneClasses(line.type) : "bg-tg-bg-app text-tg-text-muted/70"}`}
                  >
                    {line ? formatLineNumber(line.newLineNumber) : ""}
                  </div>
                  <div
                    class="overflow-x-hidden px-2 py-0 font-mono text-[11px] leading-5"
                  >
                    <span class="block whitespace-pre-wrap break-all">
                      {@html line ? line.html : " "}
                    </span>
                    {#if line?.noTrailingNewLine}
                      <div class="text-[10px] italic text-amber-200">
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
            class="flex items-center justify-between gap-2 border-y border-tg-border-soft bg-tg-bg-elevated px-2.5 py-1 font-mono text-[10px] text-sky-100"
          >
            <span class="min-w-0 truncate">{hunk.header}</span>
            <button
              type="button"
              class="tg-focus-ring inline-flex h-6 shrink-0 items-center gap-1 rounded-full border border-tg-blue-soft/25 bg-tg-blue-soft/10 px-1.5 text-[9px] font-medium text-sky-100 transition hover:border-tg-blue-soft/45 hover:bg-tg-blue-soft/18"
              title={copyHunkLabel(hunkIndex)}
              aria-label={copyHunkLabel(hunkIndex)}
              on:click={() => copyHunk(hunkIndex)}
            >
              {#if copiedHunkIndex === hunkIndex}
                <svg
                  viewBox="0 0 16 16"
                  class="h-2.5 w-2.5 fill-current text-emerald-300"
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
                  class="h-2.5 w-2.5 fill-current"
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
              class={`grid grid-cols-[3rem_3rem_minmax(0,1fr)] border-b border-slate-700/45 ${lineToneClasses(line.type)}`}
            >
              <div
                class={`border-r border-slate-700/55 px-1.5 py-0 text-right font-mono text-[10px] leading-5 ${lineNumberToneClasses(line.type)}`}
              >
                {formatLineNumber(line.oldLineNumber)}
              </div>
              <div
                class={`border-r border-slate-700/55 px-1.5 py-0 text-right font-mono text-[10px] leading-5 ${lineNumberToneClasses(line.type)}`}
              >
                {formatLineNumber(line.newLineNumber)}
              </div>
              <div
                class="overflow-x-hidden px-2 py-0 font-mono text-[11px] leading-5"
              >
                <span class="block whitespace-pre-wrap break-all">
                  {@html line.html}
                </span>
                {#if line.noTrailingNewLine}
                  <div class="text-[10px] italic text-amber-200">
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
