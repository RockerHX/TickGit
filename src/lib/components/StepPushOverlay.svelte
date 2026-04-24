<script lang="ts">
  import type { StepPushUiState } from "$lib/types";

  export let state: StepPushUiState | null = null;

  $: progress = state
    ? Math.max(0, Math.min(100, (state.current / state.total) * 100))
    : 0;
</script>

{#if state}
  <div
    class="pointer-events-none fixed inset-x-0 top-0 z-50 flex justify-center p-4"
  >
    <div
      class="w-full max-w-xl rounded-md border border-[#3d444d] bg-[#2d333b] px-4 py-4 shadow-lg shadow-black/30"
    >
      <div class="flex items-center justify-between gap-4">
        <div>
          <div class="text-sm font-semibold text-white">
            {#if state.status === "finished"}
              分步提交已完成
            {:else if state.status === "failed"}
              分步提交失败
            {:else}
              正在分步提交 Commit
            {/if}
          </div>
          <div class="mt-1 text-xs text-slate-400">
            {state.current}/{state.total} · {state.hash}
          </div>
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
          {Math.round(progress)}%
        </div>
      </div>

      <div class="mt-4 h-2 overflow-hidden rounded-sm bg-[#22272e]">
        <div
          class={`h-full rounded-sm transition-all duration-300 ${
            state.status === "failed"
              ? "bg-rose-400"
              : state.status === "finished"
                ? "bg-emerald-400"
                : "bg-sky-400"
          }`}
          style={`width:${progress}%`}
        ></div>
      </div>
    </div>
  </div>
{/if}
