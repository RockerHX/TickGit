<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import {
    formatRepositoryPath,
    repositoryStatusBadgeLabel,
    repositoryStatusLabel,
    repositoryStatusMessage,
    repositoryStatusTone,
  } from "$lib/tickgit/repositories";
  import type { RepositorySummary } from "$lib/types";

  export let repository: RepositorySummary | null = null;
  export let open = false;

  const dispatch = createEventDispatcher<{ toggle: void }>();

  $: displayPath = repository ? formatRepositoryPath(repository.path) : null;
  $: statusLabel = repository
    ? repositoryStatusLabel(repository.status, $locale)
    : null;
  $: unavailableMessage = repository
    ? repositoryStatusMessage(repository, $locale)
    : null;
  $: cardLabel = repository
    ? `${translate($locale, "repository.current")}: ${repository.name}. ${statusLabel}. ${displayPath}`
    : translate($locale, "repository.select");
  $: cardTitle = repository
    ? unavailableMessage
      ? `${repository.name} · ${displayPath} · ${unavailableMessage}`
      : `${repository.name} · ${displayPath}`
    : translate($locale, "repository.select");

  function toggle() {
    dispatch("toggle");
  }
</script>

<button
  class={`group tg-card flex min-h-[60px] w-full items-center gap-3 px-3.5 py-2.5 text-left transition duration-200 ${
    open
      ? "border-tg-blue-soft/45 bg-tg-blue/15 shadow-tg-glow"
      : "tg-card-hover"
  }`}
  type="button"
  aria-expanded={open}
  aria-haspopup="dialog"
  aria-label={cardLabel}
  title={cardTitle}
  on:click|stopPropagation={toggle}
>
  <span
    class="tg-icon-tile flex h-9.5 w-9.5 shrink-0 items-center justify-center transition group-hover:shadow-[0_14px_30px_rgba(37,99,235,0.36)]"
    aria-hidden="true"
  >
    <svg viewBox="0 0 16 16" class="h-4.5 w-4.5 fill-current drop-shadow">
      <path
        d="M8 1.75c-3.1 0-5.5.95-5.5 2.25v8c0 1.3 2.4 2.25 5.5 2.25s5.5-.95 5.5-2.25V4c0-1.3-2.4-2.25-5.5-2.25Zm0 1.5c2.35 0 3.55.53 3.9.75-.35.22-1.55.75-3.9.75S4.45 4.22 4.1 4c.35-.22 1.55-.75 3.9-.75Zm4 3.02V8c-.67.42-2.08.75-4 .75S4.67 8.42 4 8V6.27c1.01.45 2.46.73 4 .73s2.99-.28 4-.73Zm0 3V12c-.35.22-1.55.75-4 .75s-3.55-.53-3.9-.75V9.27c1.01.45 2.46.73 4 .73s2.99-.28 4-.73Z"
      ></path>
    </svg>
  </span>

  <span class="min-w-0 flex-1">
    <span class="flex min-w-0 items-center gap-2">
      <span
        class="min-w-0 flex-1 truncate text-[0.95rem] font-semibold text-tg-text-primary"
      >
        {repository?.name ?? translate($locale, "repository.select")}
      </span>
      {#if repository}
        <span
          class={`shrink-0 rounded-full border px-2 py-0.5 text-[8px] font-bold tracking-[0.14em] shadow-[inset_0_1px_0_rgba(255,255,255,0.05)] ${repositoryStatusTone(repository.status)}`}
        >
          {repositoryStatusBadgeLabel(repository.status)}
        </span>
      {/if}
    </span>
    <span class="mt-1 block truncate text-[11px] text-tg-text-secondary/80">
      {displayPath ?? translate($locale, "repository.select")}
    </span>
  </span>

  <span
    class={`flex h-8 w-7 shrink-0 items-center justify-center border-l transition duration-200 ${
      open
        ? "border-tg-blue-soft/40 text-sky-100"
        : "border-tg-border-soft text-tg-text-secondary/80 group-hover:text-tg-text-primary"
    }`}
  >
    <svg
      viewBox="0 0 16 16"
      class={`h-3.5 w-3.5 fill-current transition ${open ? "rotate-180" : ""}`}
      aria-hidden="true"
    >
      <path
        d="M4.47 6.97a.75.75 0 0 1 1.06 0L8 9.44l2.47-2.47a.75.75 0 1 1 1.06 1.06l-3 3a.75.75 0 0 1-1.06 0l-3-3a.75.75 0 0 1 0-1.06Z"
      ></path>
    </svg>
  </span>
</button>
