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

<div class="flex h-full min-h-0 flex-col overflow-hidden bg-[#22272e]">
  <div class="border-b border-[#30363d] bg-[#22272e] px-5 py-4">
    {#if commit}
      <div class="flex items-start justify-between gap-6">
        <div class="min-w-0">
          <div class="text-[1.6rem] font-semibold text-[#f0f6fc]">
            {commit.summary}
          </div>
          <div
            class="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-sm text-slate-400"
          >
            <span>{commit.authorName}</span>
            <span>{commit.authorEmail}</span>
            <span>{formatAbsoluteDate(commit.committedAt)}</span>
          </div>
        </div>
        <div
          class="min-w-[280px] rounded-md border border-[#30363d] bg-[#1f2428] px-3 py-2 text-right"
        >
          <div class="text-[11px] uppercase tracking-[0.22em] text-slate-500">
            Commit
          </div>
          <div class="mt-1 truncate font-mono text-sm text-slate-200">
            {commit.hash}
          </div>
        </div>
      </div>
    {:else}
      <div class="text-sm text-slate-500">选择左侧 Commit 查看详情</div>
    {/if}
  </div>

  <div class="grid min-h-0 flex-1 grid-cols-[360px_minmax(0,1fr)]">
    <div class="flex min-h-0 flex-col border-r border-[#30363d] bg-[#22272e]">
      <div
        class="border-b border-[#30363d] px-4 py-3 text-sm font-semibold text-[#f0f6fc]"
      >
        {files.length} changed files
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto">
        {#if loadingFiles}
          <div class="px-4 py-4 text-sm text-slate-400">正在读取文件变更…</div>
        {:else if files.length === 0}
          <div
            class="m-4 border border-dashed border-[#3d444d] bg-[#1f2428] px-4 py-8 text-center text-sm text-slate-500"
          >
            当前 Commit 没有可展示的文件变更
          </div>
        {:else}
          <div>
            {#each files as file (file.path + file.status)}
              <button
                class={`w-full border-b border-[#30363d] px-4 py-3 text-left transition ${
                  selectedFilePath === file.path
                    ? "bg-[#1f6feb]/20"
                    : "bg-transparent hover:bg-[#2d333b]"
                }`}
                on:click={() => dispatch("selectFile", { path: file.path })}
              >
                <div class="flex items-start gap-3">
                  <span
                    class={`mt-0.5 rounded-sm border px-2 py-0.5 text-[11px] font-medium ${statusTone(file.status)}`}
                  >
                    {file.status}
                  </span>
                  <span class="break-all text-sm leading-5 text-slate-200"
                    >{file.displayPath}</span
                  >
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <div class="flex min-h-0 flex-col bg-[#1f2428]">
      <div
        class="flex items-center justify-between gap-3 border-b border-[#30363d] px-4 py-3 text-sm"
      >
        <div class="truncate font-semibold text-[#f0f6fc]">
          {selectedFilePath ?? "Diff"}
        </div>
        <div class="shrink-0 text-xs uppercase tracking-[0.18em] text-slate-500">
          Diff
        </div>
      </div>
      <div class="min-h-0 flex-1 overflow-auto">
        {#if loadingDiff}
          <div class="px-4 py-4 text-sm text-slate-400">正在加载 Diff…</div>
        {:else if !selectedFilePath}
          <div
            class="m-4 border border-dashed border-[#3d444d] bg-[#22272e] px-4 py-10 text-center text-sm text-slate-500"
          >
            选择一个文件查看具体代码差异
          </div>
        {:else if !diffText}
          <div
            class="m-4 border border-dashed border-[#3d444d] bg-[#22272e] px-4 py-10 text-center text-sm text-slate-500"
          >
            当前文件没有可显示的 Diff 内容
          </div>
        {:else}
          <pre
            class="min-h-full overflow-x-auto bg-[#1f2428] text-[12px] leading-6 text-slate-300">
            {#each diffLines as line}
              <div class={`border-b border-[#2d333b]/70 px-4 py-0.5 ${diffLineClass(line)}`}>
                {line || " "}
              </div>
            {/each}
          </pre>
        {/if}
      </div>
    </div>
  </div>
</div>
