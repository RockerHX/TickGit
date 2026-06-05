# 发布工程化维护说明

本文档记录 TickGit 发布与依赖维护规则，避免发布工程改动混入功能开发。

## 依赖升级策略

- Tauri 2 相关依赖需要优先保持前端包与 Rust 依赖的 minor 族对齐，包括 `@tauri-apps/api`、`@tauri-apps/cli`、`@tauri-apps/plugin-*`、`tauri`、`tauri-build`、`tauri-plugin-*`。
- patch / minor 升级可以作为独立批次处理，但不应和功能开发 PR 混合。
- major 升级必须单独开分支处理，并在提交说明中记录迁移点、破坏性变化和验证结果。
- 不确定是否属于安全的小版本升级时，按 major 升级流程处理。

major 升级固定验收命令：

```bash
rtk pnpm test:run
rtk proxy pnpm typecheck
rtk pnpm build
rtk pnpm format:check
rtk cargo test --manifest-path src-tauri/Cargo.toml
rtk cargo check --manifest-path src-tauri/Cargo.toml
```

## 当前依赖升级分类

截至 2026-06-05，本轮 `pnpm outdated` 结果按以下策略拆分。

可纳入小版本 / patch 批次：

- `@tauri-apps/api`
- `@tauri-apps/cli`
- `@tauri-apps/plugin-opener`
- `@sveltejs/kit`
- `svelte`
- `svelte-check`
- `tailwindcss`
- `@tailwindcss/vite`

暂不在本轮升级的 major 候选：

- `vite`
- `vitest`
- `typescript`
- `@sveltejs/vite-plugin-svelte`
- `prettier-plugin-svelte`

major 候选应在独立分支中处理，不能作为发布脚本、安全加固或普通功能提交的顺手改动。
