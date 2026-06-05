<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import { getStepPushPlanBlockedMessage } from "$lib/tickgit/step-push-plan";
  import type { StepPushPlan } from "$lib/types";

  export let open = false;
  export let loading = false;
  export let plan: StepPushPlan | null = null;
  export let errorMessage: string | null = null;

  const dispatch = createEventDispatcher<{
    confirm: void;
    cancel: void;
  }>();

  $: blockedMessage = plan?.blockedReason
    ? getStepPushPlanBlockedMessage(plan, $locale)
    : errorMessage;
  $: canConfirm = Boolean(plan?.available && plan.items.length > 0);
</script>

{#if open}
  <div
    class="fixed inset-0 z-40 flex items-center justify-center bg-black/45 p-4"
  >
    <div
      class="w-full max-w-2xl rounded-sm border border-[#444c56] bg-[#2d333b] shadow-xl shadow-black/40"
      role="dialog"
      aria-modal="true"
      aria-labelledby="step-push-plan-title"
    >
      <div
        class="flex items-start justify-between gap-4 border-b border-[#444c56] px-5 py-4"
      >
        <div>
          <h2
            id="step-push-plan-title"
            class="text-base font-semibold text-white"
          >
            {translate($locale, "stepPush.previewTitle")}
          </h2>
          <p class="mt-1 text-xs text-slate-400">
            {translate($locale, "stepPush.previewDescription")}
          </p>
        </div>
        <button
          type="button"
          class="flex h-7 w-7 items-center justify-center rounded-sm border border-[#444c56] bg-[#24292f] text-slate-300 transition hover:border-[#6e7681] hover:text-white"
          aria-label={translate($locale, "stepPush.closePreview")}
          on:click={() => dispatch("cancel")}
        >
          <svg
            viewBox="0 0 16 16"
            class="h-3.5 w-3.5 fill-current"
            aria-hidden="true"
          >
            <path
              d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 1 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"
            ></path>
          </svg>
        </button>
      </div>

      <div class="px-5 py-4">
        {#if loading}
          <div
            class="rounded-sm border border-sky-500/25 bg-sky-500/10 px-4 py-3 text-sm text-sky-100"
          >
            {translate($locale, "stepPush.loadingPlan")}
          </div>
        {:else if blockedMessage}
          <div
            class="rounded-sm border border-rose-500/30 bg-rose-500/10 px-4 py-3 text-sm text-rose-100"
          >
            <div class="font-semibold">{translate($locale, "stepPush.unavailable")}</div>
            <div class="mt-1 text-xs leading-5 text-rose-100/85">
              {blockedMessage}
            </div>
          </div>
        {:else if plan?.available}
          <div
            class="mb-3 flex items-center justify-between gap-3 text-xs text-slate-400"
          >
            <span
              >{translate($locale, "stepPush.branch")}: <span class="text-slate-200">{plan.branch}</span></span
            >
            <span>{translate($locale, "stepPush.commits", { count: plan.items.length })}</span>
          </div>
          <ol
            class="max-h-80 overflow-y-auto rounded-sm border border-[#444c56] bg-[#24292f]"
          >
            {#each plan.items as item, index}
              <li
                class="flex gap-3 border-b border-[#30363d] px-4 py-3 last:border-b-0"
              >
                <span
                  class="mt-0.5 flex h-5 min-w-5 items-center justify-center rounded-full bg-[#444c56] text-[11px] font-semibold text-slate-200"
                >
                  {index + 1}
                </span>
                <div class="min-w-0 flex-1">
                  <div class="truncate text-sm font-medium text-white">
                    {item.summary || translate($locale, "stepPush.noCommitMessage")}
                  </div>
                  <div class="mt-1 font-mono text-xs text-slate-400">
                    {item.shortHash || item.hash}
                  </div>
                </div>
              </li>
            {/each}
          </ol>
        {:else}
          <div
            class="rounded-sm border border-[#444c56] bg-[#24292f] px-4 py-3 text-sm text-slate-300"
          >
            {translate($locale, "stepPush.noPushableCommits")}
          </div>
        {/if}
      </div>

      <div class="flex justify-end gap-2 border-t border-[#444c56] px-5 py-4">
        <button
          type="button"
          class="rounded-sm border border-[#444c56] bg-[#24292f] px-3 py-2 text-sm font-semibold text-slate-200 transition hover:border-[#6e7681] hover:text-white"
          on:click={() => dispatch("cancel")}
        >
          {translate($locale, "common.cancel")}
        </button>
        <button
          type="button"
          class="rounded-sm border border-[#2f81f7] bg-[#2f81f7] px-3 py-2 text-sm font-semibold text-white transition hover:bg-[#1f6feb] disabled:cursor-not-allowed disabled:border-[#444c56] disabled:bg-[#444c56] disabled:text-slate-400"
          disabled={loading || !canConfirm}
          on:click={() => dispatch("confirm")}
        >
          {translate($locale, "stepPush.start")}
        </button>
      </div>
    </div>
  </div>
{/if}
