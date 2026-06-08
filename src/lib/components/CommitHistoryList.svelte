<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate, translateBranchDisabledReason } from "$lib/i18n";
  import HistoryFilters from "$lib/components/HistoryFilters.svelte";
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

  const dispatch = createEventDispatcher<{
    select: { commit: CommitListItem };
    pageChange: { pageIndex: number };
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

<div class="flex h-full min-h-0 flex-col overflow-hidden bg-tg-bg-panel">
  <div class="border-b border-tg-border-soft px-4 py-3">
    <div class="flex items-center justify-between gap-3">
      <div class="text-sm font-semibold text-tg-text-primary">
        {translate($locale, "history.title")}
      </div>
      {#if activeFilterCount > 0}
        <button
          type="button"
          class="tg-control tg-focus-ring px-2 py-1 text-[11px] font-medium"
          on:click={() => dispatch("clearFilters")}
        >
          {translate($locale, "history.clearFilters", {
            count: activeFilterCount,
          })}
        </button>
      {/if}
    </div>
    <div class="mt-1 text-xs text-tg-text-secondary/80">
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
        class="m-4 rounded-tg-control border border-dashed border-tg-border-strong bg-tg-bg-app px-4 py-10 text-center text-sm text-tg-text-muted"
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
          class={`tg-focus-ring group relative min-h-[92px] w-full overflow-hidden rounded-2xl border px-4 py-3.5 text-left shadow-sm transition duration-150 ${
            isSelected
              ? "border-tg-blue-soft/60 bg-gradient-to-r from-tg-blue/35 via-tg-blue/20 to-tg-bg-panel shadow-[0_14px_34px_rgba(37,99,235,0.2)]"
              : "border-tg-border-soft bg-tg-bg-card/25 shadow-black/10 hover:border-tg-blue/30 hover:bg-tg-bg-elevated/55 hover:shadow-[0_10px_26px_rgba(15,23,42,0.24)]"
          }`}
          on:click={() => dispatch("select", { commit })}
          on:contextmenu={(event) => openMenu(event, commit)}
        >
          {#if isSelected}
            <div
              class="absolute inset-y-2 left-0 w-1.5 rounded-r-full bg-gradient-to-b from-sky-300 via-tg-blue-soft to-tg-blue shadow-[0_0_18px_rgba(96,165,250,0.6)]"
            ></div>
          {/if}

          <div class="flex items-start gap-3">
            <div class="relative mt-0.5 shrink-0">
              {#if !commit.isPushed}
                {#if commit.isSafePushTarget}
                  <span
                    class="absolute -left-1.5 -top-1.5 z-10 flex h-5 w-5 items-center justify-center rounded-full border border-emerald-300/60 bg-gradient-to-br from-emerald-500/25 to-tg-bg-app text-emerald-200 shadow-[0_0_14px_rgba(52,211,153,0.32)] ring-2 ring-tg-bg-app"
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
                    class="absolute -left-1.5 -top-1.5 z-10 flex h-5 w-5 items-center justify-center rounded-full border border-rose-300/60 bg-gradient-to-br from-rose-500/25 to-tg-bg-app text-rose-200 shadow-[0_0_14px_rgba(251,113,133,0.28)] ring-2 ring-tg-bg-app"
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
                    ? "border-tg-blue-soft/60 bg-tg-blue/35 text-sky-100"
                    : "border-tg-border-strong bg-tg-bg-card text-tg-text-secondary group-hover:border-tg-border-strong group-hover:bg-tg-bg-elevated"
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
                      class="min-w-0 flex-1 truncate text-[14px] font-semibold leading-5 text-tg-text-primary"
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
                        ? "bg-tg-blue-soft/20 text-sky-100"
                        : "bg-tg-bg-card text-tg-text-secondary/80 group-hover:text-tg-text-secondary"
                    }`}
                  >
                    {relativeTime}
                  </span>
                  {#if !commit.isPushed}
                    <span
                      class={`h-2.5 w-2.5 rounded-full ring-2 ring-tg-bg-app/90 ${
                        commit.isSafePushTarget
                          ? "bg-emerald-400 shadow-[0_0_10px_rgba(52,211,153,0.55)]"
                          : "bg-rose-400 shadow-[0_0_10px_rgba(251,113,133,0.45)]"
                      }`}
                      title={pushStatusTitle}
                      aria-label={pushStatusTitle}
                    ></span>
                  {:else}
                    <span
                      class="h-2.5 w-2.5 rounded-full bg-tg-text-muted ring-2 ring-tg-bg-app/90"
                      title={pushStatusTitle}
                      aria-label={pushStatusTitle}
                    ></span>
                  {/if}
                </div>
              </div>

              <div
                class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-[12px] text-tg-text-secondary/80"
              >
                <span class="min-w-0 max-w-[9rem] truncate">
                  {commit.authorName}
                </span>
                <span>•</span>
                <span>{relativeTime}</span>
                <span>•</span>
                <span class="font-mono text-tg-text-secondary">
                  {commit.shortHash}
                </span>
              </div>
            </div>
          </div>
        </button>
      {/each}
    </div>

    {#if loading}
      <div class="px-4 py-4 text-center text-xs text-tg-text-secondary/80">
        {translate($locale, "history.loading")}
      </div>
    {/if}
  </div>

  <div class="border-t border-tg-border-soft bg-tg-bg-card px-4 py-3">
    <div class="text-xs text-tg-text-secondary/80">
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
          class="tg-control tg-focus-ring flex h-8 w-8 items-center justify-center border-transparent disabled:cursor-not-allowed disabled:opacity-40"
          aria-label={translate($locale, "history.previousPage")}
          disabled={!pagination.canPrevious || loading}
          on:click={() => changePage(pagination.pageIndex - 1)}
        >
          ‹
        </button>

        {#each pagination.buttons as button}
          {#if button.kind === "ellipsis"}
            <span class="px-1 text-xs text-tg-text-muted" aria-hidden="true">
              {button.label}
            </span>
          {:else}
            <button
              type="button"
              class={`h-8 min-w-8 rounded-md px-2 text-sm font-semibold transition ${
                button.active
                  ? "bg-tg-blue/24 text-sky-100 shadow-sm shadow-tg-blue/20"
                  : "bg-tg-bg-panel text-tg-text-secondary hover:bg-tg-bg-elevated hover:text-tg-text-primary"
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
          class="tg-control tg-focus-ring flex h-8 w-8 items-center justify-center border-transparent disabled:cursor-not-allowed disabled:opacity-40"
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
