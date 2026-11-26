<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";

  export let enabled = false;
  export let rebaseEnabled = false;
  export let rebaseNeeded = false;
  export let rebaseReason = "";

  const dispatch = createEventDispatcher<{ stash: void; rebase: void }>();
</script>

<div
  class="grid grid-cols-2 gap-3 border-t border-tg-border-soft bg-tg-bg-app/40 px-5 py-2.5"
>
  <button
    type="button"
    class="tg-focus-ring group flex min-w-0 items-center gap-3 rounded-xl border border-sky-400/30 bg-sky-400/[0.07] px-3.5 py-2.5 text-left transition hover:border-sky-300/60 hover:bg-sky-400/15 disabled:cursor-not-allowed disabled:opacity-45"
    disabled={!enabled}
    on:click={() => dispatch("stash")}
  >
    <span
      class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-sky-400/15 text-sky-200"
    >
      <svg
        viewBox="0 0 24 24"
        class="h-5 w-5 fill-none stroke-current"
        stroke-width="1.7"
        aria-hidden="true"
      >
        <path
          d="M4 9h16v11H4zM3 4h18v5H3zM9 13h6"
          stroke-linejoin="round"
          stroke-linecap="round"
        />
      </svg>
    </span>
    <span class="min-w-0 flex-1">
      <span class="block text-[13px] font-semibold text-sky-100"
        >{translate($locale, "stash.action")}</span
      >
      <span class="mt-0.5 block truncate text-[10px] text-tg-text-secondary"
        >{translate($locale, "stash.subtitle")}</span
      >
    </span>
    <span class="text-lg text-sky-200/70" aria-hidden="true">›</span>
  </button>

  <button
    type="button"
    class={`tg-focus-ring flex min-w-0 items-center gap-3 rounded-xl border px-3.5 py-2.5 text-left transition disabled:cursor-not-allowed ${rebaseNeeded ? "border-amber-400/50 bg-amber-400/10 hover:border-amber-300 hover:bg-amber-400/15" : "border-tg-border-soft bg-tg-bg-card/35 disabled:opacity-55"}`}
    disabled={!rebaseEnabled}
    title={rebaseReason}
    on:click={() => dispatch("rebase")}
  >
    <span
      class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-amber-400/15 text-amber-200"
    >
      <svg
        viewBox="0 0 24 24"
        class="h-5 w-5 fill-none stroke-current"
        stroke-width="1.7"
        aria-hidden="true"
      >
        <circle cx="7" cy="5" r="2" /><circle cx="17" cy="5" r="2" /><circle
          cx="17"
          cy="19"
          r="2"
        /><path
          d="M7 7v4a4 4 0 0 0 4 4h2a4 4 0 0 1 4 2M17 7v6"
          stroke-linecap="round"
        />
      </svg>
    </span>
    <span class="min-w-0 flex-1">
      <span class="block text-[13px] font-semibold text-amber-100"
        >{translate($locale, "rebase.shortAction")}</span
      >
      <span class="mt-0.5 block truncate text-[10px] text-tg-text-secondary"
        >{rebaseReason}</span
      >
    </span>
    {#if rebaseNeeded}
      <span
        class="shrink-0 rounded-full bg-amber-400/20 px-2 py-0.5 text-[9px] font-semibold text-amber-100"
        >{translate($locale, "rebase.recommended")}</span
      >
    {/if}
  </button>
</div>
