<script lang="ts">
  import type { ToastItem } from "$lib/types";

  export let toasts: ToastItem[] = [];
</script>

<div
  class="pointer-events-none fixed right-4 top-4 z-50 flex w-[360px] max-w-[calc(100vw-2rem)] flex-col gap-3"
>
  {#each toasts as toast (toast.id)}
    <div
      class={`rounded-2xl border px-4 py-3 shadow-xl backdrop-blur ${
        toast.tone === "error"
          ? "border-rose-500/30 bg-rose-950/80"
          : toast.tone === "success"
            ? "border-emerald-500/30 bg-emerald-950/70"
            : "border-slate-700 bg-slate-900/85"
      }`}
    >
      <div class="text-sm font-semibold text-white">{toast.title}</div>
      <div class="mt-1 text-sm text-slate-300">{toast.message}</div>
    </div>
  {/each}
</div>
