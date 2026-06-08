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
  import RepositoryCard from "$lib/components/RepositoryCard.svelte";
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
  <RepositoryCard
    repository={currentRepository}
    {open}
    on:toggle={toggleOpen}
  />

  {#if open}
    <div
      class="tg-panel absolute left-0 right-0 top-full z-30 mt-2 overflow-hidden rounded-xl shadow-[0_18px_44px_rgba(0,0,0,0.42)]"
    >
      <div class="border-b border-tg-border-soft px-3.5 pb-2.5 pt-3">
        <div
          class="text-[11px] font-semibold uppercase tracking-[0.16em] text-tg-text-muted"
        >
          {translate($locale, "repository.select")}
        </div>
      </div>

      <div class="border-b border-tg-border-soft px-3.5 py-3">
        <label
          class="flex h-9 items-center gap-2.5 rounded-tg-control border border-tg-border-soft bg-tg-bg-card px-2.5 text-tg-text-secondary/80 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)] focus-within:border-tg-blue-soft/70 focus-within:shadow-[0_0_0_1px_rgba(96,165,250,0.16)]"
        >
          <svg
            viewBox="0 0 16 16"
            class="h-4 w-4 shrink-0 fill-current"
            aria-hidden="true"
          >
            <path
              d="M10.68 11.74a6 6 0 1 1 1.06-1.06l3.26 3.27a.75.75 0 1 1-1.06 1.06l-3.26-3.27ZM11 6.5a4.5 4.5 0 1 0-9 0 4.5 4.5 0 0 0 9 0Z"
            ></path>
          </svg>
          <input
            class="w-full bg-transparent text-sm text-tg-text-primary outline-none placeholder:text-tg-text-muted"
            placeholder={translate($locale, "repository.filter")}
            bind:value={filterText}
          />
        </label>
      </div>

      <div class="max-h-[280px] overflow-y-auto px-2 py-2">
        {#if filteredRepositories.length === 0}
          <div class="px-3 py-6 text-center text-xs text-tg-text-muted">
            {translate($locale, "repository.noneFound")}
          </div>
        {:else}
          {#each filteredRepositories as repository (repository.path)}
            <div
              class={`flex items-center gap-2 rounded-md px-2 py-1.5 transition ${
                repository.path === currentPath
                  ? "bg-tg-blue/80 text-white"
                  : "text-tg-text-primary hover:bg-white/[0.05]"
              }`}
            >
              <button
                class="flex min-w-0 flex-1 items-start gap-2.5 rounded-md px-1 py-1 text-left"
                type="button"
                on:click={() => selectRepository(repository.path)}
              >
                <span
                  class="flex w-4 shrink-0 items-center justify-center pt-0.5"
                >
                  {#if repository.path === currentPath}
                    <svg
                      viewBox="0 0 16 16"
                      class="h-3.5 w-3.5 fill-current"
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
                      class="min-w-0 flex-1 truncate text-sm font-semibold leading-5"
                    >
                      {repository.name}
                    </span>
                    <span
                      class={`shrink-0 rounded-full border px-1.5 py-0.5 text-[9px] font-semibold uppercase leading-none ${repositoryStatusTone(repository.status)}`}
                    >
                      {repositoryStatusLabel(repository.status, $locale)}
                    </span>
                  </span>
                  <span
                    class="mt-0.5 block truncate text-xs leading-4 text-tg-text-secondary/80"
                  >
                    {formatRepositoryPath(repository.path)}
                  </span>
                  {#if repositoryStatusMessage(repository, $locale)}
                    <span
                      class="mt-0.5 block text-[11px] leading-4 text-amber-100/85"
                    >
                      {repositoryStatusMessage(repository, $locale)}
                    </span>
                  {/if}
                </span>
              </button>

              <div class="flex shrink-0 flex-col gap-1">
                <button
                  type="button"
                  class="rounded border border-tg-border-soft bg-white/[0.035] px-1.5 py-0.5 text-[10px] font-medium leading-4 text-tg-text-secondary/80 transition hover:border-tg-blue-soft/40 hover:bg-tg-blue/12 hover:text-tg-text-secondary disabled:cursor-not-allowed disabled:opacity-45"
                  disabled={managementDisabled}
                  on:click={() => relocateRepository(repository.path)}
                >
                  {translate($locale, "repository.relocate")}
                </button>
                <button
                  type="button"
                  class="rounded border border-rose-400/20 bg-rose-500/[0.06] px-1.5 py-0.5 text-[10px] font-medium leading-4 text-rose-200/75 transition hover:bg-rose-500/12 hover:text-rose-100 disabled:cursor-not-allowed disabled:opacity-45"
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
