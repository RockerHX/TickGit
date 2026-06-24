<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import {
    repositoryContextMenuActionMap,
    type RepositoryContextMenuActionId,
  } from "$lib/tickgit/repository-context-menu";
  import type { RepositorySummary } from "$lib/types";

  export let open = false;
  export let x = 0;
  export let y = 0;
  export let repository: RepositorySummary | null = null;
  export let githubUrl: string | null = null;
  export let githubLoading = false;
  export let managementDisabled = false;

  const dispatch = createEventDispatcher<{
    copyName: { repository: RepositorySummary };
    copyPath: { repository: RepositorySummary };
    viewGithub: { repository: RepositorySummary; url: string };
    openTerminal: { repository: RepositorySummary };
    revealInFinder: { repository: RepositorySummary };
    openInVSCode: { repository: RepositorySummary };
    remove: { repository: RepositorySummary };
    close: void;
  }>();

  const actionLabels: Record<RepositoryContextMenuActionId, TranslationKey> = {
    copyName: "repository.contextCopyName",
    copyPath: "repository.contextCopyPath",
    viewGithub: "repository.contextViewGithub",
    openTerminal: "repository.contextOpenTerminal",
    revealInFinder: "repository.contextRevealFinder",
    openInVSCode: "repository.contextOpenVSCode",
    remove: "repository.contextRemove",
  };

  const actionEvents: Record<
    Exclude<RepositoryContextMenuActionId, "viewGithub">,
    | "copyName"
    | "copyPath"
    | "openTerminal"
    | "revealInFinder"
    | "openInVSCode"
    | "remove"
  > = {
    copyName: "copyName",
    copyPath: "copyPath",
    openTerminal: "openTerminal",
    revealInFinder: "revealInFinder",
    openInVSCode: "openInVSCode",
    remove: "remove",
  };

  const orderedActions: RepositoryContextMenuActionId[] = [
    "copyName",
    "copyPath",
    "viewGithub",
    "openTerminal",
    "revealInFinder",
    "openInVSCode",
    "remove",
  ];

  $: actionMap = repositoryContextMenuActionMap({
    repository,
    githubUrl,
    githubLoading,
    managementDisabled,
  });

  function close() {
    dispatch("close");
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (open && event.key === "Escape") {
      close();
    }
  }

  function runAction(actionId: RepositoryContextMenuActionId) {
    const action = actionMap[actionId];
    if (!repository || action.disabled) {
      return;
    }

    if (actionId === "viewGithub") {
      if (!githubUrl) {
        return;
      }
      dispatch("viewGithub", { repository, url: githubUrl });
      return;
    }

    dispatch(actionEvents[actionId], { repository });
  }

  function actionTitle(actionId: RepositoryContextMenuActionId) {
    const reasonKey = actionMap[actionId].disabledReasonKey;
    return reasonKey ? translate($locale, reasonKey) : undefined;
  }
</script>

<svelte:window on:keydown={handleWindowKeydown} />

{#if open && repository}
  <button
    class="fixed inset-0 z-20 cursor-default bg-transparent"
    aria-label={translate($locale, "context.close")}
    on:click={close}
  ></button>
  <div
    class="tg-panel fixed z-30 min-w-[260px] overflow-hidden rounded-tg-control p-1 shadow-lg shadow-black/35"
    style={`left:${x}px; top:${y}px;`}
  >
    {#each orderedActions as actionId, index}
      {#if index === 2 || index === 3 || index === 6}
        <div class="my-1 border-t border-tg-border-soft"></div>
      {/if}
      <button
        class={`flex w-full items-center gap-3 rounded-tg-control px-3 py-2 text-left text-sm transition hover:bg-white/[0.06] disabled:cursor-not-allowed disabled:opacity-40 ${
          actionId === "remove" ? "text-rose-100" : "text-white"
        }`}
        type="button"
        disabled={actionMap[actionId].disabled}
        title={actionTitle(actionId)}
        on:click={() => runAction(actionId)}
      >
        <span
          class={`flex h-5 w-5 shrink-0 items-center justify-center ${
            actionId === "remove" ? "text-rose-300" : "text-tg-blue"
          }`}
          aria-hidden="true"
        >
          {#if actionId === "copyName" || actionId === "copyPath"}
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
              <path
                d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25Z"
              ></path>
              <path
                d="M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"
              ></path>
            </svg>
          {:else if actionId === "viewGithub"}
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
              <path
                d="M8 0a8 8 0 0 0-2.53 15.59c.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82A7.52 7.52 0 0 1 8 3.87c.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8 8 0 0 0 8 0Z"
              ></path>
            </svg>
          {:else if actionId === "remove"}
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
              <path
                d="M6.5 1.75A1.75 1.75 0 0 1 8.25 0h1.5A1.75 1.75 0 0 1 11.5 1.75V2h2.75a.75.75 0 0 1 0 1.5h-.66l-.63 10.05A2.75 2.75 0 0 1 10.21 16H5.79a2.75 2.75 0 0 1-2.75-2.45L2.41 3.5h-.66a.75.75 0 0 1 0-1.5H4.5v-.25ZM6 2v.25h4V2a.25.25 0 0 0-.25-.25h-3.5A.25.25 0 0 0 6 2Zm-2.09 1.5.62 9.95c.04.59.53 1.05 1.12 1.05h4.7c.59 0 1.08-.46 1.12-1.05l.62-9.95Z"
              ></path>
            </svg>
          {:else}
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
              <path
                d="M8.75 1.75a.75.75 0 0 0-1.5 0V7.5H1.75a.75.75 0 0 0 0 1.5h5.5v5.25a.75.75 0 0 0 1.5 0V9h5.5a.75.75 0 0 0 0-1.5h-5.5Z"
              ></path>
            </svg>
          {/if}
        </span>
        <span class="min-w-0 flex-1 truncate">
          {translate($locale, actionLabels[actionId])}
        </span>
      </button>
    {/each}
  </div>
{/if}
