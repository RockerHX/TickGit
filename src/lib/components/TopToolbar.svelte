<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { tick } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import BranchSwitcher from "$lib/components/BranchSwitcher.svelte";
  import PushCard from "$lib/components/PushCard.svelte";
  import RepositorySwitcher from "$lib/components/RepositorySwitcher.svelte";
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
  export let stashEnabled = false;
  export let rebaseEnabled = false;
  export let rebaseNeeded = false;
  export let rebaseReason = "";

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
    stash: void;
    rebase: void;
    openSettings: void;
  }>();

  let menuOpen = false;
  let menuContainer: HTMLDivElement | null = null;
  let menuButton: HTMLButtonElement | null = null;
  let menuElement: HTMLDivElement | null = null;

  function closeMenu() {
    menuOpen = false;
  }

  async function toggleMenu() {
    menuOpen = !menuOpen;
    if (menuOpen) {
      await tick();
      menuElement
        ?.querySelector<HTMLButtonElement>("button:not(:disabled)")
        ?.focus();
    }
  }

  function handleWindowClick(event: MouseEvent) {
    if (!menuContainer?.contains(event.target as Node)) {
      closeMenu();
    }
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (!menuOpen) return;
    if (event.key === "Escape") {
      closeMenu();
      menuButton?.focus();
    }
  }

  function handleMenuKeydown(event: KeyboardEvent) {
    if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const items = Array.from(
      menuElement?.querySelectorAll<HTMLButtonElement>(
        "button:not(:disabled)",
      ) ?? [],
    );
    if (!items.length) return;
    const currentIndex = items.indexOf(
      document.activeElement as HTMLButtonElement,
    );
    const nextIndex =
      event.key === "Home"
        ? 0
        : event.key === "End"
          ? items.length - 1
          : (currentIndex +
              (event.key === "ArrowDown" ? 1 : items.length - 1)) %
            items.length;
    items[nextIndex]?.focus();
  }

  function runMenuAction(action: "refresh" | "stash" | "rebase" | "settings") {
    closeMenu();
    menuButton?.focus();
    if (action === "refresh") dispatch("refresh");
    if (action === "stash") dispatch("stash");
    if (action === "rebase") dispatch("rebase");
    if (action === "settings") dispatch("openSettings");
  }
</script>

<svelte:window on:click={handleWindowClick} on:keydown={handleWindowKeydown} />

