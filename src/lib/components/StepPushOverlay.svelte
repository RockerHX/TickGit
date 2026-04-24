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
      class="w-full max-w-xl rounded-2xl border border-slate-700 bg-slate-950/95 px-4 py-4 shadow-2xl shadow-slate-950/70"
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
          class={`rounded-full px-3 py-1 text-xs font-medium ${
            state.status === "failed"
              ? "bg-rose-500/15 text-rose-200"
              : state.status === "finished"
                ? "bg-emerald-500/15 text-emerald-200"
                : "bg-sky-500/15 text-sky-200"
          }`}
        >
          {Math.round(progress)}%
        </div>
      </div>

      <div class="mt-4 h-2 overflow-hidden rounded-full bg-slate-800">
        <div
          class={`h-full rounded-full transition-all duration-300 ${
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
