<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import BranchSwitcher from "$lib/components/BranchSwitcher.svelte";
  import PushCard from "$lib/components/PushCard.svelte";
  import RefreshButton from "$lib/components/RefreshButton.svelte";
  import RepositorySwitcher from "$lib/components/RepositorySwitcher.svelte";
  import SettingsButton from "$lib/components/SettingsButton.svelte";
  import type { BranchStatus, RepositorySummary } from "$lib/types";

  export let repositories: RepositorySummary[] = [];
  export let currentRepository: RepositorySummary | null = null;
  export let repositoryManagementDisabled = false;
  export let localBranches: string[] = [];
  export let branchStatus: BranchStatus | null = null;
  export let branchDisabled = false;
  export let branchDisabledReason: string | null = null;
  export let pushTitle = "";
  export let pushSubtitle = "";
  export let pushAheadCount = 0;
  export let pushEnabled = false;
  export let pushLoading = false;
  export let pushBlockedReason: string | null = null;
  export let pushAriaLabel = "";
  export let refreshEnabled = false;
  export let refreshLoading = false;
  export let settingsOpen = false;

  const dispatch = createEventDispatcher<{
    repositoryChange: { path: string };
    repositoryRemove: { path: string };
    repositoryRelocate: { path: string };
    branchChange: { branch: string };
    push: void;
    refresh: void;
    openSettings: void;
  }>();
</script>

<div
  class="overflow-visible bg-[radial-gradient(circle_at_top,rgba(59,130,246,0.1),transparent_42%)] px-6 pb-3 pt-2"
>
  <div class="relative mb-2 flex h-8 items-center justify-center">
    <div class="absolute left-0 flex items-center gap-2" aria-hidden="true">
      <span class="h-3 w-3 rounded-full bg-rose-400/80 shadow-sm"></span>
      <span class="h-3 w-3 rounded-full bg-amber-300/80 shadow-sm"></span>
      <span class="h-3 w-3 rounded-full bg-emerald-400/80 shadow-sm"></span>
    </div>

    <div
      class="pointer-events-none flex items-center gap-2 rounded-full border border-tg-border-soft bg-tg-bg-card/55 px-3 py-1 text-xs font-semibold tracking-[0.12em] text-tg-text-primary shadow-[0_10px_26px_rgba(15,23,42,0.22)]"
    >
      <span
        class="flex h-5 w-5 items-center justify-center rounded-full bg-tg-blue/20 text-sky-100"
        aria-hidden="true"
      >
        <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
          <path
            d="M5.75 2a1.75 1.75 0 1 0 1.72 2.06l1.6.64a1.75 1.75 0 0 0 2.16 2.16l.64 1.6a1.75 1.75 0 1 0 1.38-.56 1.73 1.73 0 0 0-.31.03l-.64-1.6a1.75 1.75 0 0 0-2.16-2.16l-1.6-.64A1.75 1.75 0 0 0 5.75 2Zm0 1.5a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Zm4.5 2a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Zm3 4a.25.25 0 1 1 0 .5.25.25 0 0 1 0-.5Z"
          ></path>
        </svg>
      </span>
      <span>{translate($locale, "app.title")}</span>
    </div>
  </div>

  <div
    class="grid min-w-0 items-start gap-3"
    style="grid-template-columns: minmax(240px, 1fr) 10px minmax(180px, 0.72fr) 10px minmax(220px, 270px) 56px 56px;"
  >
    <div class="min-w-0">
      <div
        class="mb-1 text-[9px] font-semibold uppercase tracking-[0.16em] text-tg-text-muted"
      >
        {translate($locale, "repository.current")}
      </div>
      <RepositorySwitcher
        {repositories}
        currentPath={currentRepository?.path ?? null}
        managementDisabled={repositoryManagementDisabled}
        on:change={(event) =>
          dispatch("repositoryChange", { path: event.detail.path })}
        on:remove={(event) =>
          dispatch("repositoryRemove", { path: event.detail.path })}
        on:relocate={(event) =>
          dispatch("repositoryRelocate", { path: event.detail.path })}
      />
    </div>

    <div
      class="flex h-[74px] items-center justify-center text-tg-text-muted/70"
      aria-hidden="true"
    >
      <span class="text-xs">›</span>
    </div>

    <div class="min-w-0">
      <div
        class="mb-1 text-[9px] font-semibold uppercase tracking-[0.16em] text-tg-text-muted"
      >
        {translate($locale, "branch.current")}
      </div>
      <BranchSwitcher
        branches={localBranches}
        currentBranch={branchStatus?.branch ?? null}
        upstream={branchStatus?.upstream ?? null}
        disabled={branchDisabled}
        disabledReason={branchDisabledReason}
        on:change={(event) =>
          dispatch("branchChange", { branch: event.detail.branch })}
      />
    </div>

    <div
      class="flex h-[74px] items-center justify-center text-tg-text-muted/70"
      aria-hidden="true"
    >
      <span class="text-xs">›</span>
    </div>

    <PushCard
      title={pushTitle}
      subtitle={pushSubtitle}
      aheadCount={pushAheadCount}
      enabled={pushEnabled}
      loading={pushLoading}
      blockedReason={pushBlockedReason}
      ariaLabel={pushAriaLabel}
      on:push={() => dispatch("push")}
    />

    <RefreshButton
      enabled={refreshEnabled}
      loading={refreshLoading}
      on:refresh={() => dispatch("refresh")}
    />

    <SettingsButton
      open={settingsOpen}
      on:open={() => dispatch("openSettings")}
    />
  </div>
</div>
