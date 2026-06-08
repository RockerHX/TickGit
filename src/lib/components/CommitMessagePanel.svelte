<script lang="ts">
  import { locale, translate } from "$lib/i18n";
  import type { BranchStatus, CommitListItem } from "$lib/types";

  export let commit: CommitListItem;
  export let body = "";
  export let branchStatus: BranchStatus | null = null;

  $: messageText = body.trim() || commit.summary;
</script>

<section
  class="mt-2 rounded-lg border border-tg-border-soft bg-tg-bg-panel px-2.5 py-1.5"
  aria-label={translate($locale, "commit.messageTitle")}
>
  <div class="mb-1 flex flex-wrap items-center justify-between gap-1.5">
    <div
      class="text-[9px] font-semibold uppercase tracking-[0.14em] text-tg-text-muted"
    >
      {translate($locale, "commit.messageTitle")}
    </div>

    <div class="flex flex-wrap justify-end gap-1.5">
      {#if commit.isSafePushTarget}
        <span
          class="rounded-full border border-emerald-300/20 bg-emerald-400/10 px-1.5 py-0.5 text-[9px] font-semibold text-emerald-300"
        >
          {translate($locale, "history.safeStepPush")}
        </span>
      {/if}
      <span
        class="rounded-full border border-sky-300/20 bg-sky-400/10 px-1.5 py-0.5 text-[9px] font-semibold text-sky-200"
      >
        {translate($locale, "commit.behindBadge", {
          count: branchStatus?.behindCount ?? 0,
        })}
      </span>
    </div>
  </div>

  <div
    class="max-h-18 overflow-y-auto whitespace-pre-wrap break-words font-mono text-[11px] leading-5 text-tg-text-secondary"
  >
    {messageText}
  </div>
</section>
