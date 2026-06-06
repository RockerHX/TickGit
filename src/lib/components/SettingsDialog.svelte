<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import LanguageOptionList from "$lib/components/LanguageOptionList.svelte";

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

      <div class="px-5 py-5">
        <LanguageOptionList />
      </div>
    </div>
  </div>
{/if}
