<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import { formatRepositoryPath } from "$lib/tickgit/repositories";
  import type { RepositorySummary } from "$lib/types";

  export let repository: RepositorySummary | null = null;
  export let loading = false;

  const dispatch = createEventDispatcher<{
    confirm: void;
    cancel: void;
  }>();

  $: open = Boolean(repository);
  $: displayPath = repository ? formatRepositoryPath(repository.path) : "";

  function cancel() {
    if (loading) {
      return;
    }

    dispatch("cancel");
  }

  function confirm() {
    if (!repository || loading) {
      return;
    }

    dispatch("confirm");
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      cancel();
    }
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (!open || event.key !== "Escape") {
      return;
    }

    cancel();
  }
</script>

<svelte:window on:keydown={handleWindowKeydown} />

{#if repository}
  <div
    class="fixed inset-0 z-40 flex items-center justify-center bg-tg-bg-app/75 px-4 backdrop-blur-sm"
    role="presentation"
    on:click={handleBackdropClick}
  >
    <div
      class="tg-panel w-full max-w-md overflow-hidden rounded-2xl shadow-[0_28px_90px_rgba(0,0,0,0.55)]"
      role="dialog"
      aria-modal="true"
      aria-labelledby="remove-repository-title"
      aria-describedby="remove-repository-description"
    >
      <div class="border-b border-tg-border-soft px-5 py-4">
        <h2
          id="remove-repository-title"
          class="text-base font-semibold text-tg-text-primary"
        >
          {translate($locale, "repository.removeConfirmTitle")}
        </h2>
        <p
          id="remove-repository-description"
          class="mt-1 text-xs leading-5 text-tg-text-secondary/80"
        >
          {translate($locale, "repository.removeConfirmDescription")}
        </p>
      </div>

      <div class="px-5 py-4">
        <div class="tg-card p-3">
          <div class="truncate text-sm font-semibold text-tg-text-primary">
            {repository.name}
          </div>
          <div
            class="mt-1 truncate font-mono text-xs text-tg-text-secondary/80"
          >
            {displayPath}
          </div>
        </div>
      </div>

      <div
        class="flex justify-end gap-2 border-t border-tg-border-soft px-5 py-4"
      >
        <button
          type="button"
          class="tg-control tg-focus-ring px-3 py-2 text-sm font-semibold disabled:cursor-not-allowed disabled:opacity-50"
          disabled={loading}
          on:click={cancel}
        >
          {translate($locale, "common.cancel")}
        </button>
        <button
          type="button"
          class="rounded-md border border-rose-400/35 bg-rose-500/15 px-3 py-2 text-sm font-semibold text-rose-100 transition hover:bg-rose-500/24 disabled:cursor-not-allowed disabled:opacity-50"
          disabled={loading}
          on:click={confirm}
        >
          {loading
            ? translate($locale, "repository.removing")
            : translate($locale, "repository.removeConfirmAction")}
        </button>
      </div>
    </div>
  </div>
{/if}
