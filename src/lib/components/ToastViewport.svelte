<script lang="ts">
  import type { ToastItem } from "$lib/types";

  export let toasts: ToastItem[] = [];
</script>

<div
  class="pointer-events-none fixed right-4 top-4 z-50 flex w-[360px] max-w-[calc(100vw-2rem)] flex-col gap-3"
>
  {#each toasts as toast (toast.id)}
    <div
      class={`rounded-sm border px-4 py-3 shadow-lg shadow-black/25 ${
        toast.tone === "error"
          ? "border-rose-500/40 bg-[#3a2024]"
          : toast.tone === "success"
            ? "border-emerald-500/35 bg-[#1f3328]"
            : "border-tg-border-strong bg-tg-bg-card"
      }`}
    >
      <div class="text-sm font-semibold text-white">{toast.title}</div>
      <div class="mt-1 text-sm text-tg-text-secondary">{toast.message}</div>
    </div>
  {/each}
</div>
