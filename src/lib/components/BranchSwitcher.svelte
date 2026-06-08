<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import BranchCard from "$lib/components/BranchCard.svelte";

  export let branches: string[] = [];
  export let currentBranch: string | null = null;
  export let upstream: string | null = null;
  export let disabled = false;
  export let disabledReason: string | null = null;

  const dispatch = createEventDispatcher<{ change: { branch: string } }>();

  let open = false;
  let filterText = "";
  let container: HTMLDivElement | null = null;

  $: normalizedFilter = filterText.trim().toLowerCase();
  $: filteredBranches = branches.filter((branch) =>
    normalizedFilter.length === 0
      ? true
      : branch.toLowerCase().includes(normalizedFilter),
  );

  function toggleOpen() {
    if (disabled || branches.length === 0) {
      return;
    }

    open = !open;
    if (!open) {
      filterText = "";
    }
  }

  function close() {
    open = false;
    filterText = "";
  }

  function selectBranch(branch: string) {
    close();

    if (branch === currentBranch) {
      return;
    }

    dispatch("change", { branch });
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
  <BranchCard
    {currentBranch}
    {upstream}
    {disabled}
    {disabledReason}
    selectable={branches.length > 0}
    {open}
    on:toggle={toggleOpen}
  />

  {#if open}
    <div
      class="tg-panel absolute left-0 right-0 top-full z-30 mt-2 overflow-hidden rounded-tg-card shadow-[0_18px_44px_rgba(0,0,0,0.42)]"
    >
      <div class="border-b border-tg-border-soft px-3.5 pb-2.5 pt-3">
        <div
          class="text-[11px] font-semibold uppercase tracking-[0.16em] text-tg-text-muted"
        >
          {translate($locale, "branch.switch")}
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
            placeholder={translate($locale, "branch.filter")}
            bind:value={filterText}
          />
        </label>
      </div>

      <div class="max-h-[280px] overflow-y-auto px-2 py-2">
        {#if filteredBranches.length === 0}
          <div class="px-3 py-6 text-center text-xs text-tg-text-muted">
            {translate($locale, "branch.noneFound")}
          </div>
        {:else}
          {#each filteredBranches as branch (branch)}
            <button
              class={`flex w-full items-center gap-2.5 rounded-md px-3 py-2 text-left transition ${
                branch === currentBranch
                  ? "bg-tg-blue/80 text-white"
                  : "text-tg-text-primary hover:bg-white/[0.05]"
              }`}
              type="button"
              on:click={() => selectBranch(branch)}
            >
              <span class="flex w-4 shrink-0 items-center justify-center">
                {#if branch === currentBranch}
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

              <span
                class="min-w-0 flex-1 truncate text-sm font-medium leading-5"
              >
                {branch}
              </span>
            </button>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>
