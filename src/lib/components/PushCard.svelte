<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let title: string;
  export let subtitle: string;
  export let aheadCount: number;
  export let enabled: boolean;
  export let loading: boolean;
  export let blockedReason: string | null;
  export let ariaLabel: string;

  const dispatch = createEventDispatcher<{ push: void }>();

  $: active = enabled || loading;

  function push() {
    if (!enabled) {
      return;
    }

    dispatch("push");
  }
</script>

<button
  class={`group tg-card mt-[18px] flex min-h-[60px] min-w-0 items-center gap-3 px-3.5 py-2.5 text-left transition duration-200 ${
    active
      ? "tg-card-hover text-tg-text-primary"
      : "cursor-not-allowed border-tg-border-soft/60 bg-tg-bg-panel/50 text-tg-text-muted opacity-70"
  }`}
  type="button"
  disabled={!enabled}
  aria-label={ariaLabel}
  title={blockedReason ?? undefined}
  on:click={push}
>
  <span
    class={`flex h-9.5 w-9.5 shrink-0 items-center justify-center rounded-full border transition ${
      active
        ? "border-tg-blue-soft/25 bg-tg-blue/20 text-sky-100 shadow-[0_12px_26px_rgba(37,99,235,0.24)] group-hover:shadow-[0_14px_30px_rgba(37,99,235,0.32)]"
        : "border-tg-border-soft bg-tg-bg-elevated/60 text-tg-text-muted shadow-[inset_0_1px_0_rgba(255,255,255,0.03)]"
    }`}
    aria-hidden="true"
  >
    {#if loading}
      <svg viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current">
        <path
          d="M8 1.5a6.5 6.5 0 1 0 6.5 6.5.75.75 0 0 0-1.5 0 5 5 0 1 1-1.46-3.54.75.75 0 1 0 1.06-1.06A6.48 6.48 0 0 0 8 1.5Z"
        ></path>
      </svg>
    {:else}
      <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
        <path
          d="M8 14.25a.75.75 0 0 1-.75-.75V5.81L5.03 8.03a.75.75 0 0 1-1.06-1.06l3.5-3.5a.75.75 0 0 1 1.06 0l3.5 3.5a.75.75 0 1 1-1.06 1.06L8.75 5.81v7.69a.75.75 0 0 1-.75.75Z"
        ></path>
      </svg>
    {/if}
  </span>

  <span class="min-w-0 flex-1">
    <span
      class={`block truncate text-[0.95rem] font-semibold ${
        active ? "text-tg-text-primary" : "text-tg-text-muted"
      }`}
    >
      {title}
    </span>
    <span
      class={`mt-1 block truncate text-[11px] ${
        active ? "text-tg-text-secondary/80" : "text-tg-text-muted/70"
      }`}
    >
      {subtitle}
    </span>
  </span>

  <span
    class={`flex shrink-0 items-center gap-1 rounded-full px-2.5 py-0.5 text-[11px] font-bold tracking-[0.06em] transition ${
      enabled
        ? "bg-tg-blue text-white shadow-[0_8px_22px_rgba(59,130,246,0.3)] group-hover:shadow-[0_10px_26px_rgba(59,130,246,0.38)]"
        : "bg-tg-text-muted/35 text-tg-text-secondary/80"
    }`}
  >
    <span>{aheadCount}</span>
    <span aria-hidden="true">↑</span>
  </span>
</button>
