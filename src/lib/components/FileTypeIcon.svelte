<script lang="ts" context="module">
  export type FileTypeIconFile = {
    path: string;
    status?: string;
    language?: string | null;
  };
</script>

<script lang="ts">
  import { statusTone } from "$lib/utils";

  export let file: FileTypeIconFile;

  $: fileName = file.path.split("/").pop()?.toLocaleLowerCase() ?? file.path;
  $: status = file.status ?? "·";
  $: language =
    file.language ??
    (fileName.endsWith(".lock") ||
    fileName === "package-lock.json" ||
    fileName === "pnpm-lock.yaml" ||
    fileName === "bun.lockb"
      ? "lock"
      : fileName === "package.json" || fileName.endsWith(".json")
        ? "json"
        : fileName.endsWith(".yaml") || fileName.endsWith(".yml")
          ? "yaml"
          : null);
  $: label =
    language === "json"
      ? "{}"
      : language === "yaml"
        ? "YAML"
        : language === "lock"
          ? "LOCK"
          : status;
  $: tone =
    language === "json"
      ? "border-tg-blue-soft/25 bg-tg-blue-soft/10 text-sky-100"
      : language === "yaml"
        ? "border-violet-300/25 bg-violet-400/10 text-violet-100"
        : language === "lock"
          ? "border-amber-300/25 bg-amber-400/10 text-amber-100"
          : statusTone(status);
</script>

<span
  class={`inline-flex h-6 min-w-6 shrink-0 items-center justify-center rounded-md border px-1.5 text-[8px] font-semibold uppercase ${tone}`}
  aria-hidden="true"
>
  {label}
</span>
