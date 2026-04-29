<script lang="ts">
  import type { PushToCommitUiState } from "$lib/types";

  export let state: PushToCommitUiState | null = null;
</script>

{#if state}
  <div class="pointer-events-none fixed inset-x-0 top-0 z-50 flex justify-center p-4">
    <div
      class="w-full max-w-xl rounded-sm border border-[#444c56] bg-[#2d333b] px-4 py-4 shadow-lg shadow-black/35"
    >
      <div class="flex items-center justify-between gap-4">
        <div>
          <div class="text-sm font-semibold text-white">
            {#if state.status === "finished"}
              Push finished
            {:else if state.status === "failed"}
              Push failed
            {:else}
              {state.targetKind === "commit" ? "Uploading commit" : "Pushing branch"}
            {/if}
          </div>
          <div class="mt-1 text-xs text-slate-400">{state.target}</div>
          {#if state.message}
            <div class="mt-1 text-xs text-rose-300">{state.message}</div>
          {/if}
        </div>
        <div
          class={`rounded-sm border px-3 py-1 text-xs font-medium ${
            state.status === "failed"
              ? "border-rose-500/40 bg-rose-500/10 text-rose-200"
              : state.status === "finished"
                ? "border-emerald-500/40 bg-emerald-500/10 text-emerald-200"
                : "border-sky-500/40 bg-sky-500/10 text-sky-200"
          }`}
        >
          {#if state.status === "running"}Uploading{:else if state.status === "finished"}Done{:else}Failed{/if}
        </div>
      </div>
    </div>
  </div>
{/if}
