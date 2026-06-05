<script lang="ts">
  import { locale, translate } from "$lib/i18n";
  export let active = false;
</script>

{#if active}
  <div class="pointer-events-none fixed inset-0 z-40 bg-black/35">
    <div class="flex h-full items-center justify-center p-6">
      <div
        class="rounded-sm border border-dashed border-[#539bf5] bg-[#2d333b] px-10 py-8 text-center shadow-lg shadow-black/30"
      >
        <div
          class="text-sm font-semibold uppercase tracking-[0.3em] text-sky-300"
        >
          {translate($locale, "drop.title")}
        </div>
        <div class="mt-3 text-2xl font-semibold text-white">
          {translate($locale, "drop.heading")}
        </div>
        <div class="mt-2 text-sm text-slate-400">
          {translate($locale, "drop.description")}
        </div>
      </div>
    </div>
  </div>
{/if}
