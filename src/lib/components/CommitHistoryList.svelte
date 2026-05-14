<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { BranchStatus, CommitListItem } from "$lib/types";
  import { formatRelativeDate, getInitials } from "$lib/utils";

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

<div class="flex h-full min-h-0 flex-col overflow-hidden bg-[#2d333b]">
  <div class="border-b border-[#1f2328] px-4 py-3">
    <div class="text-sm font-semibold text-[#f0f6fc]">History</div>
    <div class="mt-1 text-xs text-slate-400">
      {#if branchStatus?.pushAvailable}
        Ahead {branchStatus.aheadCount} · Safe step-push {branchStatus.safeAheadCount}
        · Behind {branchStatus.behindCount}
      {:else}
        {branchStatus?.disabledReason ?? "当前仓库未启用推送"}
      {/if}
    </div>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto" on:scroll={handleScroll}>
    {#if commits.length === 0 && !loading}
      <div
        class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2b3036] px-4 py-10 text-center text-sm text-slate-500"
      >
        No commits found for this repository
      </div>
    {/if}

    <div>
      {#each commits as commit (commit.hash)}
        <button
          class={`group relative w-full border-b border-[#373e47] px-4 py-3 text-left transition ${
            selectedHash === commit.hash
              ? "bg-[#347dff]/14"
              : "bg-transparent hover:bg-[#373e47]/45"
          }`}
          on:click={() => dispatch("select", { commit })}
          on:contextmenu={(event) => openMenu(event, commit)}
        >
          {#if selectedHash === commit.hash}
            <div
              class="absolute inset-y-2 left-0 w-1 rounded-r-full bg-[#2f81f7]"
            ></div>
          {/if}

          <div class="flex items-start gap-3">
            <div class="relative mt-0.5 shrink-0">
              {#if !commit.isPushed}
                {#if commit.isSafePushTarget}
                  <span
                    class="absolute -left-1 -top-1 z-10 flex h-4.5 w-4.5 items-center justify-center rounded-full border border-emerald-400/45 bg-[#1f2328] text-emerald-200 shadow-sm shadow-black/35"
                    title="可安全分步推送"
                    aria-label="可安全分步推送"
                  >
                    <svg
                      viewBox="0 0 16 16"
                      class="h-3 w-3 fill-current"
                      aria-hidden="true"
                    >
                      <path
                        d="M7.25 2.5a.75.75 0 0 1 1.5 0v6.69l1.22-1.22a.75.75 0 1 1 1.06 1.06l-2.5 2.5a.75.75 0 0 1-1.06 0l-2.5-2.5a.75.75 0 1 1 1.06-1.06l1.22 1.22V2.5Zm-4 10.75a.75.75 0 0 1 .75-.75h8a.75.75 0 0 1 0 1.5H4a.75.75 0 0 1-.75-.75Z"
                      ></path>
                    </svg>
                  </span>
                {:else}
                  <span
                    class="absolute -left-1 -top-1 z-10 flex h-4.5 w-4.5 items-center justify-center rounded-full border border-rose-400/45 bg-[#1f2328] text-rose-200 shadow-sm shadow-black/35"
                    title={commit.pushBlockedReason ??
                      "当前 Commit 不能安全分步推送"}
                    aria-label="不能安全分步推送"
                  >
                    <svg
                      viewBox="0 0 16 16"
                      class="h-3 w-3 fill-current"
                      aria-hidden="true"
                    >
                      <path
                        d="M8 1.75a6.25 6.25 0 1 0 0 12.5 6.25 6.25 0 0 0 0-12.5Zm0 1.5a4.75 4.75 0 0 1 3.07 8.37L4.38 4.93A4.73 4.73 0 0 1 8 3.25Zm-3.07 1.68 6.69 6.69A4.75 4.75 0 0 1 4.93 4.93Z"
                      ></path>
                    </svg>
                  </span>
                {/if}
              {/if}

              <div
                class={`flex h-8 w-8 items-center justify-center rounded-full border text-[11px] font-semibold ${
                  selectedHash === commit.hash
                    ? "border-[#539bf5]/40 bg-[#2f81f7]/20 text-[#cae8ff]"
                    : "border-[#444c56] bg-[#373e47] text-slate-200"
                }`}
              >
                {getInitials(commit.authorName)}
              </div>
            </div>

            <div class="min-w-0 flex-1">
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <div class="flex min-w-0 items-center gap-1.5">
                    <div
                      class="min-w-0 flex-1 truncate text-[13px] font-semibold text-[#f0f6fc]"
                    >
                      {commit.summary}
                    </div>
                  </div>
                  {#if commit.tags.length > 0}
                    <div class="mt-1 flex flex-wrap items-center gap-1">
                      {#each commit.tags as tag}
                        <span
                          class="max-w-full truncate rounded-full border border-amber-400/30 bg-amber-400/10 px-2 py-0.5 text-[10px] font-medium text-amber-200"
                          title={tag}
                        >
                          {tag}
                        </span>
                      {/each}
                    </div>
                  {/if}
                </div>

                <div class="mt-0.5 flex shrink-0 items-center gap-1.5">
                  {#if !commit.isPushed}
                    <span
                      class="flex h-7 w-7 items-center justify-center rounded-full bg-[#6e7681] text-[#f0f6fc]"
                      title="Local commit"
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
                    </span>
                  {:else}
                    <svg
                      viewBox="0 0 16 16"
                      class="h-3.5 w-3.5 fill-[#8b949e]"
                      aria-hidden="true"
                    >
                      <path
                        d="M13.78 4.97a.75.75 0 0 1 0 1.06L7.53 12.28a.75.75 0 0 1-1.06 0L2.22 8.03a.75.75 0 0 1 1.06-1.06L7 10.69l5.72-5.72a.75.75 0 0 1 1.06 0Z"
                      ></path>
                    </svg>
                  {/if}
                </div>
              </div>

              <div
                class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-[12px] text-slate-400"
              >
                <span>{commit.authorName}</span>
                <span>•</span>
                <span>{formatRelativeDate(commit.committedAt)}</span>
                <span>•</span>
                <span class="font-mono text-slate-300">{commit.shortHash}</span>
              </div>
            </div>
          </div>
        </button>
      {/each}
    </div>

    {#if loading}
      <div class="px-4 py-4 text-center text-xs text-slate-400">
        Loading history…
      </div>
    {/if}
  </div>
</div>
