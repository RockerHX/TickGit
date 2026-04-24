<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { RepositorySummary } from "$lib/types";
  import { formatRelativeDate } from "$lib/utils";

  export let repositories: RepositorySummary[] = [];
  export let currentPath: string | null = null;

  const dispatch = createEventDispatcher<{ change: { path: string } }>();

  function handleChange(event: Event) {
    const target = event.currentTarget as HTMLSelectElement;
    dispatch("change", { path: target.value });
  }
</script>

<div class="relative min-w-[320px]">
  <select
    class="h-9 w-full appearance-none rounded-sm border border-[#444c56] bg-[#2d333b] px-3 pr-9 text-sm text-[#f0f6fc] outline-none transition focus:border-[#539bf5]"
    value={currentPath ?? ""}
    on:change={handleChange}
  >
    <option value="" disabled>Select repository</option>
    {#each repositories as repository}
      <option value={repository.path}>
        {repository.name} · {formatRelativeDate(repository.lastOpenedAt)}
      </option>
    {/each}
  </select>

  <div
    class="pointer-events-none absolute inset-y-0 right-0 flex w-9 items-center justify-center border-l border-[#444c56] text-slate-400"
  >
    <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current" aria-hidden="true">
      <path d="M4.47 6.97a.75.75 0 0 1 1.06 0L8 9.44l2.47-2.47a.75.75 0 1 1 1.06 1.06l-3 3a.75.75 0 0 1-1.06 0l-3-3a.75.75 0 0 1 0-1.06Z"></path>
    </svg>
  </div>
</div>
