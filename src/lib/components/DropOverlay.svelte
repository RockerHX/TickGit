<script lang="ts">
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
          Drop Repository
        </div>
        <div class="mt-3 text-2xl font-semibold text-white">
          Drop a local Git repository here
        </div>
        <div class="mt-2 text-sm text-slate-400">
          TickGit will verify the folder and add it to your repository list
        </div>
      </div>
    </div>
  </div>
{/if}
