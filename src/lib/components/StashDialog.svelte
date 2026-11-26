<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import type { StashState } from "$lib/types";

  export let open = false;
  export let loading = false;
  export let state: StashState | null = null;
  export let errorMessage: string | null = null;
  export let notice: string | null = null;
  const dispatch = createEventDispatcher<{
    close: void;
    refresh: void;
    create: void;
    restore: { hash: string };
  }>();

  function onKeydown(event: KeyboardEvent) {
    if (open && !loading && event.key === "Escape") dispatch("close");
  }
</script>

<svelte:window on:keydown={onKeydown} />

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-tg-bg-app/75 p-4 backdrop-blur-sm"
  >
    <div
      class="tg-panel flex max-h-[85vh] w-full max-w-xl flex-col overflow-hidden rounded-2xl shadow-2xl"
      role="dialog"
      aria-modal="true"
      aria-labelledby="stash-title"
      aria-describedby="stash-description"
      aria-busy={loading}
    >
      <div class="border-b border-tg-border-soft px-5 py-4">
        <h2
          id="stash-title"
          class="text-base font-semibold text-tg-text-primary"
        >
          {translate($locale, "stash.title")}
        </h2>
        <p
          id="stash-description"
          class="mt-2 text-xs leading-5 text-tg-text-secondary"
        >
          {translate($locale, "stash.description")}
        </p>
        {#if state}<p class="mt-2 break-all font-mono text-xs text-sky-200">
            {state.branch}
          </p>{/if}
      </div>
      <div class="min-h-0 space-y-4 overflow-y-auto px-5 py-4">
        {#if errorMessage}<p
            role="alert"
            class="break-words rounded-lg border border-rose-400/30 bg-rose-400/10 p-3 text-xs leading-5 text-rose-100"
          >
            {errorMessage}
          </p>{/if}
        {#if notice}<p
            role="status"
            class="rounded-lg border border-emerald-400/30 bg-emerald-400/10 p-3 text-xs leading-5 text-emerald-100"
          >
            {notice}
          </p>{/if}
        <div
          class="rounded-xl border border-sky-400/25 bg-sky-400/[0.06] p-3.5"
        >
          <p class="text-xs leading-5 text-tg-text-secondary">
            {state?.hasChanges
              ? translate($locale, "stash.hasChanges")
              : translate($locale, "stash.clean")}
          </p>
          <button
            type="button"
            class="tg-focus-ring mt-3 rounded-lg border border-sky-400/50 bg-sky-500/20 px-4 py-2 text-xs font-semibold text-sky-100 transition hover:bg-sky-500/30 disabled:cursor-not-allowed disabled:opacity-40"
            disabled={loading || !state?.hasChanges}
            on:click={() => dispatch("create")}
            >{translate($locale, "stash.create")}</button
          >
        </div>
        <div>
          <div class="mb-2 flex items-center justify-between gap-3">
            <h3 class="text-xs font-semibold text-tg-text-primary">
              {translate($locale, "stash.saved")}
            </h3>
            <button
              type="button"
              class="tg-control tg-focus-ring px-2 py-1 text-[10px]"
              disabled={loading}
              on:click={() => dispatch("refresh")}
              >{translate($locale, "stash.refresh")}</button
            >
          </div>
          {#if state?.entries.length}
            <ul class="space-y-2">
              {#each state.entries as entry (entry.hash)}
                <li
                  class="rounded-lg border border-tg-border-soft bg-tg-bg-card/40 p-3"
                >
                  <p
                    class="break-words text-xs leading-5 text-tg-text-secondary"
                  >
                    {entry.message}
                  </p>
                  <div class="mt-2 flex items-center justify-between gap-3">
                    <code class="text-[10px] text-tg-text-muted"
                      >{entry.hash.slice(0, 10)}</code
                    >
                    <button
                      type="button"
                      class="tg-control tg-focus-ring px-3 py-1.5 text-xs disabled:cursor-not-allowed disabled:opacity-40"
                      disabled={loading || !state.canRestore}
                      title={!state.canRestore
                        ? translate($locale, "stash.restoreBlocked")
                        : undefined}
                      on:click={() => dispatch("restore", { hash: entry.hash })}
                      >{translate($locale, "stash.restore")}</button
                    >
                  </div>
                </li>
              {/each}
            </ul>
          {:else}<p
              class="rounded-lg border border-dashed border-tg-border-soft p-4 text-center text-xs text-tg-text-muted"
            >
              {translate($locale, "stash.empty")}
            </p>{/if}
          <p class="mt-2 text-[10px] leading-5 text-tg-text-muted">
            {translate($locale, "stash.restoreHint")}
          </p>
        </div>
      </div>
      <div
        class="flex items-center justify-between gap-3 border-t border-tg-border-soft px-5 py-3"
      >
        <span role="status" class="text-xs text-sky-200"
          >{loading ? translate($locale, "stash.loading") : ""}</span
        >
        <button
          type="button"
          class="tg-control tg-focus-ring px-3 py-2 text-xs"
          disabled={loading}
          on:click={() => dispatch("close")}
          >{translate($locale, "stash.close")}</button
        >
      </div>
    </div>
  </div>
{/if}
