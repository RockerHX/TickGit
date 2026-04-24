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

<label class="flex min-w-[320px] flex-col gap-1">
  <span
    class="text-[11px] font-semibold uppercase tracking-[0.22em] text-slate-500"
  >
    Current Repository
  </span>

  <div class="relative">
    <select
      class="h-10 w-full appearance-none rounded-md border border-[#3d444d] bg-[#2d333b] px-3 pr-9 text-sm text-[#f0f6fc] outline-none transition focus:border-[#2f81f7]"
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

    <div
      class="pointer-events-none absolute inset-y-0 right-0 flex w-9 items-center justify-center border-l border-[#3d444d] text-sm text-slate-400"
    >
      ▾
    </div>
  </div>
</label>
