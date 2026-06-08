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

<div class="mt-1.5 space-y-1">
  <div class="relative">
    <span
      class="pointer-events-none absolute left-2 top-1/2 -translate-y-1/2 text-tg-text-muted"
      aria-hidden="true"
    >
      <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
        <path
          d="M10.68 11.74a6 6 0 1 1 1.06-1.06l3.04 3.04a.75.75 0 0 1-1.06 1.06l-3.04-3.04ZM11 7a4 4 0 1 0-8 0 4 4 0 0 0 8 0Z"
        ></path>
      </svg>
    </span>
    <input
      bind:this={searchInput}
      class="h-7 w-full rounded-md border border-tg-border-soft bg-white/[0.025] px-7 pr-10 text-[10px] font-medium text-tg-text-primary outline-none transition placeholder:text-tg-text-muted/85 focus:border-tg-blue-soft/60 focus:bg-tg-bg-app/80"
      placeholder={translate($locale, "history.commitSearch")}
      aria-label={translate($locale, "history.commitSearch")}
      value={filters.query ?? ""}
      on:input={(event) => updateFilter("query", event.currentTarget.value)}
    />
    <span
      class="pointer-events-none absolute right-1 top-1/2 inline-flex h-4 -translate-y-1/2 items-center rounded border border-tg-border-soft bg-tg-bg-card/70 px-1 text-[8px] font-semibold leading-none text-tg-text-secondary/75"
      aria-label={translate($locale, "history.searchShortcut")}
    >
      ⌘K
    </span>
  </div>

  <div class="flex items-center gap-1.5 overflow-hidden">
    {#each filterButtons as filter}
      <button
        type="button"
        class={`inline-flex h-6 shrink-0 items-center justify-center gap-1.5 rounded-md border px-2 text-[10px] font-medium leading-none transition ${
          isActive(filter.key) || expandedFilter === filter.key
            ? "border-tg-blue-soft/45 bg-tg-blue/15 text-sky-100"
            : "border-tg-border-soft bg-white/[0.02] text-tg-text-secondary/85 hover:border-tg-blue-soft/35 hover:bg-white/[0.045] hover:text-tg-text-primary"
        }`}
        aria-controls="history-expanded-filters"
        aria-expanded={shouldShowInput(filter.key)}
        aria-pressed={isActive(filter.key)}
        on:click={() => toggleFilter(filter.key)}
      >
        {#if filter.key === "author"}
          <svg
            viewBox="0 0 16 16"
            class="h-3 w-3 shrink-0 fill-current"
            aria-hidden="true"
          >
            <path
              d="M8 8.25a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Zm0-1.5a2 2 0 1 1 0-4 2 2 0 0 1 0 4Zm-5.75 7a5.75 5.75 0 0 1 11.5 0 .75.75 0 0 1-1.5 0 4.25 4.25 0 0 0-8.5 0 .75.75 0 0 1-1.5 0Z"
            ></path>
          </svg>
        {:else if filter.key === "filePath"}
          <svg
            viewBox="0 0 16 16"
            class="h-3 w-3 shrink-0 fill-current"
            aria-hidden="true"
          >
            <path
              d="M3.75 2.25A2.25 2.25 0 0 0 1.5 4.5v7A2.25 2.25 0 0 0 3.75 13.75h8.5a2.25 2.25 0 0 0 2.25-2.25v-5A2.25 2.25 0 0 0 12.25 4.25H8.63L7.42 2.86a1.75 1.75 0 0 0-1.32-.61H3.75Zm0 1.5H6.1c.07 0 .14.03.18.08l1.43 1.64c.14.17.35.27.57.27h3.97c.41 0 .75.34.75.75v5c0 .41-.34.75-.75.75h-8.5A.75.75 0 0 1 3 11.5v-7c0-.41.34-.75.75-.75Z"
            ></path>
          </svg>
        {:else if filter.key === "message"}
          <svg
            viewBox="0 0 16 16"
            class="h-3 w-3 shrink-0 fill-current"
            aria-hidden="true"
          >
            <path
              d="M2.75 2.5A2.25 2.25 0 0 0 .5 4.75v4.5a2.25 2.25 0 0 0 2.25 2.25h1.38l2.34 2.06a.75.75 0 0 0 1.25-.56V11.5h5.53a2.25 2.25 0 0 0 2.25-2.25v-4.5a2.25 2.25 0 0 0-2.25-2.25H2.75Zm0 1.5h10.5c.41 0 .75.34.75.75v4.5c0 .41-.34.75-.75.75H6.97a.75.75 0 0 0-.75.75v.6L4.9 10.18a.75.75 0 0 0-.5-.18H2.75A.75.75 0 0 1 2 9.25v-4.5c0-.41.34-.75.75-.75Z"
            ></path>
          </svg>
        {/if}
        <span>{translate($locale, filter.labelKey)}</span>
      </button>
    {/each}

    <button
      type="button"
      class={`inline-flex h-6 w-7 shrink-0 items-center justify-center rounded-md border text-tg-text-secondary transition ${
        expandedFilter === "all"
          ? "border-tg-blue-soft/45 bg-tg-blue/15 text-sky-100"
          : "border-tg-border-soft bg-white/[0.02] text-tg-text-secondary/85 hover:border-tg-blue-soft/35 hover:bg-white/[0.045] hover:text-tg-text-primary"
      }`}
      title={translate($locale, "history.moreFilters")}
      aria-label={translate($locale, "history.moreFilters")}
      aria-controls="history-expanded-filters"
      aria-expanded={expandedFilter === "all"}
      aria-pressed={expandedFilter === "all"}
      on:click={toggleAllFilters}
    >
      <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current" aria-hidden="true">
        <path
          d="M3.25 3.5a2 2 0 0 1 3.87-.75h5.13a.75.75 0 0 1 0 1.5H7.12a2 2 0 0 1-3.87-.75Zm2 0a.5.5 0 1 0-1 0 .5.5 0 0 0 1 0Zm3.63 4.5a2 2 0 0 1 3.87-.75h.5a.75.75 0 0 1 0 1.5h-.5a2 2 0 0 1-3.87-.75Zm2 0a.5.5 0 1 0-1 0 .5.5 0 0 0 1 0ZM2.75 7.25h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1 0-1.5Zm.5 5.25a2 2 0 0 1 3.87-.75h5.13a.75.75 0 0 1 0 1.5H7.12a2 2 0 0 1-3.87-.75Zm2 0a.5.5 0 1 0-1 0 .5.5 0 0 0 1 0Z"
        ></path>
      </svg>
    </button>
  </div>

  {#if expandedFilter}
    <div
      id="history-expanded-filters"
      class="space-y-1 rounded-md border border-tg-border-strong bg-tg-bg-panel p-1"
    >
      {#each filterButtons as filter}
        {#if shouldShowInput(filter.key)}
          <label class="block">
            <span
              class="mb-0.5 block text-[8px] font-semibold uppercase tracking-[0.14em] text-tg-text-muted"
            >
              {translate($locale, filter.labelKey)}
            </span>
            <input
              class="h-6 w-full rounded-md border border-tg-border-strong bg-tg-bg-app px-2 text-[10px] text-tg-text-primary outline-none transition placeholder:text-tg-text-muted focus:border-tg-blue-soft/70"
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
