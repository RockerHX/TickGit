<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate, translateBranchDisabledReason } from "$lib/i18n";
  import HistoryFilters from "$lib/components/HistoryFilters.svelte";
  import MainViewTabs, {
    type MainViewId,
  } from "$lib/components/MainViewTabs.svelte";
  import type {
    BranchStatus,
    CommitHistoryFilters,
    CommitListItem,
  } from "$lib/types";
  import { formatRelativeDate, getInitials } from "$lib/utils";
  import { EMPTY_HISTORY_FILTERS } from "$lib/tickgit/history";
  import {
    HISTORY_PAGE_SIZE,
    getPaginationState,
  } from "$lib/tickgit/pagination";

  export let commits: CommitListItem[] = [];
  export let selectedHash: string | null = null;
  export let loading = false;
  export let totalCount = 0;
  export let pageIndex = 0;
  export let pageSize = HISTORY_PAGE_SIZE;
  export let branchStatus: BranchStatus | null = null;
  export let filters: CommitHistoryFilters = EMPTY_HISTORY_FILTERS;
  export let activeFilterCount = 0;
  export let activeMainView: MainViewId = "history";

  const dispatch = createEventDispatcher<{
    select: { commit: CommitListItem };
    pageChange: { pageIndex: number };
    mainViewChange: { view: MainViewId };
    openMenu: { commit: CommitListItem; x: number; y: number };
    filterChange: { filters: CommitHistoryFilters };
    clearFilters: void;
  }>();

  $: pagination = getPaginationState(totalCount, pageIndex, pageSize);

  function openMenu(event: MouseEvent, commit: CommitListItem) {
    if (commit.isPushed || !branchStatus?.pushAvailable) {
      return;
    }

    event.preventDefault();
    dispatch("openMenu", { commit, x: event.clientX, y: event.clientY });
  }

  function changePage(nextPageIndex: number) {
    if (loading) {
      return;
    }

    dispatch("pageChange", { pageIndex: nextPageIndex });
  }
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden bg-[#2d333b]">
  <div class="border-b border-[#1f2328] px-4 py-3">
    <MainViewTabs
      active={activeMainView}
      on:change={(event) =>
        dispatch("mainViewChange", { view: event.detail.view })}
    />

    <div class="mt-3 flex items-center justify-between gap-3">
      <div class="text-sm font-semibold text-[#f0f6fc]">
        {translate($locale, "history.title")}
      </div>
      {#if activeFilterCount > 0}
        <button
          type="button"
          class="rounded-md border border-[#444c56] bg-[#373e47] px-2 py-1 text-[11px] font-medium text-slate-200 transition hover:border-[#539bf5]/45 hover:bg-[#347dff]/15"
          on:click={() => dispatch("clearFilters")}
        >
          {translate($locale, "history.clearFilters", {
            count: activeFilterCount,
          })}
        </button>
      {/if}
    </div>
    <div class="mt-1 text-xs text-slate-400">
      {#if branchStatus?.pushAvailable}
        {translate($locale, "history.branchStats", {
          aheadCount: branchStatus.aheadCount,
          safeAheadCount: branchStatus.safeAheadCount,
          behindCount: branchStatus.behindCount,
        })}
      {:else}
        {translateBranchDisabledReason(
          $locale,
          branchStatus?.disabledReasonCode,
          branchStatus?.disabledReason,
        )}
      {/if}
    </div>
    <HistoryFilters
      {filters}
      on:filterChange={(event) =>
        dispatch("filterChange", { filters: event.detail.filters })}
    />
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto">
    {#if commits.length === 0 && !loading}
      <div
        class="m-4 rounded-sm border border-dashed border-[#444c56] bg-[#2b3036] px-4 py-10 text-center text-sm text-slate-500"
      >
        {activeFilterCount > 0
          ? translate($locale, "history.noMatchingCommits")
          : translate($locale, "history.noCommits")}
      </div>
    {/if}

    <div class="space-y-2.5 p-3">
      {#each commits as commit (commit.hash)}
        {@const isSelected = selectedHash === commit.hash}
        {@const relativeTime = formatRelativeDate(commit.committedAt, $locale)}
        {@const pushStatusTitle = commit.isPushed
          ? translate($locale, "history.pushedCommit")
          : commit.isSafePushTarget
            ? translate($locale, "history.safeStepPush")
            : (commit.pushBlockedReason ??
              translate($locale, "history.unsafeStepPushFallback"))}
        <button
          type="button"
          class={`group relative min-h-[92px] w-full overflow-hidden rounded-2xl border px-4 py-3.5 text-left shadow-sm outline-none transition duration-150 focus-visible:ring-2 focus-visible:ring-[#60a5fa]/60 focus-visible:ring-offset-2 focus-visible:ring-offset-[#2d333b] ${
            isSelected
              ? "border-[#60a5fa]/60 bg-gradient-to-r from-[#2563eb]/36 via-[#1d4ed8]/24 to-[#0f172a]/28 shadow-[0_14px_34px_rgba(37,99,235,0.2)]"
              : "border-[#334155]/25 bg-[#0f172a]/18 shadow-black/10 hover:border-[#3b82f6]/30 hover:bg-[#1e293b]/42 hover:shadow-[0_10px_26px_rgba(15,23,42,0.24)]"
          }`}
          on:click={() => dispatch("select", { commit })}
          on:contextmenu={(event) => openMenu(event, commit)}
        >
          {#if isSelected}
            <div
              class="absolute inset-y-2 left-0 w-1.5 rounded-r-full bg-gradient-to-b from-[#93c5fd] via-[#60a5fa] to-[#2563eb] shadow-[0_0_18px_rgba(96,165,250,0.6)]"
            ></div>
          {/if}

          <div class="flex items-start gap-3">
            <div class="relative mt-0.5 shrink-0">
              {#if !commit.isPushed}
                {#if commit.isSafePushTarget}
                  <span
                    class="absolute -left-1.5 -top-1.5 z-10 flex h-5 w-5 items-center justify-center rounded-full border border-emerald-300/60 bg-gradient-to-br from-emerald-500/25 to-[#0f172a] text-emerald-200 shadow-[0_0_14px_rgba(52,211,153,0.32)] ring-2 ring-[#0f172a]"
                    title={pushStatusTitle}
                    aria-label={pushStatusTitle}
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
                    class="absolute -left-1.5 -top-1.5 z-10 flex h-5 w-5 items-center justify-center rounded-full border border-rose-300/60 bg-gradient-to-br from-rose-500/25 to-[#0f172a] text-rose-200 shadow-[0_0_14px_rgba(251,113,133,0.28)] ring-2 ring-[#0f172a]"
                    title={pushStatusTitle}
                    aria-label={pushStatusTitle}
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
                class={`flex h-9 w-9 items-center justify-center rounded-full border text-[11px] font-semibold shadow-sm shadow-black/20 transition ${
                  isSelected
                    ? "border-[#93c5fd]/60 bg-[#2563eb]/35 text-[#dbeafe]"
                    : "border-[#334155] bg-[#1e293b] text-slate-200 group-hover:border-[#475569] group-hover:bg-[#243247]"
                }`}
              >
                {getInitials(commit.authorName)}
              </div>
            </div>

            <div class="min-w-0 flex-1">
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0 flex-1">
                  <div class="flex min-w-0 items-center gap-1.5">
                    <div
                      class="min-w-0 flex-1 truncate text-[14px] font-semibold leading-5 text-[#f8fafc]"
                      title={commit.summary}
                    >
                      {commit.summary}
                    </div>
                  </div>
                  {#if commit.tags.length > 0}
                    <div
                      class="mt-1 flex max-w-full flex-wrap items-center gap-1 overflow-hidden"
                    >
                      {#each commit.tags as tag}
                        <span
                          class="max-w-[10rem] truncate rounded-full border border-amber-400/30 bg-amber-400/10 px-2 py-0.5 text-[10px] font-medium text-amber-200"
                          title={tag}
                        >
                          {tag}
                        </span>
                      {/each}
                    </div>
                  {/if}
                </div>

                <div class="mt-0.5 flex shrink-0 items-center gap-2">
                  <span
                    class={`rounded-full px-2 py-0.5 text-[11px] font-semibold ${
                      isSelected
                        ? "bg-[#60a5fa]/20 text-[#bfdbfe]"
                        : "bg-[#1e293b] text-slate-400 group-hover:text-slate-200"
                    }`}
                  >
                    {relativeTime}
                  </span>
                  {#if !commit.isPushed}
                    <span
                      class={`h-2.5 w-2.5 rounded-full ring-2 ring-[#0f172a]/90 ${
                        commit.isSafePushTarget
                          ? "bg-emerald-400 shadow-[0_0_10px_rgba(52,211,153,0.55)]"
                          : "bg-rose-400 shadow-[0_0_10px_rgba(251,113,133,0.45)]"
                      }`}
                      title={pushStatusTitle}
                      aria-label={pushStatusTitle}
                    ></span>
                  {:else}
                    <span
                      class="h-2.5 w-2.5 rounded-full bg-slate-600 ring-2 ring-[#0f172a]/90"
                      title={pushStatusTitle}
                      aria-label={pushStatusTitle}
                    ></span>
                  {/if}
                </div>
              </div>

              <div
                class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-[12px] text-slate-400"
              >
                <span class="min-w-0 max-w-[9rem] truncate">
                  {commit.authorName}
                </span>
                <span>•</span>
                <span>{relativeTime}</span>
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
        {translate($locale, "history.loading")}
      </div>
    {/if}
  </div>

  <div class="border-t border-[#1f2328] bg-[#24292f] px-4 py-3">
    <div class="text-xs text-slate-400">
      {translate($locale, "history.showingRange", {
        start: pagination.showingStart,
        end: pagination.showingEnd,
        total: pagination.totalCount,
      })}
    </div>

    {#if pagination.totalPages > 1}
      <div class="mt-3 flex items-center gap-2">
        <button
          type="button"
          class="flex h-8 w-8 items-center justify-center rounded-md border border-transparent text-slate-400 transition hover:border-[#444c56] hover:bg-[#373e47] hover:text-slate-200 disabled:cursor-not-allowed disabled:opacity-40"
          aria-label={translate($locale, "history.previousPage")}
          disabled={!pagination.canPrevious || loading}
          on:click={() => changePage(pagination.pageIndex - 1)}
        >
          ‹
        </button>

        {#each pagination.buttons as button}
          {#if button.kind === "ellipsis"}
            <span class="px-1 text-xs text-slate-500" aria-hidden="true">
              {button.label}
            </span>
          {:else}
            <button
              type="button"
              class={`h-8 min-w-8 rounded-md px-2 text-sm font-semibold transition ${
                button.active
                  ? "bg-[#347dff]/24 text-[#cae8ff] shadow-sm shadow-[#2f81f7]/20"
                  : "bg-[#2d333b] text-slate-300 hover:bg-[#373e47] hover:text-slate-100"
              }`}
              aria-label={translate($locale, "history.pageLabel", {
                page: button.pageIndex + 1,
              })}
              aria-current={button.active ? "page" : undefined}
              disabled={button.active || loading}
              on:click={() => changePage(button.pageIndex)}
            >
              {button.label}
            </button>
          {/if}
        {/each}

        <button
          type="button"
          class="flex h-8 w-8 items-center justify-center rounded-md border border-transparent text-slate-400 transition hover:border-[#444c56] hover:bg-[#373e47] hover:text-slate-200 disabled:cursor-not-allowed disabled:opacity-40"
          aria-label={translate($locale, "history.nextPage")}
          disabled={!pagination.canNext || loading}
          on:click={() => changePage(pagination.pageIndex + 1)}
        >
          ›
        </button>
      </div>
    {/if}
  </div>
</div>
