<script lang="ts">
  import { locale, translate } from "$lib/i18n";
  export let active = false;
</script>

{#if active}
  <div class="pointer-events-none fixed inset-0 z-40 bg-black/35">
    <div class="flex h-full items-center justify-center p-6">
      <div
        class="rounded-tg-card border border-dashed border-tg-blue-soft bg-tg-bg-card px-10 py-8 text-center shadow-lg shadow-black/30 backdrop-blur-xl"
      >
        <div
          class="text-sm font-semibold uppercase tracking-[0.3em] text-sky-300"
        >
          {translate($locale, "drop.title")}
        </div>
        <div class="mt-3 text-2xl font-semibold text-white">
          {translate($locale, "drop.heading")}
        </div>
        <div class="mt-2 text-sm text-tg-text-secondary/80">
          {translate($locale, "drop.description")}
        </div>
      </div>
    </div>
  </div>
{/if}
