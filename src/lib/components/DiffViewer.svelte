<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    getDiffViewerState,
    getSplitDiffRowsForMode,
    parseUnifiedDiff,
    type DiffLine,
    type ParsedTextDiff,
    type SplitDiffRow,
  } from "$lib/tickgit/diff";
  import type { CommitFileDiffResult } from "$lib/types";

  export let title = "Diff";
  export let selectedFilePath: string | null = null;
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

  let optionsOpen = false;
  let parsedDiff: ParsedTextDiff = parseUnifiedDiff("");
  let splitRows: SplitDiffRow[] = [];

  // Unified / Split 共用同一份解析结果，避免两套渲染路径各自维护 diff 语义。
  $: diffText = diffResult.text;
  $: parsedDiff = parseUnifiedDiff(diffText);
  $: splitRows = getSplitDiffRowsForMode(parsedDiff, mode);
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
        return "bg-emerald-500/12 text-emerald-100";
      case "delete":
        return "bg-rose-500/12 text-rose-100";
      default:
        return "text-slate-300";
    }
  }

  function lineNumberToneClasses(type: DiffLine["type"]) {
    switch (type) {
      case "add":
        return "bg-emerald-500/18 text-emerald-300";
      case "delete":
        return "bg-rose-500/18 text-rose-300";
      default:
        return "text-slate-500";
    }
  }

  function formatLineNumber(value: number | null) {
    return value === null ? "" : String(value);
  }

  function toggleOptions() {
    optionsOpen = !optionsOpen;
  }

  function setMode(nextMode: "unified" | "split") {
    optionsOpen = false;
    dispatch("modeChange", { mode: nextMode });
  }

  function setHideWhitespace(value: boolean) {
    dispatch("hideWhitespaceChange", { value });
  }

  function closeOptions() {
    optionsOpen = false;
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
</script>

{#if optionsOpen}
  <button
    class="fixed inset-0 z-20 cursor-default bg-transparent"
    aria-label="Close diff options"
    on:click={closeOptions}
  ></button>
{/if}

<div class="flex min-h-0 flex-1 flex-col bg-[#2b3036]">
  <div
    class="relative flex items-center justify-between gap-3 border-b border-[#1f2328] bg-[#2d333b] px-4 py-3 text-sm"
  >
    <div class="truncate font-semibold text-[#f0f6fc]">
      {selectedFilePath ?? title}
    </div>
    <div class="flex items-center gap-2">
      <div
        class="hidden text-[11px] uppercase tracking-[0.18em] text-slate-500 sm:block"
      >
        {mode === "split" ? "Split" : "Unified"}
      </div>
      <button
        type="button"
        class="inline-flex h-8 items-center gap-1 rounded-md border border-[#444c56] bg-[#373e47] px-2.5 text-xs font-medium text-[#f0f6fc] transition hover:border-[#539bf5]/50 hover:bg-[#347dff]/15"
        aria-label="Diff options"
        aria-expanded={optionsOpen}
        on:click={toggleOptions}
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
        <svg
          viewBox="0 0 16 16"
          class="h-3 w-3 fill-current text-slate-300"
          aria-hidden="true"
        >
          <path
            d="M12.78 5.97a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06 0L3.22 7.03a.75.75 0 1 1 1.06-1.06L8 9.69l3.72-3.72a.75.75 0 0 1 1.06 0Z"
          ></path>
        </svg>
      </button>
      {#if optionsOpen}
        <div
          class="absolute right-4 top-[calc(100%-4px)] z-30 w-[240px] overflow-hidden rounded-md border border-[#444c56] bg-[#2d333b] p-3 shadow-lg shadow-black/35"
        >
          <div
            class="text-xs font-semibold uppercase tracking-[0.16em] text-slate-400"
          >
            Diff options
          </div>

          <div class="mt-3">
            <div class="mb-2 text-xs font-medium text-slate-300">
              Diff display
            </div>
            <div class="grid grid-cols-2 gap-2">
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
          </div>

          <label
            class="mt-4 flex cursor-pointer items-start gap-3 rounded-md border border-[#444c56] bg-[#373e47]/55 px-3 py-2.5 transition hover:border-[#539bf5]/35"
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
                Hide whitespace changes
              </div>
              <div class="mt-1 text-[11px] leading-4 text-slate-400">
                Reload the current file diff with Git whitespace filtering.
              </div>
            </div>
          </label>
        </div>
      {/if}
    </div>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto overflow-x-hidden bg-[#2b3036]">
    {#if viewerState === "loading"}
      <div class="px-4 py-4 text-sm text-slate-400">Loading diff…</div>
    {:else if viewerState === "no-file"}
      <div
        class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2d333b] px-4 py-10 text-center text-sm text-slate-500"
      >
        Select a changed file to inspect the diff
      </div>
    {:else if viewerState === "image"}
      <div
        class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2d333b] px-4 py-10 text-center text-sm text-slate-500"
      >
        图片文件暂不展示图片 Diff
      </div>
    {:else if viewerState === "binary"}
      <div
        class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2d333b] px-4 py-10 text-center text-sm text-slate-500"
      >
        二进制文件暂不展示文本 Diff
      </div>
    {:else if viewerState === "too-large"}
      <div
        class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2d333b] px-4 py-10 text-center text-sm text-slate-500"
      >
        <div>大型 Diff 已保护性跳过</div>
        <div class="mt-2 text-xs text-slate-500">
          {#if diffResult.byteCount > 0}
            Patch size: {formatDiffSize(diffResult.byteCount)} ·
          {/if}
          Changed lines: {diffResult.lineCount}
        </div>
      </div>
    {:else if viewerState === "only-whitespace"}
      <div
        class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2d333b] px-4 py-10 text-center text-sm text-slate-500"
      >
        Only whitespace changes were found for this file
      </div>
    {:else if viewerState === "no-content"}
      <div
        class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2d333b] px-4 py-10 text-center text-sm text-slate-500"
      >
        No diff content is available for this file
      </div>
    {:else if viewerState === "parse-error"}
      <div
        class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2d333b] px-4 py-10 text-center text-sm text-slate-500"
      >
        This diff could not be rendered in the structured viewer yet
      </div>
    {:else if mode === "split"}
      <div>
        {#each splitRows as row, index (row.kind === "hunk" ? `${row.header}-${index}` : `${row.left?.originalLineNumber ?? "x"}-${row.right?.originalLineNumber ?? "y"}-${index}`)}
          {#if row.kind === "hunk"}
            <div
              class="border-b border-[#373e47]/70 bg-sky-500/10 px-4 py-1.5 font-mono text-[12px] text-sky-200"
            >
              {row.header}
            </div>
          {:else}
            <div
              class="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)] border-b border-[#373e47]/70"
            >
              {#each [row.left, row.right] as line, sideIndex}
                <div
                  class={`grid min-w-0 grid-cols-[4rem_4rem_minmax(0,1fr)] ${
                    sideIndex === 0 ? "border-r border-[#373e47]/70" : ""
                  } ${line ? lineToneClasses(line.type) : ""}`}
                >
                  <div
                    class={`border-r border-[#373e47]/60 px-2 py-0.5 text-right font-mono text-[11px] ${line ? lineNumberToneClasses(line.type) : "text-slate-600"}`}
                  >
                    {line ? formatLineNumber(line.oldLineNumber) : ""}
                  </div>
                  <div
                    class={`border-r border-[#373e47]/60 px-2 py-0.5 text-right font-mono text-[11px] ${line ? lineNumberToneClasses(line.type) : "text-slate-600"}`}
                  >
                    {line ? formatLineNumber(line.newLineNumber) : ""}
                  </div>
                  <div
                    class="overflow-x-hidden px-3 py-0.5 font-mono text-[12px] leading-6"
                  >
                    <span class="block whitespace-pre-wrap break-all">
                      {line ? line.content : " "}
                    </span>
                    {#if line?.noTrailingNewLine}
                      <div class="text-[11px] italic text-amber-200">
                        \ No newline at end of file
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
        {#each parsedDiff.hunks as hunk}
          <div
            class="border-b border-[#373e47]/70 bg-sky-500/10 px-4 py-1.5 font-mono text-[12px] text-sky-200"
          >
            {hunk.header}
          </div>
          {#each hunk.lines as line}
            <div
              class={`grid grid-cols-[4rem_4rem_minmax(0,1fr)] border-b border-[#373e47]/70 ${lineToneClasses(line.type)}`}
            >
              <div
                class={`border-r border-[#373e47]/60 px-2 py-0.5 text-right font-mono text-[11px] ${lineNumberToneClasses(line.type)}`}
              >
                {formatLineNumber(line.oldLineNumber)}
              </div>
              <div
                class={`border-r border-[#373e47]/60 px-2 py-0.5 text-right font-mono text-[11px] ${lineNumberToneClasses(line.type)}`}
              >
                {formatLineNumber(line.newLineNumber)}
              </div>
              <div
                class="overflow-x-hidden px-3 py-0.5 font-mono text-[12px] leading-6"
              >
                <span class="block whitespace-pre-wrap break-all">
                  {line.content || " "}
                </span>
                {#if line.noTrailingNewLine}
                  <div class="text-[11px] italic text-amber-200">
                    \ No newline at end of file
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
