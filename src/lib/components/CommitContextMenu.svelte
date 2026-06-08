<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import type { CommitListItem } from "$lib/types";

  export let open = false;
  export let x = 0;
  export let y = 0;
  export let commit: CommitListItem | null = null;
  export let disabled = false;
  export let pushToCommitDisabled = false;
  export let stepPushDisabled = false;
  export let pushToCommitReason: string | null = null;
  export let stepPushReason: string | null = null;

  const dispatch = createEventDispatcher<{
    pushToCommit: void;
    stepPush: void;
    close: void;
  }>();
</script>

{#if open && commit}
  <button
    class="fixed inset-0 z-20 cursor-default bg-transparent"
    aria-label={translate($locale, "context.close")}
    on:click={() => dispatch("close")}
  ></button>
  <div
    class="tg-panel fixed z-30 min-w-[244px] overflow-hidden rounded-tg-control p-1 shadow-lg shadow-black/35"
    style={`left:${x}px; top:${y}px;`}
  >
    <button
      class="flex w-full items-start gap-3 rounded-tg-control px-3 py-2 text-left transition hover:bg-white/[0.06] disabled:cursor-not-allowed disabled:opacity-40"
      disabled={disabled || pushToCommitDisabled}
      on:click={() => dispatch("pushToCommit")}
    >
      <div class="mt-0.5 flex h-5 w-5 items-center justify-center text-tg-blue">
        <svg
          viewBox="0 0 16 16"
          class="h-4 w-4 fill-current"
          aria-hidden="true"
        >
          <path
            d="M8 1.75a.75.75 0 0 1 .75.75v7.69l2.22-2.22a.75.75 0 1 1 1.06 1.06l-3.5 3.5a.75.75 0 0 1-1.06 0l-3.5-3.5a.75.75 0 1 1 1.06-1.06l2.22 2.22V2.5A.75.75 0 0 1 8 1.75Z"
          ></path>
        </svg>
      </div>
      <div>
        <div class="text-sm font-medium text-white">
          {translate($locale, "context.pushToCommit")}
        </div>
        <div class="text-xs text-tg-text-secondary/80">
          {pushToCommitReason ??
            translate($locale, "context.pushBranchStateTo", {
              target: commit.shortHash,
            })}
        </div>
      </div>
    </button>

    <button
      class="flex w-full items-start gap-3 rounded-tg-control px-3 py-2 text-left transition hover:bg-white/[0.06] disabled:cursor-not-allowed disabled:opacity-40"
      disabled={disabled || stepPushDisabled}
      on:click={() => dispatch("stepPush")}
    >
      <div
        class="mt-0.5 flex h-5 w-5 items-center justify-center text-[#f2cc60]"
      >
        <svg
          viewBox="0 0 16 16"
          class="h-4 w-4 fill-current"
          aria-hidden="true"
        >
          <path
            d="M8 1.75a.75.75 0 0 1 .75.75v5.69l1.72-1.72a.75.75 0 1 1 1.06 1.06l-3 3a.75.75 0 0 1-1.06 0l-3-3a.75.75 0 1 1 1.06-1.06l1.72 1.72V2.5A.75.75 0 0 1 8 1.75Zm-4.25 10a.75.75 0 0 1 .75-.75h7a.75.75 0 0 1 0 1.5h-7a.75.75 0 0 1-.75-.75Z"
          ></path>
        </svg>
      </div>
      <div>
        <div class="text-sm font-medium text-white">
          {translate($locale, "context.stepPushToCommit")}
        </div>
        <div class="text-xs text-tg-text-secondary/80">
          {stepPushReason ?? translate($locale, "context.stepPushDescription")}
        </div>
      </div>
    </button>
  </div>
{/if}
