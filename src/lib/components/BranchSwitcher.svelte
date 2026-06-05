<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";

  export let branches: string[] = [];
  export let currentBranch: string | null = null;
  export let upstream: string | null = null;
  export let disabled = false;

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
  <button
    class={`flex min-h-[74px] w-full items-center gap-3 rounded-xl border px-3 py-3 text-left shadow-[0_14px_32px_rgba(0,0,0,0.18)] backdrop-blur transition ${
      disabled
        ? "cursor-not-allowed border-white/[0.05] bg-white/[0.025] text-slate-500 opacity-75"
        : open
          ? "border-[#539bf5]/70 bg-[#1f6feb]/16 shadow-[0_0_0_1px_rgba(83,155,245,0.2),0_18px_42px_rgba(0,0,0,0.28)]"
          : "border-white/[0.08] bg-white/[0.045] hover:border-[#539bf5]/35 hover:bg-white/[0.07]"
    }`}
    type="button"
    disabled={disabled || branches.length === 0}
    title={disabled
      ? translate($locale, "branch.pushDisabledFallback")
      : undefined}
    on:click|stopPropagation={toggleOpen}
  >
    <span
      class={`flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border shadow-[0_12px_24px_rgba(47,129,247,0.18)] ${
        disabled
          ? "border-white/[0.06] bg-[#30363d] text-slate-500"
          : "border-[#539bf5]/30 bg-gradient-to-br from-[#1f6feb] to-[#39c5cf] text-[#f0f6fc]"
      }`}
      aria-hidden="true"
    >
      <svg viewBox="0 0 16 16" class="h-5 w-5 fill-current">
        <path
          d="M5.75 2a1.75 1.75 0 1 0 1.72 2.06l1.6.64a1.75 1.75 0 0 0 2.16 2.16l.64 1.6a1.75 1.75 0 1 0 1.38-.56 1.73 1.73 0 0 0-.31.03l-.64-1.6a1.75 1.75 0 0 0-2.16-2.16l-1.6-.64A1.75 1.75 0 0 0 5.75 2Zm0 1.5a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Zm4.5 2a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Zm3 4a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Z"
        ></path>
      </svg>
    </span>

    <span class="min-w-0 flex-1">
      <span class="block truncate text-[1rem] font-semibold text-[#f0f6fc]">
        {currentBranch ?? translate($locale, "branch.noneSelected")}
      </span>
      <span class="mt-1 block truncate text-xs text-slate-400">
        {upstream ?? translate($locale, "branch.noUpstream")}
      </span>
    </span>

    <span
      class={`flex h-9 w-9 shrink-0 items-center justify-center rounded-full border transition ${
        disabled
          ? "border-white/[0.05] bg-[#0d1117]/20 text-slate-600"
          : open
            ? "border-[#539bf5]/50 bg-[#347dff]/20 text-[#cae8ff]"
            : "border-white/[0.08] bg-[#0d1117]/30 text-slate-400"
      }`}
    >
      <svg
        viewBox="0 0 16 16"
        class={`h-4 w-4 fill-current transition ${open ? "rotate-180" : ""}`}
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
        <div class="text-[0.95rem] font-semibold text-slate-400">
          {translate($locale, "branch.switch")}
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
            placeholder={translate($locale, "branch.filter")}
            bind:value={filterText}
          />
        </label>
      </div>

      <div class="max-h-[420px] overflow-y-auto px-2 py-3">
        {#if filteredBranches.length === 0}
          <div class="px-3 py-8 text-center text-sm text-slate-500">
            {translate($locale, "branch.noneFound")}
          </div>
        {:else}
          {#each filteredBranches as branch (branch)}
            <button
              class={`flex w-full items-center gap-3 rounded-md px-3 py-3 text-left transition ${
                branch === currentBranch
                  ? "bg-[#345fc2] text-white"
                  : "text-[#f0f6fc] hover:bg-[#373e47]/70"
              }`}
              type="button"
              on:click={() => selectBranch(branch)}
            >
              <span class="flex w-5 shrink-0 items-center justify-center">
                {#if branch === currentBranch}
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

              <span
                class="min-w-0 flex-1 truncate text-[1.05rem] font-semibold"
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
