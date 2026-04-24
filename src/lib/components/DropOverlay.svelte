<script lang="ts">
  export let active = false;
</script>

{#if active}
  <div
    class="pointer-events-none fixed inset-0 z-40 bg-sky-500/12 backdrop-blur-[2px]"
  >
    <div class="flex h-full items-center justify-center p-6">
      <div
        class="rounded-3xl border border-sky-400/40 bg-slate-950/90 px-10 py-8 text-center shadow-2xl shadow-sky-950/60"
      >
        <div
          class="text-sm font-semibold uppercase tracking-[0.3em] text-sky-300"
        >
          Drop Repository
        </div>
        <div class="mt-3 text-2xl font-semibold text-white">
          将 Git 仓库文件夹拖入这里
        </div>
        <div class="mt-2 text-sm text-slate-400">
          TickGit 会自动校验 .git 并加入仓库列表
        </div>
      </div>
    </div>
  </div>
{/if}
