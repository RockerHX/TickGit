<script lang="ts">
  import { onMount } from "svelte";
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import type { CommitHistoryFilters } from "$lib/types";
  import { EMPTY_HISTORY_FILTERS } from "$lib/tickgit/history";

  type FilterKey = "author" | "filePath" | "message";

  export let filters: CommitHistoryFilters = EMPTY_HISTORY_FILTERS;

  const filterButtons: Array<{
    key: FilterKey;
    labelKey: "history.author" | "history.path" | "history.message";
    placeholderKey:
      | "history.authorPlaceholder"
      | "history.pathPlaceholder"
      | "history.messagePlaceholder";
  }> = [
    {
      key: "author",
      labelKey: "history.author",
      placeholderKey: "history.authorPlaceholder",
    },
    {
      key: "filePath",
      labelKey: "history.path",
      placeholderKey: "history.pathPlaceholder",
    },
    {
      key: "message",
      labelKey: "history.message",
      placeholderKey: "history.messagePlaceholder",
    },
  ];

  const dispatch = createEventDispatcher<{
    filterChange: { filters: CommitHistoryFilters };
  }>();

  let expandedFilter: FilterKey | "all" | null = null;
  let searchInput: HTMLInputElement | null = null;

  function filterValue(key: keyof CommitHistoryFilters) {
    return filters[key] ?? "";
  }

  function isActive(key: keyof CommitHistoryFilters) {
    return Boolean(filterValue(key).trim());
  }

  function updateFilter(key: keyof CommitHistoryFilters, value: string) {
    dispatch("filterChange", {
      filters: {
        query: filters.query ?? "",
        author: filters.author ?? "",
        filePath: filters.filePath ?? "",
        message: filters.message ?? "",
        [key]: value,
      },
    });
  }

  function toggleFilter(key: FilterKey) {
    expandedFilter = expandedFilter === key ? null : key;
  }

  function toggleAllFilters() {
    expandedFilter = expandedFilter === "all" ? null : "all";
  }

  function shouldShowInput(key: FilterKey) {
    return expandedFilter === "all" || expandedFilter === key;
  }

  function focusSearch() {
    searchInput?.focus();
    searchInput?.select();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!event.metaKey || event.key.toLowerCase() !== "k") {
      return;
    }

    event.preventDefault();
    focusSearch();
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);

    return () => {
      window.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<div class="mt-3 space-y-2">
  <div class="relative">
    <span
      class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-slate-500"
      aria-hidden="true"
    >
      <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
        <path
          d="M10.68 11.74a6 6 0 1 1 1.06-1.06l3.04 3.04a.75.75 0 0 1-1.06 1.06l-3.04-3.04ZM11 7a4 4 0 1 0-8 0 4 4 0 0 0 8 0Z"
        ></path>
      </svg>
    </span>
    <input
      bind:this={searchInput}
      class="h-10 w-full rounded-lg border border-[#334155]/80 bg-[#111827]/80 px-10 pr-15 text-sm text-[#f8fafc] outline-none transition placeholder:text-slate-500 focus:border-[#60a5fa]/70 focus:bg-[#0f172a]"
      placeholder={translate($locale, "history.commitSearch")}
      value={filters.query ?? ""}
      on:input={(event) => updateFilter("query", event.currentTarget.value)}
    />
    <span
      class="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 rounded-md border border-[#334155] bg-[#1e293b]/80 px-1.5 py-0.5 text-[10px] font-semibold text-slate-400"
      aria-label={translate($locale, "history.searchShortcut")}
    >
      ⌘K
    </span>
  </div>

  <div class="grid grid-cols-[1fr_1fr_1fr_auto] gap-2">
    {#each filterButtons as filter}
      <button
        type="button"
        class={`flex h-9 items-center justify-center gap-1.5 rounded-lg border px-2 text-xs font-semibold transition ${
          isActive(filter.key) || expandedFilter === filter.key
            ? "border-[#3b82f6]/60 bg-[#2563eb]/20 text-[#bfdbfe]"
            : "border-[#334155]/80 bg-[#111827]/70 text-slate-300 hover:border-[#60a5fa]/45 hover:bg-[#1e293b]"
        }`}
        on:click={() => toggleFilter(filter.key)}
      >
        {#if filter.key === "filePath"}
          <span aria-hidden="true">▱</span>
        {:else if filter.key === "message"}
          <span aria-hidden="true">▢</span>
        {:else}
          <span aria-hidden="true">♙</span>
        {/if}
        <span>{translate($locale, filter.labelKey)}</span>
      </button>
    {/each}

    <button
      type="button"
      class={`flex h-9 w-10 items-center justify-center rounded-lg border text-slate-300 transition ${
        expandedFilter === "all"
          ? "border-[#3b82f6]/60 bg-[#2563eb]/20 text-[#bfdbfe]"
          : "border-[#334155]/80 bg-[#111827]/70 hover:border-[#60a5fa]/45 hover:bg-[#1e293b]"
      }`}
      title={translate($locale, "history.moreFilters")}
      aria-label={translate($locale, "history.moreFilters")}
      on:click={toggleAllFilters}
    >
      <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current" aria-hidden="true">
        <path
          d="M2.25 4a.75.75 0 0 1 .75-.75h10a.75.75 0 0 1 0 1.5H3A.75.75 0 0 1 2.25 4Zm2 4a.75.75 0 0 1 .75-.75h6a.75.75 0 0 1 0 1.5H5A.75.75 0 0 1 4.25 8Zm2 4a.75.75 0 0 1 .75-.75h2a.75.75 0 0 1 0 1.5H7A.75.75 0 0 1 6.25 12Z"
        ></path>
      </svg>
    </button>
  </div>

  {#if expandedFilter}
    <div
      class="space-y-2 rounded-lg border border-[#334155]/70 bg-[#0f172a]/80 p-2"
    >
      {#each filterButtons as filter}
        {#if shouldShowInput(filter.key)}
          <label class="block">
            <span
              class="mb-1 block text-[10px] font-semibold uppercase tracking-[0.14em] text-slate-500"
            >
              {translate($locale, filter.labelKey)}
            </span>
            <input
              class="h-9 w-full rounded-md border border-[#334155]/80 bg-[#111827] px-3 text-xs text-[#f8fafc] outline-none transition placeholder:text-slate-500 focus:border-[#60a5fa]/70"
              placeholder={translate($locale, filter.placeholderKey)}
              value={filterValue(filter.key)}
              on:input={(event) =>
                updateFilter(filter.key, event.currentTarget.value)}
            />
          </label>
        {/if}
      {/each}
    </div>
  {/if}
</div>
