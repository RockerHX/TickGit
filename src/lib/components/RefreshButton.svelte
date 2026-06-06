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
  class={`mt-[18px] flex min-h-[56px] w-full items-center justify-center rounded-lg border backdrop-blur transition ${
    enabled
      ? "border-white/[0.08] bg-white/[0.045] text-slate-300 hover:border-[#539bf5]/35 hover:bg-white/[0.07] hover:text-[#f0f6fc]"
      : "cursor-not-allowed border-white/[0.05] bg-white/[0.025] text-slate-600 opacity-75"
  }`}
  disabled={!enabled}
  aria-label={label}
  title={label}
  on:click={refresh}
>
  <svg
    viewBox="0 0 16 16"
    class={`h-4.5 w-4.5 fill-current ${loading ? "animate-spin" : ""}`}
    aria-hidden="true"
  >
    <path
      d="M1.705 8a6.5 6.5 0 0 1 11.39-4.273V1.75a.75.75 0 0 1 1.5 0V5.5a.75.75 0 0 1-.75.75h-3.75a.75.75 0 0 1 0-1.5h1.962A5 5 0 1 0 13 8a.75.75 0 0 1 1.5 0A6.5 6.5 0 1 1 1.705 8Z"
    ></path>
  </svg>
</button>
