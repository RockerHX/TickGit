<script lang="ts">
  import { statusTone } from "$lib/utils";
  import type { CommitFileChange } from "$lib/types";

  export let file: CommitFileChange;

  $: fileName = file.path.split("/").pop()?.toLocaleLowerCase() ?? file.path;
  $: language =
    file.language ??
    (fileName === "package.json" || fileName.endsWith(".json")
      ? "json"
      : fileName.endsWith(".yaml") || fileName.endsWith(".yml")
        ? "yaml"
        : fileName.endsWith(".lock") ||
            fileName === "package-lock.json" ||
            fileName === "pnpm-lock.yaml" ||
            fileName === "bun.lockb"
          ? "lock"
          : null);
  $: label =
    language === "json"
      ? "{}"
      : language === "yaml"
        ? "YAML"
        : language === "lock"
          ? "LOCK"
          : file.status;
  $: tone =
    language === "json"
      ? "border-sky-300/25 bg-sky-400/10 text-sky-100"
      : language === "yaml"
        ? "border-violet-300/25 bg-violet-400/10 text-violet-100"
        : language === "lock"
          ? "border-amber-300/25 bg-amber-400/10 text-amber-100"
          : statusTone(file.status);
</script>

<span
  class={`inline-flex h-7 min-w-7 shrink-0 items-center justify-center rounded-lg border px-1.5 text-[10px] font-semibold uppercase ${tone}`}
  aria-hidden="true"
>
  {label}
</span>
