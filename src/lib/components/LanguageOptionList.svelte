<script lang="ts">
  import {
    SUPPORTED_LOCALES,
    locale,
    setLocale,
    translate,
    type Locale,
  } from "$lib/i18n";

  function chooseLocale(nextLocale: Locale) {
    setLocale(nextLocale);
  }
</script>

<div
  id="settings-language-title"
  class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-500"
>
  {translate($locale, "settings.language")}
</div>
<div
  class="mt-3 grid gap-2"
  role="group"
  aria-labelledby="settings-language-title"
>
  {#each SUPPORTED_LOCALES as option}
    <button
      type="button"
      class={`flex items-center justify-between rounded-xl border px-4 py-3 text-left transition ${
        $locale === option
          ? "border-[#539bf5]/45 bg-[#347dff]/16 text-[#f0f6fc]"
          : "border-white/[0.08] bg-white/[0.035] text-slate-300 hover:border-[#539bf5]/30 hover:bg-white/[0.06]"
      }`}
      aria-pressed={$locale === option}
      on:click={() => chooseLocale(option)}
    >
      <span class="font-semibold">
        {translate($locale, `language.${option}`)}
      </span>
      {#if $locale === option}
        <span
          class="flex h-6 w-6 items-center justify-center rounded-full bg-[#347dff] text-white"
          aria-hidden="true"
        >
          <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
            <path
              d="M13.78 4.97a.75.75 0 0 1 0 1.06l-6.25 6.25a.75.75 0 0 1-1.06 0L2.22 8.03a.75.75 0 0 1 1.06-1.06L7 10.69l5.72-5.72a.75.75 0 0 1 1.06 0Z"
            ></path>
          </svg>
        </span>
      {/if}
    </button>
  {/each}
</div>
