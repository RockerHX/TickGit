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

<div
  class="flex h-full flex-col overflow-hidden rounded-3xl border border-slate-800/80 bg-slate-950/70"
>
  <div class="border-b border-slate-800 px-5 py-4">
    <div class="text-sm font-semibold text-white">Commit 历史</div>
    <div class="mt-1 text-xs text-slate-400">
      {#if branchStatus?.pushAvailable}
        未推送 {branchStatus.aheadCount} 条 · 落后 {branchStatus.behindCount} 条
      {:else}
        {branchStatus?.disabledReason ?? "当前仓库未启用推送"}
      {/if}
    </div>
  </div>

  <div
    class="min-h-0 flex-1 overflow-y-auto px-3 py-3"
    on:scroll={handleScroll}
  >
    {#if commits.length === 0 && !loading}
      <div
        class="rounded-2xl border border-dashed border-slate-800 bg-slate-900/60 px-4 py-10 text-center text-sm text-slate-500"
      >
        当前仓库暂无提交记录
      </div>
    {/if}

    <div class="space-y-2">
      {#each commits as commit (commit.hash)}
        <button
          class={`w-full rounded-2xl border px-4 py-3 text-left transition ${
            selectedHash === commit.hash
              ? "border-sky-400/50 bg-sky-500/10 shadow-lg shadow-sky-950/20"
              : commit.isPushed
                ? "border-slate-800 bg-slate-900/40 hover:border-slate-700 hover:bg-slate-900/80"
                : "border-amber-500/30 bg-amber-500/10 hover:border-amber-400/40 hover:bg-amber-500/14"
          }`}
          on:click={() => dispatch("select", { commit })}
          on:contextmenu={(event) => openMenu(event, commit)}
        >
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <div class="truncate text-sm font-semibold text-white">
                {commit.summary}
              </div>
              <div
                class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-slate-400"
              >
                <span>{commit.authorName}</span>
                <span>•</span>
                <span>{formatRelativeDate(commit.committedAt)}</span>
                <span>•</span>
                <span class="font-mono">{commit.shortHash}</span>
              </div>
            </div>

            <span
              class={`shrink-0 rounded-full px-2 py-1 text-[11px] font-medium ${
                commit.isPushed
                  ? "bg-slate-800 text-slate-300"
                  : "bg-amber-500/20 text-amber-200"
              }`}
            >
              {commit.isPushed ? "已推送" : "未推送"}
            </span>
          </div>
        </button>
      {/each}
    </div>

    {#if loading}
      <div class="px-2 py-4 text-center text-xs text-slate-400">
        正在加载提交历史…
      </div>
    {/if}
  </div>
</div>
