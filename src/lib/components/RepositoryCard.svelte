<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import {
    formatRepositoryPath,
    repositoryStatusBadgeLabel,
    repositoryStatusTone,
  } from "$lib/tickgit/repositories";
  import type { RepositorySummary } from "$lib/types";

  export let repository: RepositorySummary | null = null;
  export let open = false;

  const dispatch = createEventDispatcher<{ toggle: void }>();

  function toggle() {
    dispatch("toggle");
  }
</script>

<button
  class={`flex min-h-[56px] w-full items-center gap-2.5 rounded-lg border px-3 py-2 text-left backdrop-blur transition ${
    open
      ? "border-[#4d7cff]/55 bg-[#1f6feb]/14 shadow-[0_0_0_1px_rgba(83,155,245,0.12)]"
      : "border-white/[0.08] bg-white/[0.04] hover:border-[#539bf5]/30 hover:bg-white/[0.06]"
  }`}
  type="button"
  on:click|stopPropagation={toggle}
>
  <span
    class="flex h-8.5 w-8.5 shrink-0 items-center justify-center rounded-lg border border-[#539bf5]/25 bg-gradient-to-br from-[#2f81f7] to-[#5b5cf6] text-[#f0f6fc] shadow-[0_10px_22px_rgba(47,129,247,0.2)]"
    aria-hidden="true"
  >
    <svg viewBox="0 0 16 16" class="h-4.5 w-4.5 fill-current">
      <path
        d="M8 1.75c-3.1 0-5.5.95-5.5 2.25v8c0 1.3 2.4 2.25 5.5 2.25s5.5-.95 5.5-2.25V4c0-1.3-2.4-2.25-5.5-2.25Zm0 1.5c2.35 0 3.55.53 3.9.75-.35.22-1.55.75-3.9.75S4.45 4.22 4.1 4c.35-.22 1.55-.75 3.9-.75Zm4 3.02V8c-.67.42-2.08.75-4 .75S4.67 8.42 4 8V6.27c1.01.45 2.46.73 4 .73s2.99-.28 4-.73Zm0 3V12c-.35.22-1.55.75-4 .75s-3.55-.53-3.9-.75V9.27c1.01.45 2.46.73 4 .73s2.99-.28 4-.73Z"
      ></path>
    </svg>
  </span>

  <span class="min-w-0 flex-1">
    <span class="flex min-w-0 items-center gap-2">
      <span
        class="min-w-0 flex-1 truncate text-[0.9rem] font-semibold text-[#f0f6fc]"
      >
        {repository?.name ?? translate($locale, "repository.select")}
      </span>
      {#if repository}
        <span
          class={`shrink-0 rounded-full border px-2 py-0.5 text-[8px] font-bold tracking-[0.12em] ${repositoryStatusTone(repository.status)}`}
        >
          {repositoryStatusBadgeLabel(repository.status)}
        </span>
      {/if}
    </span>
    <span class="mt-0.5 block truncate text-[11px] text-slate-400">
      {repository
        ? formatRepositoryPath(repository.path)
        : translate($locale, "repository.select")}
    </span>
  </span>

  <span
    class={`flex h-8 w-7 shrink-0 items-center justify-center border-l transition ${
      open
        ? "border-[#539bf5]/40 text-[#cae8ff]"
        : "border-white/[0.08] text-slate-400"
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
