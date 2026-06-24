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
    repositoryCopyName: { repository: RepositorySummary };
    repositoryCopyPath: { repository: RepositorySummary };
    repositoryViewGithub: { repository: RepositorySummary; url: string };
    repositoryOpenTerminal: { repository: RepositorySummary };
    repositoryRevealInFinder: { repository: RepositorySummary };
    repositoryOpenInVSCode: { repository: RepositorySummary };
    branchChange: { branch: string };
    push: void;
    refresh: void;
    openSettings: void;
  }>();
</script>

<div
  class="overflow-visible bg-[radial-gradient(circle_at_top,rgba(59,130,246,0.1),transparent_42%)] px-5 pb-2 pt-1.5"
>
  <div
    class="grid min-w-0 items-start gap-2.5"
    style="grid-template-columns: minmax(220px, 1fr) 8px minmax(165px, 0.68fr) 8px minmax(200px, 250px) 48px 48px;"
  >
    <div class="min-w-0">
      <div
        class="mb-0.5 text-[8px] font-semibold uppercase tracking-[0.16em] text-tg-text-muted"
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
        on:copyName={(event) =>
          dispatch("repositoryCopyName", {
            repository: event.detail.repository,
          })}
        on:copyPath={(event) =>
          dispatch("repositoryCopyPath", {
            repository: event.detail.repository,
          })}
        on:viewGithub={(event) =>
          dispatch("repositoryViewGithub", {
            repository: event.detail.repository,
            url: event.detail.url,
          })}
        on:openTerminal={(event) =>
          dispatch("repositoryOpenTerminal", {
            repository: event.detail.repository,
          })}
        on:revealInFinder={(event) =>
          dispatch("repositoryRevealInFinder", {
            repository: event.detail.repository,
          })}
        on:openInVSCode={(event) =>
          dispatch("repositoryOpenInVSCode", {
            repository: event.detail.repository,
          })}
      />
    </div>

    <div
      class="flex h-[66px] items-center justify-center text-tg-text-muted/70"
      aria-hidden="true"
    >
      <span class="text-xs">›</span>
    </div>

    <div class="min-w-0">
      <div
        class="mb-0.5 text-[8px] font-semibold uppercase tracking-[0.16em] text-tg-text-muted"
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
      class="flex h-[66px] items-center justify-center text-tg-text-muted/70"
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
