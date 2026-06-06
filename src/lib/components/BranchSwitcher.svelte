<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";

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
  $: upstreamLabel = upstream ?? translate($locale, "branch.noUpstream");
  $: disabledTitle =
    disabledReason ?? translate($locale, "branch.pushDisabledFallback");
  $: cardLabel = [
    `${translate($locale, "branch.current")}: ${currentBranch ?? translate($locale, "branch.noneSelected")}`,
    upstreamLabel,
    disabled ? disabledTitle : null,
  ]
    .filter(Boolean)
    .join(". ");
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
  <button
    class={`group flex min-h-[60px] w-full items-center gap-3 rounded-xl border px-3.5 py-2.5 text-left backdrop-blur-xl transition duration-200 ${
      disabled
        ? "cursor-not-allowed border-white/[0.06] bg-[#0f172a]/35 text-slate-500 opacity-70"
        : open
          ? "border-[#60a5fa]/45 bg-[#1d4ed8]/16 shadow-[0_0_0_1px_rgba(96,165,250,0.14),0_16px_36px_rgba(15,23,42,0.35)]"
          : "border-white/[0.1] bg-[#0f172a]/55 shadow-[inset_0_1px_0_rgba(255,255,255,0.04)] hover:border-[#60a5fa]/32 hover:bg-[#1e293b]/62 hover:shadow-[0_14px_34px_rgba(15,23,42,0.24)]"
    }`}
    type="button"
    disabled={disabled || branches.length === 0}
    aria-label={cardLabel}
    title={disabled ? disabledTitle : undefined}
    on:click|stopPropagation={toggleOpen}
  >
    <span
      class={`flex h-9.5 w-9.5 shrink-0 items-center justify-center rounded-[10px] border transition ${
        disabled
          ? "border-white/[0.06] bg-[#1f2937] text-slate-500 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)]"
          : "border-[#93c5fd]/25 bg-gradient-to-br from-[#2563eb] via-[#3b82f6] to-[#06b6d4] text-[#f8fafc] shadow-[0_12px_26px_rgba(37,99,235,0.28)] group-hover:shadow-[0_14px_30px_rgba(37,99,235,0.36)]"
      }`}
      aria-hidden="true"
    >
      <svg viewBox="0 0 16 16" class="h-4.5 w-4.5 fill-current drop-shadow">
        <path
          d="M5.75 2a1.75 1.75 0 1 0 1.72 2.06l1.6.64a1.75 1.75 0 0 0 2.16 2.16l.64 1.6a1.75 1.75 0 1 0 1.38-.56 1.73 1.73 0 0 0-.31.03l-.64-1.6a1.75 1.75 0 0 0-2.16-2.16l-1.6-.64A1.75 1.75 0 0 0 5.75 2Zm0 1.5a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Zm4.5 2a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Zm3 4a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Z"
        ></path>
      </svg>
    </span>

    <span class="min-w-0 flex-1">
      <span
        class={`block truncate text-[0.95rem] font-semibold ${
          disabled ? "text-slate-500" : "text-[#f8fafc]"
        }`}
      >
        {currentBranch ?? translate($locale, "branch.noneSelected")}
      </span>
      <span
        class={`mt-1 block truncate text-[11px] ${
          disabled ? "text-slate-600" : "text-slate-400"
        }`}
      >
        {upstreamLabel}
      </span>
    </span>

    <span
      class={`flex h-8 w-7 shrink-0 items-center justify-center border-l transition duration-200 ${
        disabled
          ? "border-white/[0.05] text-slate-600"
          : open
            ? "border-[#60a5fa]/40 text-[#dbeafe]"
            : "border-white/[0.1] text-slate-400 group-hover:text-slate-200"
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
      class="absolute left-0 right-0 top-full z-30 mt-2 overflow-hidden rounded-xl border border-white/[0.12] bg-[#22272e]/95 shadow-[0_18px_44px_rgba(0,0,0,0.42)] backdrop-blur-xl"
    >
      <div class="border-b border-white/[0.08] px-3.5 pb-2.5 pt-3">
        <div
          class="text-[11px] font-semibold uppercase tracking-[0.16em] text-slate-500"
        >
          {translate($locale, "branch.switch")}
        </div>
      </div>

      <div class="border-b border-white/[0.08] px-3.5 py-3">
        <label
          class="flex h-9 items-center gap-2.5 rounded-md border border-white/[0.12] bg-[#1f242b] px-2.5 text-slate-400 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)] focus-within:border-[#539bf5]/70 focus-within:shadow-[0_0_0_1px_rgba(83,155,245,0.16)]"
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
            class="w-full bg-transparent text-sm text-[#f0f6fc] outline-none placeholder:text-slate-500"
            placeholder={translate($locale, "branch.filter")}
            bind:value={filterText}
          />
        </label>
      </div>

      <div class="max-h-[280px] overflow-y-auto px-2 py-2">
        {#if filteredBranches.length === 0}
          <div class="px-3 py-6 text-center text-xs text-slate-500">
            {translate($locale, "branch.noneFound")}
          </div>
        {:else}
          {#each filteredBranches as branch (branch)}
            <button
              class={`flex w-full items-center gap-2.5 rounded-md px-3 py-2 text-left transition ${
                branch === currentBranch
                  ? "bg-[#345fc2]/90 text-white"
                  : "text-[#f0f6fc] hover:bg-white/[0.05]"
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
