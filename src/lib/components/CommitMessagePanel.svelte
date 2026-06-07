<script lang="ts">
  import { locale, translate } from "$lib/i18n";
  import type { CommitListItem, CommitMeta } from "$lib/types";

  export let commit: CommitListItem | null = null;
  export let commitMeta: CommitMeta | null = null;
  export let behindCount = 0;

  $: body = commitMeta?.body.trim() ?? "";
  $: message = body || commit?.summary || "";
</script>

<section
  class="border-t border-white/10 bg-[#111827] px-4 py-3"
  aria-label={translate($locale, "commit.messageTitle")}
>
  <div class="mb-2 flex flex-wrap items-center justify-between gap-2">
    <h2 class="min-w-0 text-sm font-semibold text-slate-50">
      {translate($locale, "commit.messageTitle")}
    </h2>
    <div class="flex flex-wrap justify-end gap-1.5">
      {#if commit?.isSafePushTarget}
        <span
          class="rounded-full border border-emerald-300/20 bg-emerald-400/10 px-2.5 py-0.5 text-[11px] font-semibold text-emerald-300"
        >
          {translate($locale, "history.safeStepPush")}
        </span>
      {/if}
      <span
        class="rounded-full border border-sky-300/20 bg-sky-400/10 px-2.5 py-0.5 text-[11px] font-semibold text-sky-200"
      >
        {translate($locale, "commit.behindBadge", { count: behindCount })}
      </span>
    </div>
  </div>

  <div
    class="max-h-32 overflow-y-auto whitespace-pre-wrap break-words rounded-xl border border-white/10 bg-[#18202d]/80 px-3 py-2.5 font-mono text-[12px] leading-5 text-slate-300"
  >
    {#if message}
      {message}
    {:else}
      <span class="font-sans text-slate-500">
        {translate($locale, "commit.messageEmpty")}
      </span>
    {/if}
  </div>
</section>
