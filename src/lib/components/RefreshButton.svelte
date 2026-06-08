<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";

  export let enabled = false;
  export let loading = false;

  const dispatch = createEventDispatcher<{ refresh: void }>();

  $: label = loading
    ? translate($locale, "common.refreshing")
    : translate($locale, "common.refresh");

  function refresh() {
    if (!enabled) {
      return;
    }

    dispatch("refresh");
  }
</script>

<button
  type="button"
  class={`mt-[15px] flex min-h-[48px] w-full items-center justify-center transition ${
    enabled
      ? "tg-control tg-focus-ring"
      : "cursor-not-allowed rounded-tg-control border border-tg-border-soft/50 bg-white/[0.025] text-tg-text-muted opacity-75"
  }`}
  disabled={!enabled}
  aria-label={label}
  aria-busy={loading}
  title={label}
  on:click={refresh}
>
  <svg
    viewBox="0 0 16 16"
    class={`h-4 w-4 fill-current ${loading ? "animate-spin" : ""}`}
    aria-hidden="true"
  >
    <path
      d="M1.705 8a6.5 6.5 0 0 1 11.39-4.273V1.75a.75.75 0 0 1 1.5 0V5.5a.75.75 0 0 1-.75.75h-3.75a.75.75 0 0 1 0-1.5h1.962A5 5 0 1 0 13 8a.75.75 0 0 1 1.5 0A6.5 6.5 0 1 1 1.705 8Z"
    ></path>
  </svg>
</button>
