<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";

  export let currentBranch: string | null = null;
  export let upstream: string | null = null;
  export let disabled = false;
  export let disabledReason: string | null = null;
  export let selectable = true;
  export let open = false;

  const dispatch = createEventDispatcher<{ toggle: void }>();

  $: upstreamLabel = upstream ?? translate($locale, "branch.noUpstream");
  $: disabledTitle =
    disabledReason ?? translate($locale, "branch.pushDisabledFallback");
  $: cardLabel = [
    `${translate($locale, "branch.current")}: ${currentBranch ?? translate($locale, "branch.noneSelected")}`,
    upstreamLabel,
    disabled ? disabledTitle : null,
  ]
    .filter(Boolean)
    .join(". ");
  $: buttonDisabled = disabled || !selectable;

  function toggle() {
    if (buttonDisabled) {
      return;
    }

    dispatch("toggle");
  }
</script>

<button
  class={`group tg-card flex min-h-[60px] w-full items-center gap-3 px-3.5 py-2.5 text-left transition duration-200 ${
    disabled
      ? "cursor-not-allowed border-tg-border-soft/60 bg-tg-bg-panel/50 text-tg-text-muted opacity-70"
      : open
        ? "border-tg-blue-soft/45 bg-tg-blue/15 shadow-tg-glow"
        : "tg-card-hover"
  }`}
  type="button"
  disabled={buttonDisabled}
  aria-label={cardLabel}
  aria-expanded={open}
  aria-haspopup="listbox"
  title={disabled ? disabledTitle : undefined}
  on:click|stopPropagation={toggle}
>
  <span
    class={`flex h-9.5 w-9.5 shrink-0 items-center justify-center rounded-[10px] border transition ${
      disabled
        ? "border-tg-border-soft bg-tg-bg-elevated/60 text-tg-text-muted shadow-[inset_0_1px_0_rgba(255,255,255,0.03)]"
        : "tg-icon-tile group-hover:shadow-[0_14px_30px_rgba(37,99,235,0.36)]"
    }`}
    aria-hidden="true"
  >
    <svg viewBox="0 0 16 16" class="h-4.5 w-4.5 fill-current drop-shadow">
      <path
        d="M5.75 2a1.75 1.75 0 1 0 1.72 2.06l1.6.64a1.75 1.75 0 0 0 2.16 2.16l.64 1.6a1.75 1.75 0 1 0 1.38-.56 1.73 1.73 0 0 0-.31.03l-.64-1.6a1.75 1.75 0 0 0-2.16-2.16l-1.6-.64A1.75 1.75 0 0 0 5.75 2Zm0 1.5a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Zm4.5 2a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Zm3 4a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Z"
      ></path>
    </svg>
  </span>

  <span class="min-w-0 flex-1">
    <span
      class={`block truncate text-[0.95rem] font-semibold ${
        disabled ? "text-tg-text-muted" : "text-tg-text-primary"
      }`}
    >
      {currentBranch ?? translate($locale, "branch.noneSelected")}
    </span>
    <span
      class={`mt-1 block truncate text-[11px] ${
        disabled ? "text-tg-text-muted/70" : "text-tg-text-secondary/80"
      }`}
    >
      {upstreamLabel}
    </span>
  </span>

  <span
    class={`flex h-8 w-7 shrink-0 items-center justify-center border-l transition duration-200 ${
      disabled
        ? "border-tg-border-soft/50 text-tg-text-muted/70"
        : open
          ? "border-tg-blue-soft/40 text-sky-100"
          : "border-tg-border-soft text-tg-text-secondary/80 group-hover:text-tg-text-primary"
    }`}
  >
    <svg
      viewBox="0 0 16 16"
      class={`h-3.5 w-3.5 fill-current transition ${open ? "rotate-180" : ""}`}
      aria-hidden="true"
    >
      <path
        d="M4.47 6.97a.75.75 0 0 1 1.06 0L8 9.44l2.47-2.47a.75.75 0 1 1 1.06 1.06l-3 3a.75.75 0 0 1-1.06 0l-3-3a.75.75 0 0 1 0-1.06Z"
      ></path>
    </svg>
  </span>
</button>
