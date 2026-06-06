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

  function push() {
    if (!enabled) {
      return;
    }

    dispatch("push");
  }
</script>

<button
  class={`mt-[18px] flex min-h-[56px] min-w-0 items-center gap-2.5 rounded-lg border px-3 py-2 text-left backdrop-blur transition ${
    enabled
      ? "border-[#4d7cff]/38 bg-white/[0.04] text-[#f0f6fc] hover:border-[#539bf5]/45 hover:bg-[#1f6feb]/10"
      : "cursor-not-allowed border-white/[0.05] bg-white/[0.025] text-slate-500 opacity-70"
  }`}
  type="button"
  disabled={!enabled}
  aria-label={ariaLabel}
  title={blockedReason ?? undefined}
  on:click={push}
>
  <span
    class={`flex h-8 w-8 shrink-0 items-center justify-center rounded-full border shadow-[0_10px_22px_rgba(47,129,247,0.18)] ${
      enabled
        ? "border-[#539bf5]/35 bg-[#347dff]/18 text-[#cae8ff]"
        : "border-white/[0.06] bg-[#30363d] text-slate-500"
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
    <span class="block truncate text-[0.9rem] font-semibold text-[#f0f6fc]">
      {title}
    </span>
    <span class="mt-0.5 block truncate text-[11px] text-slate-400">
      {subtitle}
    </span>
  </span>

  <span
    class={`flex shrink-0 items-center gap-1 rounded-full px-2.5 py-0.5 text-[11px] font-bold tracking-[0.06em] ${
      enabled
        ? "bg-[#347dff] text-white shadow-[0_8px_22px_rgba(52,125,255,0.3)]"
        : "bg-[#6e7681]/35 text-slate-400"
    }`}
  >
    <span>{aheadCount}</span>
    <span aria-hidden="true">↑</span>
  </span>
</button>