<div class="overflow-visible bg-[#090d16] px-5 pb-2 pt-1.5">
  <div
    class="grid min-w-0 items-start gap-2.5"
    style="grid-template-columns: minmax(220px, 1fr) 8px minmax(165px, 0.68fr) 8px minmax(200px, 250px) 48px;"
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

    <div class="relative mt-[15px] h-12 w-12" bind:this={menuContainer}>
      <button
        bind:this={menuButton}
        type="button"
        class="tg-control tg-focus-ring flex h-12 w-12 items-center justify-center"
        aria-label={translate($locale, "common.actions")}
        aria-haspopup="menu"
        aria-expanded={menuOpen}
        title={translate($locale, "common.actions")}
        on:click={toggleMenu}
        on:keydown={(event) => {
          if (event.key === "ArrowDown" && !menuOpen) {
            event.preventDefault();
            toggleMenu();
          }
        }}
      >
        <svg
          viewBox="0 0 16 16"
          class="h-4 w-4 fill-current"
          aria-hidden="true"
        >
          <circle cx="3" cy="8" r="1.35" />
          <circle cx="8" cy="8" r="1.35" />
          <circle cx="13" cy="8" r="1.35" />
        </svg>
      </button>

      {#if menuOpen}
        <div
          bind:this={menuElement}
          class="tg-panel absolute right-0 top-full z-50 mt-2 w-64 overflow-hidden rounded-tg-card p-1.5 shadow-[0_18px_44px_rgba(0,0,0,0.42)]"
          role="menu"
          tabindex="-1"
          aria-label={translate($locale, "common.actions")}
          on:keydown={handleMenuKeydown}
        >
          <button
            type="button"
            role="menuitem"
            class="flex w-full items-center gap-2.5 rounded-tg-control px-2.5 py-2 text-left text-xs text-tg-text-primary transition hover:bg-white/[0.07] disabled:cursor-not-allowed disabled:opacity-45"
            disabled={!refreshEnabled}
            on:click={() => runMenuAction("refresh")}
          >
            <svg
              viewBox="0 0 16 16"
              class={`h-4 w-4 shrink-0 fill-current ${refreshLoading ? "animate-spin" : ""}`}
              aria-hidden="true"
            >
              <path
                d="M1.705 8a6.5 6.5 0 0 1 11.39-4.273V1.75a.75.75 0 0 1 1.5 0V5.5a.75.75 0 0 1-.75.75h-3.75a.75.75 0 0 1 0-1.5h1.962A5 5 0 1 0 13 8a.75.75 0 0 1 1.5 0A6.5 6.5 0 1 1 1.705 8Z"
              />
            </svg>
            <span class="flex-1"
              >{translate(
                $locale,
                refreshLoading ? "common.refreshing" : "common.refresh",
              )}</span
            >
          </button>
          <button
            type="button"
            role="menuitem"
            class="flex w-full items-center gap-2.5 rounded-tg-control px-2.5 py-2 text-left text-xs text-tg-text-primary transition hover:bg-white/[0.07] disabled:cursor-not-allowed disabled:opacity-45"
            disabled={!stashEnabled}
            on:click={() => runMenuAction("stash")}
          >
            <svg
              viewBox="0 0 24 24"
              class="h-4 w-4 shrink-0 fill-none stroke-current text-sky-300"
              stroke-width="1.7"
              aria-hidden="true"
            >
              <path
                d="M4 9h16v11H4zM3 4h18v5H3zM9 13h6"
                stroke-linejoin="round"
                stroke-linecap="round"
              />
            </svg>
            <span class="flex-1">{translate($locale, "stash.action")}</span>
          </button>
          <button
            type="button"
            role="menuitem"
            class="flex w-full items-center gap-2.5 rounded-tg-control px-2.5 py-2 text-left text-xs text-tg-text-primary transition hover:bg-white/[0.07] disabled:cursor-not-allowed disabled:opacity-45"
            disabled={!rebaseEnabled}
            title={rebaseReason}
            on:click={() => runMenuAction("rebase")}
          >
            <svg
              viewBox="0 0 24 24"
              class="h-4 w-4 shrink-0 fill-none stroke-current text-amber-300"
              stroke-width="1.7"
              aria-hidden="true"
            >
              <circle cx="7" cy="5" r="2" /><circle
                cx="17"
                cy="5"
                r="2"
              /><circle cx="17" cy="19" r="2" />
              <path
                d="M7 7v4a4 4 0 0 0 4 4h2a4 4 0 0 1 4 2M17 7v6"
                stroke-linecap="round"
              />
            </svg>
            <span class="flex-1"
              >{translate($locale, "rebase.shortAction")}</span
            >
            {#if rebaseNeeded}
              <span
                class="rounded bg-amber-400/15 px-1.5 py-0.5 text-[9px] font-semibold text-amber-200"
                >{translate($locale, "rebase.recommended")}</span
              >
            {/if}
          </button>
          <div
            class="my-1 border-t border-tg-border-soft"
            aria-hidden="true"
          ></div>
          <button
            type="button"
            role="menuitem"
            class="flex w-full items-center gap-2.5 rounded-tg-control px-2.5 py-2 text-left text-xs text-tg-text-primary transition hover:bg-white/[0.07]"
            on:click={() => runMenuAction("settings")}
          >
            <svg
              viewBox="0 0 16 16"
              class="h-4 w-4 shrink-0 fill-current text-tg-text-secondary"
              aria-hidden="true"
            >
              <path
                d="M8 1.5a1.75 1.75 0 0 0-1.72 1.43l-.05.26a5.8 5.8 0 0 0-.9.37l-.22-.15a1.75 1.75 0 0 0-2.23.22l-.25.25a1.75 1.75 0 0 0-.22 2.23l.15.22c-.15.29-.27.59-.37.9l-.26.05a1.75 1.75 0 0 0 0 3.44l.26.05c.1.31.22.61.37.9l-.15.22a1.75 1.75 0 0 0 .22 2.23l.25.25a1.75 1.75 0 0 0 2.23.22l.22-.15c.29.15.59.27.9.37l.05.26a1.75 1.75 0 0 0 3.44 0l.05-.26c.31-.1.61-.22.9-.37l.22.15a1.75 1.75 0 0 0 2.23-.22l.25-.25a1.75 1.75 0 0 0 .22-2.23l-.15-.22c.15-.29.27-.59.37-.9l.26-.05a1.75 1.75 0 0 0 0-3.44l-.26-.05a5.8 5.8 0 0 0-.37-.9l.15-.22a1.75 1.75 0 0 0-.22-2.23l-.25-.25a1.75 1.75 0 0 0-2.23-.22l-.22.15a5.8 5.8 0 0 0-.9-.37l-.05-.26A1.75 1.75 0 0 0 8 1.5Zm0 4a2.5 2.5 0 1 1 0 5 2.5 2.5 0 0 1 0-5Z"
              />
            </svg>
            <span>{translate($locale, "settings.title")}</span>
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>
