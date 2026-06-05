<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import {
    filterRepositories,
    formatRepositoryPath,
    repositoryStatusBadgeLabel,
    repositoryStatusLabel,
    repositoryStatusMessage,
    repositoryStatusTone,
  } from "$lib/tickgit/repositories";
  import type { RepositorySummary } from "$lib/types";

  export let repositories: RepositorySummary[] = [];
  export let currentPath: string | null = null;
  export let managementDisabled = false;

  const dispatch = createEventDispatcher<{
    change: { path: string };
    remove: { path: string };
    relocate: { path: string };
  }>();

  let open = false;
  let filterText = "";
  let container: HTMLDivElement | null = null;

  $: currentRepository =
    repositories.find((repository) => repository.path === currentPath) ?? null;
  $: filteredRepositories = filterRepositories(repositories, filterText);

  function toggleOpen() {
    open = !open;
    if (!open) {
      filterText = "";
    }
  }

  function close() {
    open = false;
    filterText = "";
  }

  function selectRepository(path: string) {
    close();
    dispatch("change", { path });
  }

  function removeRepository(path: string) {
    dispatch("remove", { path });
  }

  function relocateRepository(path: string) {
    dispatch("relocate", { path });
  }

  function handleWindowClick(event: MouseEvent) {
    if (container?.contains(event.target as Node)) {
      return;
    }

    close();
  }
</script>

<svelte:window on:click={handleWindowClick} />

<div class="relative w-full min-w-0" bind:this={container}>
  <button
    class={`flex min-h-[56px] w-full items-center gap-2.5 rounded-lg border px-3 py-2 text-left backdrop-blur transition ${
      open
        ? "border-[#4d7cff]/55 bg-[#1f6feb]/14 shadow-[0_0_0_1px_rgba(83,155,245,0.12)]"
        : "border-white/[0.08] bg-white/[0.04] hover:border-[#539bf5]/30 hover:bg-white/[0.06]"
    }`}
    type="button"
    on:click|stopPropagation={toggleOpen}
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
          {currentRepository?.name ?? translate($locale, "repository.select")}
        </span>
        {#if currentRepository}
          <span
            class={`shrink-0 rounded-full border px-2 py-0.5 text-[8px] font-bold tracking-[0.12em] ${repositoryStatusTone(currentRepository.status)}`}
          >
            {repositoryStatusBadgeLabel(currentRepository.status)}
          </span>
        {/if}
      </span>
      <span class="mt-0.5 block truncate text-[11px] text-slate-400">
        {currentRepository
          ? formatRepositoryPath(currentRepository.path)
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

  {#if open}
    <div
      class="absolute left-0 right-0 top-full z-30 mt-2 overflow-hidden rounded-xl border border-[#4b5563] bg-[#2b3036] shadow-[0_18px_50px_rgba(0,0,0,0.45)]"
    >
      <div class="border-b border-[#373e47] px-5 pb-4 pt-5">
        <div class="text-[0.9rem] font-semibold text-slate-400">
          {translate($locale, "repository.select")}
        </div>
      </div>

      <div class="border-b border-[#373e47] px-5 py-4">
        <label
          class="flex h-11 items-center gap-3 rounded-md border border-[#539bf5] bg-[#24292f] px-3 text-slate-300 shadow-[0_0_0_1px_rgba(83,155,245,0.15)]"
        >
          <svg
            viewBox="0 0 16 16"
            class="h-5 w-5 shrink-0 fill-current"
            aria-hidden="true"
          >
            <path
              d="M10.68 11.74a6 6 0 1 1 1.06-1.06l3.26 3.27a.75.75 0 1 1-1.06 1.06l-3.26-3.27ZM11 6.5a4.5 4.5 0 1 0-9 0 4.5 4.5 0 0 0 9 0Z"
            ></path>
          </svg>
          <input
            class="w-full bg-transparent text-[1rem] text-[#f0f6fc] outline-none placeholder:text-slate-500"
            placeholder={translate($locale, "repository.filter")}
            bind:value={filterText}
          />
        </label>
      </div>

      <div class="max-h-[420px] overflow-y-auto px-2 py-3">
        {#if filteredRepositories.length === 0}
          <div class="px-3 py-8 text-center text-sm text-slate-500">
            {translate($locale, "repository.noneFound")}
          </div>
        {:else}
          {#each filteredRepositories as repository (repository.path)}
            <div
              class={`flex items-stretch gap-2 rounded-md px-2 py-2 transition ${
                repository.path === currentPath
                  ? "bg-[#345fc2] text-white"
                  : "text-[#f0f6fc] hover:bg-[#373e47]/70"
              }`}
            >
              <button
                class="flex min-w-0 flex-1 items-start gap-3 rounded-md px-1 py-1 text-left"
                type="button"
                on:click={() => selectRepository(repository.path)}
              >
                <span
                  class="flex w-5 shrink-0 items-center justify-center pt-1"
                >
                  {#if repository.path === currentPath}
                    <svg
                      viewBox="0 0 16 16"
                      class="h-4.5 w-4.5 fill-current"
                      aria-hidden="true"
                    >
                      <path
                        d="M13.78 4.97a.75.75 0 0 1 0 1.06L7.53 12.28a.75.75 0 0 1-1.06 0L2.22 8.03a.75.75 0 0 1 1.06-1.06L7 10.69l5.72-5.72a.75.75 0 0 1 1.06 0Z"
                      ></path>
                    </svg>
                  {/if}
                </span>

                <span class="min-w-0 flex-1">
                  <span class="flex min-w-0 items-center gap-2">
                    <span
                      class="min-w-0 flex-1 truncate text-[1.05rem] font-semibold"
                    >
                      {repository.name}
                    </span>
                    <span
                      class={`shrink-0 rounded-full border px-2 py-0.5 text-[8px] font-semibold uppercase ${repositoryStatusTone(repository.status)}`}
                    >
                      {repositoryStatusLabel(repository.status, $locale)}
                    </span>
                  </span>
                  <span class="mt-0.5 block truncate text-xs opacity-75">
                    {formatRepositoryPath(repository.path)}
                  </span>
                  {#if repositoryStatusMessage(repository, $locale)}
                    <span class="mt-1 block text-xs text-amber-100/90">
                      {repositoryStatusMessage(repository, $locale)}
                    </span>
                  {/if}
                </span>
              </button>

              <div class="flex shrink-0 flex-col gap-1">
                <button
                  type="button"
                  class="rounded-md border border-[#444c56] bg-[#2d333b] px-2 py-1 text-[11px] font-semibold text-slate-200 transition hover:border-[#539bf5]/50 hover:bg-[#347dff]/15 disabled:cursor-not-allowed disabled:opacity-50"
                  disabled={managementDisabled}
                  on:click={() => relocateRepository(repository.path)}
                >
                  {translate($locale, "repository.relocate")}
                </button>
                <button
                  type="button"
                  class="rounded-md border border-rose-400/30 bg-rose-500/10 px-2 py-1 text-[11px] font-semibold text-rose-100 transition hover:bg-rose-500/18 disabled:cursor-not-allowed disabled:opacity-50"
                  disabled={managementDisabled}
                  on:click={() => removeRepository(repository.path)}
                >
                  {translate($locale, "repository.remove")}
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>
