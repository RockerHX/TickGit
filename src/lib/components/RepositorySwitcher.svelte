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

<label class="flex min-w-[280px] flex-col gap-1">
  <span
    class="text-[11px] font-semibold uppercase tracking-[0.24em] text-slate-500"
  >
    Repository
  </span>

  <select
    class="rounded-xl border border-slate-700 bg-slate-950/90 px-3 py-2 text-sm text-white outline-none transition focus:border-sky-400"
    value={currentPath ?? ""}
    on:change={handleChange}
  >
    <option value="" disabled>选择仓库</option>
    {#each repositories as repository}
      <option value={repository.path}>
        {repository.name} · {formatRelativeDate(repository.lastOpenedAt)}
      </option>
    {/each}
  </select>
</label>
