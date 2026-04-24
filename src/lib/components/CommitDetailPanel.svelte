<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { CommitFileChange, CommitListItem } from "$lib/types";
  import { diffLineClass, formatAbsoluteDate, statusTone } from "$lib/utils";

  export let commit: CommitListItem | null = null;
  export let files: CommitFileChange[] = [];
  export let selectedFilePath: string | null = null;
  export let diffText = "";
  export let loadingFiles = false;
  export let loadingDiff = false;

  const dispatch = createEventDispatcher<{ selectFile: { path: string } }>();

  $: diffLines = diffText ? diffText.split("\n") : [];
</script>

<div
  class="flex h-full flex-col overflow-hidden rounded-3xl border border-slate-800/80 bg-slate-950/70"
>
  <div class="border-b border-slate-800 px-6 py-5">
    {#if commit}
      <div class="flex items-start justify-between gap-6">
        <div class="min-w-0">
          <div class="text-xl font-semibold text-white">{commit.summary}</div>
          <div
            class="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-sm text-slate-400"
          >
            <span>{commit.authorName}</span>
            <span>{commit.authorEmail}</span>
            <span>{formatAbsoluteDate(commit.committedAt)}</span>
          </div>
        </div>
        <div
          class="rounded-2xl border border-slate-800 bg-slate-900/80 px-3 py-2 text-right"
        >
          <div class="text-[11px] uppercase tracking-[0.24em] text-slate-500">
            Commit
          </div>
          <div class="mt-1 font-mono text-sm text-slate-200">{commit.hash}</div>
        </div>
      </div>
    {:else}
      <div class="text-sm text-slate-500">选择左侧 Commit 查看详情</div>
    {/if}
  </div>

  <div class="grid min-h-0 flex-1 grid-cols-[320px_minmax(0,1fr)]">
    <div class="min-h-0 border-r border-slate-800">
      <div
        class="border-b border-slate-800 px-4 py-3 text-sm font-semibold text-white"
      >
        变更文件
      </div>
      <div class="h-full overflow-y-auto p-3">
        {#if loadingFiles}
          <div class="px-2 py-4 text-sm text-slate-400">正在读取文件变更…</div>
        {:else if files.length === 0}
          <div
            class="rounded-2xl border border-dashed border-slate-800 bg-slate-900/50 px-4 py-8 text-center text-sm text-slate-500"
          >
            当前 Commit 没有可展示的文件变更
          </div>
        {:else}
          <div class="space-y-2">
            {#each files as file (file.path + file.status)}
              <button
                class={`w-full rounded-2xl border px-3 py-3 text-left transition ${
                  selectedFilePath === file.path
                    ? "border-sky-400/40 bg-sky-500/10"
                    : "border-slate-800 bg-slate-900/50 hover:border-slate-700 hover:bg-slate-900/80"
                }`}
                on:click={() => dispatch("selectFile", { path: file.path })}
              >
                <div class="flex items-start gap-3">
                  <span
                    class={`rounded-full border px-2 py-1 text-[11px] font-medium ${statusTone(file.status)}`}
                  >
                    {file.status}
                  </span>
                  <span class="break-all text-sm text-slate-200"
                    >{file.displayPath}</span
                  >
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <div class="min-h-0 overflow-y-auto">
      <div
        class="border-b border-slate-800 px-4 py-3 text-sm font-semibold text-white"
      >
        Diff
      </div>
      <div class="p-4">
        {#if loadingDiff}
          <div class="text-sm text-slate-400">正在加载 Diff…</div>
        {:else if !selectedFilePath}
          <div
            class="rounded-2xl border border-dashed border-slate-800 bg-slate-900/50 px-4 py-10 text-center text-sm text-slate-500"
          >
            选择一个文件查看具体代码差异
          </div>
        {:else if !diffText}
          <div
            class="rounded-2xl border border-dashed border-slate-800 bg-slate-900/50 px-4 py-10 text-center text-sm text-slate-500"
          >
            当前文件没有可显示的 Diff 内容
          </div>
        {:else}
          <pre
            class="overflow-x-auto rounded-3xl border border-slate-800 bg-[#020617] p-4 text-[12px] leading-6">
            {#each diffLines as line}
              <div class={`px-3 ${diffLineClass(line)}`}>{line || " "}</div>
            {/each}
          </pre>
        {/if}
      </div>
    </div>
  </div>
</div>
