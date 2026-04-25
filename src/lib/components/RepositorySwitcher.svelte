<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { RepositorySummary } from "$lib/types";
  import { formatRelativeDate } from "$lib/utils";

  export let repositories: RepositorySummary[] = [];
  export let currentPath: string | null = null;

  const dispatch = createEventDispatcher<{ change: { path: string } }>();

  let open = false;
  let filterText = "";
  let container: HTMLDivElement | null = null;

  $: currentRepository =
    repositories.find((repository) => repository.path === currentPath) ?? null;
  $: normalizedFilter = filterText.trim().toLowerCase();
  $: filteredRepositories = repositories.filter((repository) =>
    normalizedFilter.length === 0
      ? true
      : repository.name.toLowerCase().includes(normalizedFilter) ||
        repository.path.toLowerCase().includes(normalizedFilter),
  );

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

  function handleWindowClick(event: MouseEvent) {
    if (container?.contains(event.target as Node)) {
      return;
    }

    close();
  }
</script>

<svelte:window on:click={handleWindowClick} />

<div class="relative min-w-[320px]" bind:this={container}>
  <button
    class={`flex h-[50px] w-full items-center justify-between rounded-sm border px-4 text-left transition ${
      open
        ? "border-[#539bf5] bg-[#2d333b]"
        : "border-[#444c56] bg-[#2d333b] hover:border-[#6e7681]"
    }`}
    type="button"
    on:click|stopPropagation={toggleOpen}
  >
    <span class="min-w-0 flex-1 pr-4">
      <span class="block truncate text-[1rem] font-semibold text-[#f0f6fc]">
        {currentRepository?.name ?? "Select repository"}
        {#if currentRepository}
          <span class="font-normal text-slate-300">
            · {formatRelativeDate(currentRepository.lastOpenedAt)}
          </span>
        {/if}
      </span>
    </span>

    <span
      class={`flex h-8 w-8 shrink-0 items-center justify-center border-l pl-3 ${
        open ? "border-[#539bf5] text-[#cae8ff]" : "border-[#444c56] text-slate-400"
      }`}
    >
      <svg
        viewBox="0 0 16 16"
        class={`h-4 w-4 fill-current transition ${open ? "rotate-180" : ""}`}
        aria-hidden="true"
      >
        <path d="M4.47 6.97a.75.75 0 0 1 1.06 0L8 9.44l2.47-2.47a.75.75 0 1 1 1.06 1.06l-3 3a.75.75 0 0 1-1.06 0l-3-3a.75.75 0 0 1 0-1.06Z"></path>
      </svg>
    </span>
  </button>

  {#if open}
    <div
      class="absolute left-0 right-0 top-full z-30 mt-2 overflow-hidden rounded-xl border border-[#4b5563] bg-[#2b3036] shadow-[0_18px_50px_rgba(0,0,0,0.45)]"
    >
      <div class="border-b border-[#373e47] px-5 pb-4 pt-5">
        <div class="text-[0.95rem] font-semibold text-slate-400">
          Select repository
        </div>
      </div>

      <div class="border-b border-[#373e47] px-5 py-4">
        <label
          class="flex h-11 items-center gap-3 rounded-md border border-[#539bf5] bg-[#24292f] px-3 text-slate-300 shadow-[0_0_0_1px_rgba(83,155,245,0.15)]"
        >
          <svg viewBox="0 0 16 16" class="h-5 w-5 shrink-0 fill-current" aria-hidden="true">
            <path d="M10.68 11.74a6 6 0 1 1 1.06-1.06l3.26 3.27a.75.75 0 1 1-1.06 1.06l-3.26-3.27ZM11 6.5a4.5 4.5 0 1 0-9 0 4.5 4.5 0 0 0 9 0Z"></path>
          </svg>
          <input
            class="w-full bg-transparent text-[1rem] text-[#f0f6fc] outline-none placeholder:text-slate-500"
            placeholder="Filter"
            bind:value={filterText}
          />
        </label>
      </div>

      <div class="max-h-[420px] overflow-y-auto px-2 py-3">
        {#if filteredRepositories.length === 0}
          <div class="px-3 py-8 text-center text-sm text-slate-500">
            No repositories found
          </div>
        {:else}
          {#each filteredRepositories as repository (repository.path)}
            <button
              class={`flex w-full items-center gap-3 rounded-md px-3 py-3 text-left transition ${
                repository.path === currentPath
                  ? "bg-[#345fc2] text-white"
                  : "text-[#f0f6fc] hover:bg-[#373e47]/70"
              }`}
              type="button"
              on:click={() => selectRepository(repository.path)}
            >
              <span class="flex w-5 shrink-0 items-center justify-center">
                {#if repository.path === currentPath}
                  <svg viewBox="0 0 16 16" class="h-4.5 w-4.5 fill-current" aria-hidden="true">
                    <path d="M13.78 4.97a.75.75 0 0 1 0 1.06L7.53 12.28a.75.75 0 0 1-1.06 0L2.22 8.03a.75.75 0 0 1 1.06-1.06L7 10.69l5.72-5.72a.75.75 0 0 1 1.06 0Z"></path>
                  </svg>
                {/if}
              </span>

              <span class="min-w-0 flex-1 truncate text-[1.05rem] font-semibold">
                {repository.name}
                <span
                  class={`font-normal ${
                    repository.path === currentPath ? "text-white/90" : "text-slate-300"
                  }`}
                >
                  · {formatRelativeDate(repository.lastOpenedAt)}
                </span>
              </span>
            </button>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>
