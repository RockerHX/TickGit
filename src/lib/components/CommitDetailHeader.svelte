<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import type { CommitListItem, CommitMeta } from "$lib/types";
  import { formatAbsoluteDate, getInitials } from "$lib/utils";

  export let commit: CommitListItem | null = null;
  export let commitMeta: CommitMeta | null = null;
  export let collapsed = false;
  export let copiedCommitHash: string | null = null;

  const dispatch = createEventDispatcher<{
    toggleCollapsed: void;
    copyHash: { hash: string };
  }>();
</script>

{#if commit}
  <div class="tg-card min-w-0 p-2.5 shadow-[0_10px_26px_rgba(8,13,24,0.16)]">
    <div class="flex items-start justify-between gap-2.5">
      <div class="min-w-0 flex-1">
        <div
          class="truncate text-[16px] font-semibold leading-5 tracking-[-0.01em] text-tg-text-primary"
          title={commit.summary}
        >
          {commit.summary}
        </div>
        {#if collapsed}
          <div
            class="mt-1 flex min-w-0 flex-wrap items-center gap-x-2.5 gap-y-0.5 text-[10px] text-tg-text-secondary/80"
          >
            <span class="min-w-0 truncate leading-5">
              {formatAbsoluteDate(commit.committedAt, $locale)}
            </span>
            {#if commitMeta}
              <span
                class="flex shrink-0 items-center gap-1.5 font-mono text-[12px] font-semibold tabular-nums"
                aria-label={`+${commitMeta.additions} -${commitMeta.deletions}`}
              >
                <span class="text-emerald-300">+{commitMeta.additions}</span>
                <span class="text-rose-300">-{commitMeta.deletions}</span>
              </span>
            {/if}
          </div>
        {/if}
      </div>
      <button
        type="button"
        class="tg-control tg-focus-ring inline-flex h-6.5 w-6.5 shrink-0 items-center justify-center"
        aria-label={collapsed
          ? translate($locale, "commit.expandInfo")
          : translate($locale, "commit.collapseInfo")}
        aria-expanded={!collapsed}
        on:click={() => dispatch("toggleCollapsed")}
      >
        <svg
          viewBox="0 0 16 16"
          class={`h-3 w-3 fill-current transition-transform ${collapsed ? "rotate-180" : ""}`}
          aria-hidden="true"
        >
          <path
            d="M3.22 9.53a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1-1.06 1.06L8 5.81 4.28 9.53a.75.75 0 0 1-1.06 0Z"
          ></path>
        </svg>
      </button>
    </div>

    {#if !collapsed && (commit.tags.length > 0 || !commit.isPushed)}
      <div class="mt-1 flex flex-wrap items-center gap-1.5">
        {#if !commit.isPushed}
          <span
            class="inline-flex items-center gap-1 rounded-full border border-sky-400/25 bg-sky-400/10 px-1.5 py-0.5 text-[9px] font-medium text-sky-200"
            title={translate($locale, "commit.local")}
          >
            <svg
              viewBox="0 0 16 16"
              class="h-2.5 w-2.5 fill-current"
              aria-hidden="true"
            >
              <path
                d="M8 12.75a.75.75 0 0 1-.75-.75V6.81L5.53 8.53a.75.75 0 1 1-1.06-1.06l3-3a.75.75 0 0 1 1.06 0l3 3a.75.75 0 0 1-1.06 1.06L8.75 6.81V12a.75.75 0 0 1-.75.75Z"
              ></path>
            </svg>
            {translate($locale, "commit.local")}
          </span>
        {/if}
        {#each commit.tags as tag}
          <span
            class="max-w-full truncate rounded-full border border-amber-400/30 bg-amber-400/10 px-1.5 py-0.5 text-[9px] font-medium text-amber-200"
            title={tag}
          >
            {tag}
          </span>
        {/each}
      </div>
    {/if}

    {#if !collapsed}
      <div
        class="mt-2 flex flex-wrap items-center justify-between gap-x-3 gap-y-1.5 text-[11px] text-tg-text-primary"
      >
        <div class="flex min-w-0 flex-1 items-center gap-2">
          <div
            class="flex h-6.5 w-6.5 shrink-0 items-center justify-center rounded-full border border-sky-300/25 bg-sky-400/15 text-[8px] font-semibold tracking-wide text-sky-100"
          >
            {getInitials(commit.authorName)}
          </div>
          <div
            class="min-w-0 flex-1"
            title={`${commit.authorName} <${commit.authorEmail}>`}
          >
            <div class="flex min-w-0 items-center gap-1.5">
              <span class="min-w-0 truncate font-medium text-slate-100">
                {commit.authorName}
              </span>
              <span class="min-w-0 truncate text-tg-text-secondary/80">
                &lt;{commit.authorEmail}&gt;
              </span>
            </div>
          </div>
        </div>

        <div
          class="ml-auto flex min-w-0 flex-wrap items-center justify-end gap-1.5 text-[10px]"
        >
          <button
            type="button"
            class="tg-focus-ring inline-flex h-6.5 max-w-full shrink-0 items-center gap-1 rounded-full border border-tg-blue-soft/20 bg-tg-blue-soft/10 px-2 font-mono font-medium text-sky-100 transition hover:border-tg-blue-soft/45 hover:bg-tg-blue-soft/15"
            title={commit.hash}
            aria-label={copiedCommitHash === commit.hash
              ? translate($locale, "commit.copiedHash")
              : translate($locale, "commit.copyHash")}
            on:click={() => dispatch("copyHash", { hash: commit.hash })}
          >
            <svg
              viewBox="0 0 16 16"
              class="h-2.5 w-2.5 shrink-0 fill-current text-sky-300"
              aria-hidden="true"
            >
              <path
                d="M1.75 8a2.75 2.75 0 1 1 5.18 1.28h2.14a2.751 2.751 0 0 1 5.18-1.28 2.75 2.75 0 1 1-5.18 1.28H6.93A2.75 2.75 0 1 1 1.75 8Zm2.75-1.25a1.25 1.25 0 1 0 0 2.5 1.25 1.25 0 0 0 0-2.5Zm7 0a1.25 1.25 0 1 0 .001 2.501A1.25 1.25 0 0 0 11.5 6.75Z"
              ></path>
            </svg>
            <span class="truncate">{commit.shortHash}</span>
            {#if copiedCommitHash === commit.hash}
              <svg
                viewBox="0 0 16 16"
                class="h-2.5 w-2.5 shrink-0 fill-current text-emerald-300"
                aria-hidden="true"
              >
                <path
                  d="M13.78 4.97a.75.75 0 0 1 0 1.06L7.53 12.28a.75.75 0 0 1-1.06 0L2.22 8.03a.75.75 0 1 1 1.06-1.06L7 10.69l5.72-5.72a.75.75 0 0 1 1.06 0Z"
                ></path>
              </svg>
            {:else}
              <svg
                viewBox="0 0 16 16"
                class="h-2.5 w-2.5 shrink-0 fill-current text-tg-text-secondary"
                aria-hidden="true"
              >
                <path
                  d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25Z"
                ></path>
                <path
                  d="M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"
                ></path>
              </svg>
            {/if}
          </button>
          <span class="text-right leading-5 text-tg-text-secondary/80">
            {formatAbsoluteDate(commit.committedAt, $locale)}
          </span>
        </div>
      </div>

      <div class="mt-2 flex flex-wrap items-center gap-1.5">
        {#if commitMeta}
          <span
            class="inline-flex items-center rounded-full border border-emerald-300/20 bg-emerald-400/10 px-2 py-0.5 font-mono text-[11px] font-semibold text-emerald-300 tabular-nums"
          >
            +{commitMeta.additions}
          </span>
          <span
            class="inline-flex items-center rounded-full border border-rose-300/20 bg-rose-400/10 px-2 py-0.5 font-mono text-[11px] font-semibold text-rose-300 tabular-nums"
          >
            -{commitMeta.deletions}
          </span>
        {/if}
      </div>
    {/if}

    <slot />
  </div>
{:else}
  <div
    class="rounded-xl border border-dashed border-tg-border-soft bg-tg-bg-card/70 px-3 py-3 text-[13px] text-tg-text-secondary/80"
  >
    {translate($locale, "commit.selectPrompt")}
  </div>
{/if}
