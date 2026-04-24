<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { BranchStatus, CommitListItem } from "$lib/types";
  import { formatRelativeDate } from "$lib/utils";

  export let commits: CommitListItem[] = [];
  export let selectedHash: string | null = null;
  export let loading = false;
  export let hasMore = false;
  export let branchStatus: BranchStatus | null = null;

  const dispatch = createEventDispatcher<{
    select: { commit: CommitListItem };
    loadMore: void;
    openMenu: { commit: CommitListItem; x: number; y: number };
  }>();

  function handleScroll(event: Event) {
    const target = event.currentTarget as HTMLDivElement;
    const threshold = 180;
    if (
      target.scrollHeight - target.scrollTop - target.clientHeight <
        threshold &&
      hasMore &&
      !loading
    ) {
      dispatch("loadMore");
    }
  }

  function openMenu(event: MouseEvent, commit: CommitListItem) {
    if (commit.isPushed || !branchStatus?.pushAvailable) {
      return;
    }

    event.preventDefault();
    dispatch("openMenu", { commit, x: event.clientX, y: event.clientY });
  }
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden border-r border-[#30363d] bg-[#22272e]">
  <div class="border-b border-[#30363d]">
    <div class="grid grid-cols-2">
      <div
        class="border-b border-[#30363d] px-4 py-3 text-center text-sm font-medium text-slate-500"
      >
        Changes
      </div>
      <div
        class="border-b-2 border-[#2f81f7] px-4 py-3 text-center text-sm font-semibold text-[#f0f6fc]"
      >
        History
      </div>
    </div>
  </div>

  <div class="border-b border-[#30363d] px-4 py-3">
    <div class="text-sm font-semibold text-[#f0f6fc]">Commit 历史</div>
    <div class="mt-1 text-xs text-slate-400">
      {#if branchStatus?.pushAvailable}
        未推送 {branchStatus.aheadCount} 条 · 落后 {branchStatus.behindCount} 条
      {:else}
        {branchStatus?.disabledReason ?? "当前仓库未启用推送"}
      {/if}
    </div>
  </div>

  <div
    class="min-h-0 flex-1 overflow-y-auto"
    on:scroll={handleScroll}
  >
    {#if commits.length === 0 && !loading}
      <div
        class="m-4 border border-dashed border-[#3d444d] bg-[#1f2428] px-4 py-10 text-center text-sm text-slate-500"
      >
        当前仓库暂无提交记录
      </div>
    {/if}

    <div>
      {#each commits as commit (commit.hash)}
        <button
          class={`w-full border-b border-[#30363d] px-4 py-3 text-left transition ${
            selectedHash === commit.hash
              ? "bg-[#1f6feb]/30"
              : "bg-transparent hover:bg-[#2d333b]"
          }`}
          on:click={() => dispatch("select", { commit })}
          on:contextmenu={(event) => openMenu(event, commit)}
        >
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <div class="truncate text-sm font-semibold text-[#f0f6fc]">
                {commit.summary}
              </div>
              <div
                class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-slate-400"
              >
                <span>{commit.authorName}</span>
                <span>•</span>
                <span>{formatRelativeDate(commit.committedAt)}</span>
                <span>•</span>
                <span class="font-mono text-slate-300">{commit.shortHash}</span>
              </div>
            </div>

            <span
              class={`mt-0.5 shrink-0 rounded-sm border px-2 py-0.5 text-[11px] font-medium ${
                commit.isPushed
                  ? "border-[#3d444d] bg-[#2d333b] text-slate-300"
                  : "border-[#8b6d1f] bg-[#3d2d13] text-[#f2cc60]"
              }`}
            >
              {commit.isPushed ? "已推送" : "未推送"}
            </span>
          </div>
        </button>
      {/each}
    </div>

    {#if loading}
      <div class="px-4 py-4 text-center text-xs text-slate-400">
        正在加载提交历史…
      </div>
    {/if}
  </div>
</div>
