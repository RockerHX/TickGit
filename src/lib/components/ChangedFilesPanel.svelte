<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import FileTypeIcon from "$lib/components/FileTypeIcon.svelte";
  import type { CommitFileChange } from "$lib/types";

  export let files: CommitFileChange[] = [];
  export let selectedFilePath: string | null = null;
  export let loadingFiles = false;
  export let copiedFilePath: string | null = null;

  const dispatch = createEventDispatcher<{
    selectFile: { path: string };
    copyPath: { path: string };
  }>();

  function copyPath(event: MouseEvent, path: string) {
    event.stopPropagation();
    dispatch("copyPath", { path });
  }
</script>

<div class="flex min-h-0 flex-col bg-tg-bg-panel">
  <div
    class="flex items-center justify-between gap-3 border-b border-tg-border-soft bg-tg-bg-card px-4 py-3"
  >
    <div class="text-sm font-semibold text-tg-text-primary">
      {translate($locale, "commit.changedFiles")}
    </div>
    <div
      class="rounded-full border border-tg-border-soft bg-white/[0.06] px-2.5 py-0.5 text-xs font-semibold text-tg-text-secondary"
    >
      {files.length}
    </div>
  </div>
  <div class="min-h-0 flex-1 overflow-y-auto bg-tg-bg-panel">
    {#if loadingFiles}
      <div class="tg-card m-4 px-4 py-5 text-sm text-tg-text-secondary/80">
        {translate($locale, "commit.loadingFiles")}
      </div>
    {:else if files.length === 0}
      <div
        class="m-4 rounded-xl border border-dashed border-tg-border-soft bg-tg-bg-card/70 px-4 py-8 text-center text-sm text-tg-text-muted"
      >
        {translate($locale, "commit.noFileChanges")}
      </div>
    {:else}
      <div class="space-y-2 p-3">
        {#each files as file (file.path + file.status)}
          <div
            class={`relative flex items-center gap-2 overflow-hidden rounded-xl border px-3 py-2.5 transition ${
              selectedFilePath === file.path
                ? "border-tg-blue-soft/25 bg-tg-blue-soft/15 shadow-[0_14px_30px_rgba(59,130,246,0.14)]"
                : "border-tg-border-soft bg-tg-bg-card/70 hover:border-tg-blue-soft/20 hover:bg-white/[0.06]"
            }`}
          >
            {#if selectedFilePath === file.path}
              <span
                class="absolute bottom-2 left-0 top-2 w-1 rounded-r-full bg-tg-blue-soft"
                aria-hidden="true"
              ></span>
            {/if}
            <button
              type="button"
              class="min-w-0 flex-1 text-left"
              title={file.displayPath}
              on:click={() => dispatch("selectFile", { path: file.path })}
            >
              <div class="flex min-w-0 items-center gap-3">
                <FileTypeIcon {file} />
                <span
                  class="min-w-0 flex-1 truncate text-[13px] leading-5 text-tg-text-secondary"
                >
                  {file.displayPath}
                </span>
              </div>
            </button>
            {#if typeof file.additions === "number" && typeof file.deletions === "number"}
              <div
                class="flex shrink-0 items-center gap-1.5 text-[11px] font-semibold tabular-nums"
                aria-label={`+${file.additions} -${file.deletions}`}
              >
                <span class="text-emerald-300">+{file.additions}</span>
                <span class="text-rose-300">-{file.deletions}</span>
              </div>
            {/if}
            <button
              type="button"
              class="tg-control tg-focus-ring flex h-7 w-7 shrink-0 items-center justify-center border-transparent"
              title={copiedFilePath === file.path
                ? translate($locale, "file.copiedPath")
                : translate($locale, "file.copyPath")}
              aria-label={copiedFilePath === file.path
                ? translate($locale, "file.copiedPath")
                : translate($locale, "file.copyPath")}
              on:click={(event) => copyPath(event, file.path)}
            >
              {#if copiedFilePath === file.path}
                <svg
                  viewBox="0 0 16 16"
                  class="h-3.5 w-3.5 fill-current text-emerald-300"
                  aria-hidden="true"
                >
                  <path
                    d="M13.78 4.97a.75.75 0 0 1 0 1.06L7.53 12.28a.75.75 0 0 1-1.06 0L2.22 8.03a.75.75 0 1 1 1.06-1.06L7 10.69l5.72-5.72a.75.75 0 0 1 1.06 0Z"
                  ></path>
                </svg>
              {:else}
                <svg
                  viewBox="0 0 16 16"
                  class="h-3.5 w-3.5 fill-current"
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
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
