<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { CommitListItem } from "$lib/types";

  export let open = false;
  export let x = 0;
  export let y = 0;
  export let commit: CommitListItem | null = null;
  export let disabled = false;

  const dispatch = createEventDispatcher<{
    pushToCommit: void;
    stepPush: void;
    close: void;
  }>();
</script>

{#if open && commit}
  <button
    class="fixed inset-0 z-20 cursor-default bg-transparent"
    aria-label="Close context menu"
    on:click={() => dispatch("close")}
  ></button>
  <div
    class="fixed z-30 min-w-[220px] overflow-hidden rounded-2xl border border-slate-700 bg-slate-950/95 p-1 shadow-2xl shadow-slate-950/70"
    style={`left:${x}px; top:${y}px;`}
  >
    <button
      class="flex w-full items-start gap-3 rounded-xl px-3 py-2 text-left transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-40"
      {disabled}
      on:click={() => dispatch("pushToCommit")}
    >
      <div class="mt-1 h-2.5 w-2.5 rounded-full bg-sky-400"></div>
      <div>
        <div class="text-sm font-medium text-white">提交到当前 Commit</div>
        <div class="text-xs text-slate-400">直接推送到 {commit.shortHash}</div>
      </div>
    </button>

    <button
      class="flex w-full items-start gap-3 rounded-xl px-3 py-2 text-left transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-40"
      {disabled}
      on:click={() => dispatch("stepPush")}
    >
      <div class="mt-1 h-2.5 w-2.5 rounded-full bg-amber-400"></div>
      <div>
        <div class="text-sm font-medium text-white">分步提交 Commit</div>
        <div class="text-xs text-slate-400">
          从最早未推送 Commit 逐条推送到这里
        </div>
      </div>
    </button>
  </div>
{/if}
