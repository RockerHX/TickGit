<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";

  export let open = false;
  export let loading = false;
  export let branch = "";
  export let upstream = "";

  const dispatch = createEventDispatcher<{
    confirm: void;
    cancel: void;
  }>();
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-tg-bg-app/75 p-4 backdrop-blur-sm"
  >
    <div
      class="tg-panel w-full max-w-xl rounded-tg-card shadow-xl shadow-black/40"
      role="dialog"
      aria-modal="true"
      aria-labelledby="rebase-step-push-title"
    >
      <div class="border-b border-tg-border-soft px-5 py-4">
        <h2
          id="rebase-step-push-title"
          class="text-base font-semibold text-white"
        >
          {translate($locale, "rebase.confirmTitle")}
        </h2>
        <p class="mt-1 text-xs text-tg-text-secondary/80">
          {translate($locale, "rebase.target", { branch, upstream })}
        </p>
      </div>

      <div class="space-y-3 px-5 py-4 text-sm text-tg-text-secondary">
        <p>{translate($locale, "rebase.confirmDescription")}</p>
        <div
          class="rounded-sm border border-amber-400/25 bg-amber-400/10 px-4 py-3 text-xs leading-5 text-amber-100"
        >
          {translate($locale, "rebase.historyWarning")}
        </div>
        <p class="text-xs leading-5 text-tg-text-secondary/80">
          {translate($locale, "rebase.backupAndConflict")}
        </p>
      </div>

      <div
        class="flex justify-end gap-2 border-t border-tg-border-soft px-5 py-4"
      >
        <button
          type="button"
          class="tg-control tg-focus-ring px-3 py-2 text-sm font-semibold"
          disabled={loading}
          on:click={() => dispatch("cancel")}
        >
          {translate($locale, "common.cancel")}
        </button>
        <button
          type="button"
          class="tg-focus-ring rounded-tg-control border border-tg-blue bg-tg-blue px-3 py-2 text-sm font-semibold text-white transition hover:bg-tg-blue-soft disabled:cursor-not-allowed disabled:border-tg-border-soft disabled:bg-tg-bg-elevated disabled:text-tg-text-muted"
          disabled={loading}
          on:click={() => dispatch("confirm")}
        >
          {loading
            ? translate($locale, "rebase.running")
            : translate($locale, "rebase.confirm")}
        </button>
      </div>
    </div>
  </div>
{/if}
