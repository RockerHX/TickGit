<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { CommitFileChange, CommitListItem } from "$lib/types";
  import {
    diffLineClass,
    formatAbsoluteDate,
    getInitials,
    statusTone,
  } from "$lib/utils";

  export let commit: CommitListItem | null = null;
  export let files: CommitFileChange[] = [];
  export let selectedFilePath: string | null = null;
  export let diffText = "";
  export let loadingFiles = false;
  export let loadingDiff = false;

  const dispatch = createEventDispatcher<{ selectFile: { path: string } }>();

  $: diffLines = diffText ? diffText.split("\n") : [];
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden bg-[#2b3036]">
  <div class="border-b border-[#1f2328] bg-[#2d333b] px-5 py-4">
    {#if commit}
      <div class="flex items-start justify-between gap-6">
        <div class="flex min-w-0 items-start gap-3">
          <div
            class="mt-0.5 flex h-10 w-10 shrink-0 items-center justify-center rounded-full border border-[#444c56] bg-[#373e47] text-xs font-semibold text-[#f0f6fc]"
          >
            {getInitials(commit.authorName)}
          </div>

          <div class="min-w-0">
            <div class="truncate text-[1.35rem] font-semibold text-[#f0f6fc]">
              {commit.summary}
            </div>
            <div
              class="mt-1.5 flex flex-wrap items-center gap-x-2 gap-y-1 text-[13px] text-slate-400"
            >
              <span class="font-medium text-slate-300">{commit.authorName}</span>
              <span>{commit.authorEmail}</span>
              <span>•</span>
              <span>{formatAbsoluteDate(commit.committedAt)}</span>
            </div>
          </div>
        </div>

        <div
          class="min-w-[300px] rounded-sm border border-[#444c56] bg-[#2b3036] px-3 py-2 text-right"
        >
          <div class="text-[11px] uppercase tracking-[0.2em] text-slate-500">
            Commit
          </div>
          <div class="mt-1 truncate font-mono text-sm text-slate-200">
            {commit.hash}
          </div>
        </div>
      </div>
    {:else}
      <div class="text-sm text-slate-500">Select a commit to inspect its details</div>
    {/if}
  </div>

  <div class="grid min-h-0 flex-1 grid-cols-[340px_minmax(0,1fr)]">
    <div class="flex min-h-0 flex-col border-r border-[#1f2328] bg-[#2d333b]">
      <div
        class="flex items-center justify-between gap-3 border-b border-[#1f2328] px-4 py-3"
      >
        <div class="text-sm font-semibold text-[#f0f6fc]">changed files</div>
        <div class="text-xs font-medium text-slate-400">{files.length}</div>
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto bg-[#2d333b]">
        {#if loadingFiles}
          <div class="px-4 py-4 text-sm text-slate-400">Loading changed files…</div>
        {:else if files.length === 0}
          <div
            class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2b3036] px-4 py-8 text-center text-sm text-slate-500"
          >
            No file changes available for this commit
          </div>
        {:else}
          <div>
            {#each files as file (file.path + file.status)}
              <button
                class={`w-full border-b border-[#373e47] px-4 py-2.5 text-left transition ${
                  selectedFilePath === file.path
                    ? "bg-[#347dff]/12"
                    : "bg-transparent hover:bg-[#373e47]/45"
                }`}
                title={file.displayPath}
                on:click={() => dispatch("selectFile", { path: file.path })}
              >
                <div class="flex items-center gap-3">
                  <span
                    class={`flex h-6 min-w-6 items-center justify-center rounded-full border px-1.5 text-[10px] font-semibold uppercase ${statusTone(file.status)}`}
                  >
                    {file.status}
                  </span>
                  <span class="truncate text-[13px] leading-5 text-slate-200">
                    {file.displayPath}
                  </span>
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <div class="flex min-h-0 flex-col border-l border-[#373e47] bg-[#2b3036]">
      <div
        class="flex items-center justify-between gap-3 border-b border-[#1f2328] bg-[#2d333b] px-4 py-3 text-sm"
      >
        <div class="truncate font-semibold text-[#f0f6fc]">
          {selectedFilePath ?? "Diff"}
        </div>
        <div class="shrink-0 text-xs uppercase tracking-[0.18em] text-slate-500">
          Diff
        </div>
      </div>
      <div class="min-h-0 flex-1 overflow-auto bg-[#2b3036]">
        {#if loadingDiff}
          <div class="px-4 py-4 text-sm text-slate-400">Loading diff…</div>
        {:else if !selectedFilePath}
          <div
            class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2d333b] px-4 py-10 text-center text-sm text-slate-500"
          >
            Select a changed file to inspect the diff
          </div>
        {:else if !diffText}
          <div
            class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2d333b] px-4 py-10 text-center text-sm text-slate-500"
          >
            No diff content is available for this file
          </div>
        {:else}
          <pre
            class="min-h-full overflow-x-auto bg-[#2b3036] text-[12px] leading-6 text-slate-300">
            {#each diffLines as line}
              <div class={`border-b border-[#373e47]/70 px-4 py-0.5 ${diffLineClass(line)}`}>
                {line || " "}
              </div>
            {/each}
          </pre>
        {/if}
      </div>
    </div>
  </div>
</div>
