<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import LanguageOptionList from "$lib/components/LanguageOptionList.svelte";
  import {
    commitInfoDefaultCollapsed,
    setCommitInfoDefaultCollapsed,
  } from "$lib/tickgit/preferences";

  export let open = false;

  const dispatch = createEventDispatcher<{ close: void }>();

  function close() {
    dispatch("close");
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (!open || event.key !== "Escape") {
      return;
    }

    close();
  }
</script>

<svelte:window on:keydown={handleWindowKeydown} />

{#if open}
  <div
    class="fixed inset-0 z-40 flex items-center justify-center bg-[#0d1117]/70 px-4 backdrop-blur-sm"
    role="presentation"
    on:click={handleBackdropClick}
  >
    <div
      class="w-full max-w-md overflow-hidden rounded-2xl border border-white/[0.08] bg-[#1b2027] shadow-[0_28px_90px_rgba(0,0,0,0.55)]"
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-title"
      aria-describedby="settings-description"
    >
      <div
        class="flex items-center justify-between border-b border-white/[0.06] px-5 py-4"
      >
        <div>
          <h2
            id="settings-title"
            class="text-base font-semibold text-[#f0f6fc]"
          >
            {translate($locale, "settings.title")}
          </h2>
          <p id="settings-description" class="mt-1 text-xs text-slate-400">
            {translate($locale, "settings.description")}
          </p>
        </div>
        <button
          type="button"
          class="flex h-9 w-9 items-center justify-center rounded-full border border-white/[0.08] bg-white/[0.04] text-slate-400 transition hover:border-[#539bf5]/35 hover:text-[#f0f6fc]"
          aria-label={translate($locale, "settings.close")}
          on:click={close}
        >
          <svg
            viewBox="0 0 16 16"
            class="h-4 w-4 fill-current"
            aria-hidden="true"
          >
            <path
              d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"
            ></path>
          </svg>
        </button>
      </div>

      <div class="space-y-5 px-5 py-5">
        <LanguageOptionList />

        <div
          class="rounded-xl border border-white/[0.08] bg-white/[0.035] px-4 py-3"
        >
          <div class="flex items-start justify-between gap-4">
            <div class="min-w-0">
              <div
                id="settings-commit-info-title"
                class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-500"
              >
                {translate($locale, "settings.commitInfo")}
              </div>
              <p class="mt-2 text-sm font-medium text-slate-200">
                {translate($locale, "settings.commitInfoDefaultCollapsed")}
              </p>
              <p class="mt-1 text-xs leading-5 text-slate-400">
                {translate(
                  $locale,
                  "settings.commitInfoDefaultCollapsedDescription",
                )}
              </p>
            </div>
            <button
              type="button"
              class={`relative mt-1 h-6 w-11 shrink-0 rounded-full border transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#539bf5]/55 focus-visible:ring-offset-2 focus-visible:ring-offset-[#1b2027] ${
                $commitInfoDefaultCollapsed
                  ? "border-[#539bf5]/55 bg-[#347dff]/70"
                  : "border-white/[0.12] bg-white/[0.08]"
              }`}
              role="switch"
              aria-labelledby="settings-commit-info-title"
              aria-checked={$commitInfoDefaultCollapsed}
              on:click={() =>
                setCommitInfoDefaultCollapsed(!$commitInfoDefaultCollapsed)}
            >
              <span
                class={`absolute top-1/2 h-4 w-4 -translate-y-1/2 rounded-full bg-white shadow transition ${
                  $commitInfoDefaultCollapsed ? "left-[22px]" : "left-1"
                }`}
                aria-hidden="true"
              ></span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
