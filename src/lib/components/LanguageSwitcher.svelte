<script lang="ts">
  import {
    SUPPORTED_LOCALES,
    locale,
    setLocale,
    translate,
    type Locale,
  } from "$lib/i18n";

  function changeLocale(value: string) {
    setLocale(value as Locale);
  }
</script>

<label class="flex min-w-[148px] flex-col gap-1 text-left">
  <span
    class="text-[11px] font-semibold uppercase tracking-[0.22em] text-slate-500"
  >
    {translate($locale, "language.label")}
  </span>
  <select
    class="h-9 rounded-sm border border-[#444c56] bg-[#2d333b] px-3 text-sm font-semibold text-[#f0f6fc] outline-none transition hover:border-[#6e7681] focus:border-[#539bf5]/70"
    value={$locale}
    on:change={(event) => changeLocale(event.currentTarget.value)}
    aria-label={translate($locale, "language.label")}
  >
    {#each SUPPORTED_LOCALES as option}
      <option value={option}>{translate($locale, `language.${option}`)}</option>
    {/each}
  </select>
</label>
