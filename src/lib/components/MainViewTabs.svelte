<script lang="ts" module>
  export type MainViewId = "history" | "changes";
</script>

<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";

  export let active: MainViewId = "history";

  const tabs: Array<{
    id: MainViewId;
    labelKey: "mainView.history" | "mainView.changes";
  }> = [
    { id: "history", labelKey: "mainView.history" },
    { id: "changes", labelKey: "mainView.changes" },
  ];

  const dispatch = createEventDispatcher<{
    change: { view: MainViewId };
  }>();
</script>

<div
  class="inline-flex overflow-hidden rounded-lg border border-[#2d3650]/80 bg-[#111827]/70 p-0.5 shadow-inner shadow-black/20"
>
  {#each tabs as tab}
    <button
      type="button"
      class={`rounded-md px-4 py-2 text-xs font-semibold transition ${
        active === tab.id
          ? "bg-gradient-to-r from-[#2563eb]/90 to-[#1d4ed8]/80 text-[#dbeafe] shadow-sm shadow-[#2563eb]/20"
          : "text-slate-400 hover:bg-white/[0.04] hover:text-slate-200"
      }`}
      aria-pressed={active === tab.id}
      on:click={() => dispatch("change", { view: tab.id })}
    >
      {translate($locale, tab.labelKey)}
    </button>
  {/each}
</div>
