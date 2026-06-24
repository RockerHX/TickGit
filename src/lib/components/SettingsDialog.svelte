<script lang="ts">
  import { tick } from "svelte";
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import LanguageOptionList from "$lib/components/LanguageOptionList.svelte";
  import {
    commitInfoDefaultCollapsed,
    setCommitInfoDefaultCollapsed,
    setTextSelectionEnabled,
    textSelectionEnabled,
  } from "$lib/tickgit/preferences";

  export let open = false;

  const dispatch = createEventDispatcher<{ close: void }>();

  let dialogElement: HTMLDivElement | null = null;
  let previousFocusedElement: HTMLElement | null = null;
  let wasOpen = false;

  $: if (open && !wasOpen) {
    wasOpen = true;
    previousFocusedElement =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    void focusFirstControl();
  } else if (!open && wasOpen) {
    wasOpen = false;
    restorePreviousFocus();
  }

  function getFocusableElements() {
    if (!dialogElement) {
      return [];
    }

    return Array.from(
      dialogElement.querySelectorAll<HTMLElement>(
        [
          "button:not([disabled])",
          "[href]",
          "input:not([disabled])",
          "select:not([disabled])",
          "textarea:not([disabled])",
          "[tabindex]:not([tabindex='-1'])",
        ].join(","),
      ),
    ).filter(
      (element) =>
        !element.hasAttribute("disabled") &&
        element.getAttribute("aria-hidden") !== "true" &&
        element.offsetParent !== null,
    );
  }

  async function focusFirstControl() {
    await tick();
    getFocusableElements()[0]?.focus();
  }

  function restorePreviousFocus() {
    if (!previousFocusedElement?.isConnected) {
      previousFocusedElement = null;
      return;
    }

    previousFocusedElement.focus();
    previousFocusedElement = null;
  }

  function close() {
    dispatch("close");
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (!open) {
      return;
    }

    if (event.key === "Escape") {
      close();
      return;
    }

    if (event.key !== "Tab") {
      return;
    }

    const focusableElements = getFocusableElements();
    if (focusableElements.length === 0) {
      event.preventDefault();
      return;
    }

    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];
    const activeElement = document.activeElement;

    if (!dialogElement?.contains(activeElement)) {
      event.preventDefault();
      firstElement.focus();
      return;
    }

    if (event.shiftKey && activeElement === firstElement) {
      event.preventDefault();
      lastElement.focus();
      return;
    }

    if (!event.shiftKey && activeElement === lastElement) {
      event.preventDefault();
      firstElement.focus();
    }
  }
</script>

<svelte:window on:keydown={handleWindowKeydown} />

{#if open}
  <div
    class="fixed inset-0 z-40 flex items-center justify-center bg-tg-bg-app/75 px-4 backdrop-blur-sm"
    role="presentation"
    on:click={handleBackdropClick}
  >
    <div
      class="tg-panel w-full max-w-md overflow-hidden rounded-2xl shadow-[0_28px_90px_rgba(0,0,0,0.55)]"
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-title"
      aria-describedby="settings-description"
      bind:this={dialogElement}
    >
      <div
        class="flex items-center justify-between border-b border-tg-border-soft px-5 py-4"
      >
        <div>
          <h2
            id="settings-title"
            class="text-base font-semibold text-tg-text-primary"
          >
            {translate($locale, "settings.title")}
          </h2>
          <p
            id="settings-description"
            class="mt-1 text-xs text-tg-text-secondary/80"
          >
            {translate($locale, "settings.description")}
          </p>
        </div>
        <button
          type="button"
          class="tg-control tg-focus-ring flex h-9 w-9 items-center justify-center rounded-full text-tg-text-secondary/80"
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

        <div class="tg-card px-4 py-3">
          <div class="flex items-start justify-between gap-4">
            <div class="min-w-0">
              <div
                id="settings-commit-info-title"
                class="text-xs font-semibold uppercase tracking-[0.2em] text-tg-text-muted"
              >
                {translate($locale, "settings.commitInfo")}
              </div>
              <p class="mt-2 text-sm font-medium text-tg-text-secondary">
                {translate($locale, "settings.commitInfoDefaultCollapsed")}
              </p>
              <p class="mt-1 text-xs leading-5 text-tg-text-secondary/80">
                {translate(
                  $locale,
                  "settings.commitInfoDefaultCollapsedDescription",
                )}
              </p>
            </div>
            <button
              type="button"
              class={`tg-focus-ring relative mt-1 h-6 w-11 shrink-0 rounded-full border transition ${
                $commitInfoDefaultCollapsed
                  ? "border-tg-blue-soft/55 bg-tg-blue/70"
                  : "border-tg-border-soft bg-white/[0.08]"
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

        <div class="tg-card px-4 py-3">
          <div class="flex items-start justify-between gap-4">
            <div class="min-w-0">
              <div
                id="settings-text-selection-title"
                class="text-xs font-semibold uppercase tracking-[0.2em] text-tg-text-muted"
              >
                {translate($locale, "settings.textSelection")}
              </div>
              <p class="mt-2 text-sm font-medium text-tg-text-secondary">
                {translate($locale, "settings.textSelectionEnabled")}
              </p>
              <p class="mt-1 text-xs leading-5 text-tg-text-secondary/80">
                {translate($locale, "settings.textSelectionEnabledDescription")}
              </p>
            </div>
            <button
              type="button"
              class={`tg-focus-ring relative mt-1 h-6 w-11 shrink-0 rounded-full border transition ${
                $textSelectionEnabled
                  ? "border-tg-blue-soft/55 bg-tg-blue/70"
                  : "border-tg-border-soft bg-white/[0.08]"
              }`}
              role="switch"
              aria-labelledby="settings-text-selection-title"
              aria-checked={$textSelectionEnabled}
              on:click={() => setTextSelectionEnabled(!$textSelectionEnabled)}
            >
              <span
                class={`absolute top-1/2 h-4 w-4 -translate-y-1/2 rounded-full bg-white shadow transition ${
                  $textSelectionEnabled ? "left-[22px]" : "left-1"
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
