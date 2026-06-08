<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";

  export let open = false;

  const dispatch = createEventDispatcher<{ open: void }>();

  function openSettings() {
    dispatch("open");
  }
</script>

<button
  type="button"
  class={`tg-control tg-focus-ring mt-[18px] flex min-h-[56px] w-full items-center justify-center ${
    open ? "border-tg-blue-soft/40 bg-tg-blue/12 text-tg-text-primary" : ""
  }`}
  aria-label={translate($locale, "settings.title")}
  aria-haspopup="dialog"
  aria-expanded={open}
  title={translate($locale, "settings.title")}
  on:click={openSettings}
>
  <svg viewBox="0 0 16 16" class="h-4.5 w-4.5 fill-current" aria-hidden="true">
    <path
      d="M8 1.5a1.75 1.75 0 0 0-1.72 1.43l-.05.26a5.8 5.8 0 0 0-.9.37l-.22-.15a1.75 1.75 0 0 0-2.23.22l-.25.25a1.75 1.75 0 0 0-.22 2.23l.15.22c-.15.29-.27.59-.37.9l-.26.05a1.75 1.75 0 0 0 0 3.44l.26.05c.1.31.22.61.37.9l-.15.22a1.75 1.75 0 0 0 .22 2.23l.25.25a1.75 1.75 0 0 0 2.23.22l.22-.15c.29.15.59.27.9.37l.05.26a1.75 1.75 0 0 0 3.44 0l.05-.26c.31-.1.61-.22.9-.37l.22.15a1.75 1.75 0 0 0 2.23-.22l.25-.25a1.75 1.75 0 0 0 .22-2.23l-.15-.22c.15-.29.27-.59.37-.9l.26-.05a1.75 1.75 0 0 0 0-3.44l-.26-.05a5.8 5.8 0 0 0-.37-.9l.15-.22a1.75 1.75 0 0 0-.22-2.23l-.25-.25a1.75 1.75 0 0 0-2.23-.22l-.22.15a5.8 5.8 0 0 0-.9-.37l-.05-.26A1.75 1.75 0 0 0 8 1.5Zm0 4a2.5 2.5 0 1 1 0 5 2.5 2.5 0 0 1 0-5Z"
    ></path>
  </svg>
</button>
