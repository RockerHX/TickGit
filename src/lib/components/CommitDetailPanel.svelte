<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { CommitFileChange, CommitListItem, CommitMeta } from "$lib/types";
  import {
    diffLineClass,
    formatAbsoluteDate,
    getInitials,
    statusTone,
  } from "$lib/utils";

  export let commit: CommitListItem | null = null;
  export let commitMeta: CommitMeta | null = null;
  export let files: CommitFileChange[] = [];
  export let selectedFilePath: string | null = null;
  export let diffText = "";
  export let loadingFiles = false;
  export let loadingDiff = false;

  const dispatch = createEventDispatcher<{ selectFile: { path: string } }>();

  $: diffLines = diffText ? diffText.split("\n") : [];
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden bg-[#2b3036]">
  <div class="border-b border-[#1f2328] bg-[#2d333b] px-5 py-3">
    {#if commit}
      <div class="min-w-0">
        <div class="flex items-start gap-3">
          <div class="min-w-0 flex-1">
            <div class="truncate text-[1.15rem] font-semibold text-[#f0f6fc]">
              {commit.summary}
            </div>
            {#if commitMeta?.body}
              <div class="mt-1 whitespace-pre-wrap text-[0.95rem] leading-6 text-slate-200">
                {commitMeta.body}
              </div>
            {/if}
          </div>
          {#if !commit.isPushed}
            <span
              class="mt-0.5 flex shrink-0 items-center text-[#f0f6fc]"
              title="Local commit"
            >
              <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current" aria-hidden="true">
                <path d="M8 3.25a.75.75 0 0 1 .75.75v5.19l1.72-1.72a.75.75 0 1 1 1.06 1.06l-3 3a.75.75 0 0 1-1.06 0l-3-3a.75.75 0 1 1 1.06-1.06l1.72 1.72V4A.75.75 0 0 1 8 3.25Z"></path>
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
          <span class="truncate text-slate-300">&lt;{commit.authorEmail}&gt;</span>
        </div>

        <div class="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 text-[13px] text-[#f0f6fc]">
          <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current" aria-hidden="true">
            <path d="M1.75 8a2.75 2.75 0 1 1 5.18 1.28h2.14a2.751 2.751 0 0 1 5.18-1.28 2.75 2.75 0 1 1-5.18 1.28H6.93A2.75 2.75 0 1 1 1.75 8Zm2.75-1.25a1.25 1.25 0 1 0 0 2.5 1.25 1.25 0 0 0 0-2.5Zm7 0a1.25 1.25 0 1 0 .001 2.501A1.25 1.25 0 0 0 11.5 6.75Z"></path>
          </svg>
          <span class="font-mono">{commit.hash}</span>
          <svg viewBox="0 0 16 16" class="ml-1 h-4 w-4 shrink-0 fill-[#f0f6fc]" aria-hidden="true">
            <path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25Z"></path>
            <path d="M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"></path>
          </svg>
          <span class="text-slate-400">{formatAbsoluteDate(commit.committedAt)}</span>
        </div>

        {#if commitMeta}
          <div class="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-[13px]">
            <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-[#f0f6fc]" aria-hidden="true">
              <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v5.5h5.5a.75.75 0 0 1 0 1.5h-5.5v5.5a.75.75 0 0 1-1.5 0v-5.5h-5.5a.75.75 0 0 1 0-1.5h5.5Z"></path>
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
