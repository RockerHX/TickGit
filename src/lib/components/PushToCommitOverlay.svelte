<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import type { PushToCommitUiState } from "$lib/types";

  export let state: PushToCommitUiState | null = null;

  const dispatch = createEventDispatcher<{
    close: void;
  }>();
</script>

{#if state}
  <div
    class="pointer-events-none fixed inset-x-0 top-0 z-50 flex justify-center p-4"
  >
    <div
      class="tg-panel pointer-events-auto w-full max-w-xl rounded-tg-card px-4 py-4 shadow-lg shadow-black/35"
    >
      <div class="flex items-center justify-between gap-4">
        <div class="min-w-0 flex-1">
          <div class="text-sm font-semibold text-white">
            {#if state.status === "finished"}
              {translate($locale, "push.finished")}
            {:else if state.status === "failed"}
              {translate($locale, "push.failedTitle")}
            {:else}
              {state.targetKind === "commit"
                ? translate($locale, "push.uploadingCommit")
                : translate($locale, "push.pushingBranch")}
            {/if}
          </div>
          <div class="mt-1 text-xs text-tg-text-secondary/80">
            {state.target}
          </div>
          {#if state.message}
            <div class="mt-1 text-xs text-rose-300">{state.message}</div>
          {/if}
        </div>
        <div class="flex items-start gap-2">
          <div
            class={`rounded-sm border px-3 py-1 text-xs font-medium ${
              state.status === "failed"
                ? "border-rose-500/40 bg-rose-500/10 text-rose-200"
                : state.status === "finished"
                  ? "border-emerald-500/40 bg-emerald-500/10 text-emerald-200"
                  : "border-sky-500/40 bg-sky-500/10 text-sky-200"
            }`}
          >
            {#if state.status === "running"}{translate(
                $locale,
                "push.uploading",
              )}{:else if state.status === "finished"}{translate(
                $locale,
                "common.done",
              )}{:else}{translate($locale, "common.failed")}{/if}
          </div>

          {#if state.status === "failed"}
            <button
              type="button"
              class="tg-control tg-focus-ring flex h-7 w-7 items-center justify-center"
              aria-label={translate($locale, "push.closeError")}
              on:click={() => dispatch("close")}
            >
              <svg
                viewBox="0 0 16 16"
                class="h-3.5 w-3.5 fill-current"
                aria-hidden="true"
              >
                <path
                  d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 1 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"
                ></path>
              </svg>
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
