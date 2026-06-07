# 发布工程化维护说明

本文档记录 TickGit 发布与依赖维护规则，避免发布工程改动混入功能开发。

## 标准发布流程

发布流程固定为：`develop` 做代码检查，`main` 执行正式打包发布。

### GitHub Actions 规则

- `develop` 分支只跑 CI，不跑 Release。
- `develop` 连续推送多次时，同一分支旧的 CI 会被取消，只保留最新一次推送对应的 CI。
- `./scripts/release-version.sh` 生成的版本提交只修改版本文件和 Release workflow 运行名称，CI 会忽略这类版本提交。
- tag push 不触发 Release。
- Release 只由 `main` 分支 push 触发；没有 tag push 触发，也没有手动触发入口。触发后会读取 `main` 当前版本号，查找对应 tag，例如版本 `1.4.3` 对应 `v1.4.3`，并打包这个 tag 指向的提交。

### 操作步骤

1. 在 `develop` 完成功能/修复提交并推送。

   ```bash
   git switch develop
   git push origin develop
   ```

2. 等 GitHub Actions 里 `develop` 最新一次 CI 通过。

3. 确认本地 `develop` 与 `origin/develop` 一致后，仍然在 `develop` 上执行版本脚本。

   ```bash
   git pull --ff-only origin develop
   ./scripts/release-version.sh 1.4.3
   ```

   这个脚本会先检查本地 `develop` 是否等于 `origin/develop`，同步 Release workflow 运行名称为 `Release v1.4.3`，然后创建 `Release v1.4.3` 提交和 `v1.4.3` tag，并自动推送 `develop` 和 tag。该步骤不会触发 CI，也不会触发 Release。

4. 将 `develop` 快进到 `main` 并推送。

   ```bash
   git switch main
   git pull --ff-only origin main
   git merge --ff-only develop
   git push origin main
   ```

5. `main` push 会触发 Release workflow。workflow 列表标题会显示版本脚本写入的 `Release vX.Y.Z`，并自动打包发布对应 tag。

不要在 `main` 上执行 `./scripts/release-version.sh`。脚本已限制只能在 `develop` 上运行。合并到 main 时使用 fast-forward，避免 main 产生额外 merge commit，让 Release workflow 打包目标对应版本提交。

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
