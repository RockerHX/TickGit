# TickGit 优化与后续开发计划

## 1. 定位

本文档用于记录 TickGit 当前代码盘点后的优化项和后续开发路线，供后续迭代按优先级执行。

本文不是架构约束文档。若后续实施过程中改变了模块边界、Git 策略、后台任务模型、持久化方式或前后端协议，应同步更新：

- `docs/ARCHITECTURE.md`
- `docs/AI_DEVELOPMENT.md`
- `README.md` 中对应范围说明

盘点时间：2026-06-03。

---

## 2. 当前状态摘要

当前项目整体分层清晰：

- 前端通过 `src/lib/tauri/api.ts` / `events.ts` 集中访问 Tauri command 与 event
- Git 核心能力集中在 Rust 侧，并已拆分到 `src-tauri/src/git/` 模块目录
- first-parent 安全分步推送已有明确产品约束和较多 Rust 单元测试
- 前端 diff、页面状态、仓库数据编排、推送事件与分步推送计划 helper 已有 Vitest 覆盖
- 发布 workflow 已覆盖多平台打包

当前主要短板：

- 缺少日常 push / PR 级 CI，只在 release / manual package 阶段做部分验证
- 同步版 push command/API 仍需确认是否保留
- 推送任务取消与运行状态查询仍暂缓
- 工作区变更视图、提交创建、历史搜索等 GitHub Desktop 类能力仍待扩展

---

## 3. 已验证的基线

本次盘点初始发现的 Rust 单测漂移与格式门禁问题已在 P0/P2 实施中修复。截至 2026-06-04 P2 收尾，本地完整质量门禁已通过：

```bash
rtk pnpm test:run
rtk proxy pnpm typecheck
rtk pnpm build
rtk pnpm format:check
rtk cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
rtk cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
rtk cargo test --manifest-path src-tauri/Cargo.toml
```

---

## 4. P0：先修质量基线

### 4.1 统一 behind/diverged 场景的 safe push 口径

现象：

- 当前实现中，`branch_status_for_path` 在 `behind_count > 0` 时禁用推送。
- `safe_unpushed_hashes_in_push_order` 会要求 upstream 是目标 commit 的祖先。
- 当远端已经推进、本地又有自己的 commit 时，本地 commit 仍显示为未推送，但不应被标记为 safe push target。
- 现有测试却期望该 commit 仍是 safe target。

建议决策：

- behind/diverged 场景下：
  - 本地未推送 commit 应继续在历史中可见
  - 不标记为 safe push target
  - 禁用 step push / push to commit
  - 不在 TickGit 内实现 pull / merge / rebase
  - 文案优先提示“远端已有更新，TickGit 暂不能安全推送”，并引导用户使用 GitHub Desktop 或 SourceTree 同步后回到 TickGit 刷新

实施项：

- 修改失败测试预期。
- 增加测试覆盖：
  - behind + local commit：unpushed 可见、safe 为 false、safe count 为 0
  - behind + refresh 后 push action 不可用
  - push blocked reason 使用 behind/diverged 更明确文案
  - UI 展示 GitHub Desktop / SourceTree 的基础同步引导和刷新状态按钮
- 如产品决策相反，则应先更新架构文档，再调整实现。

验收：

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

必须全绿。

### 4.2 收紧后端 push 分支参数

现象：

- 项目约束是“只支持当前检出分支”。
- `push_to_commit(repo_path, branch, hash)` 当前会校验 hash 是否对当前分支安全，但最终使用调用方传入的 `branch` 组装 refspec。
- `start_push_current_branch` 和 `push_current_branch` 也主要依赖前端按钮状态防误触。

风险：

- 后端 command 暴露面比产品约束宽。
- 如果前端状态、调试调用或未来代码绕过 UI 禁用逻辑，可能出现后端行为与文档不一致。

建议：

- 后端统一以当前检出分支为准。
- 对外部传入的 branch：
  - 要么删除该参数
  - 要么强校验 `branch == current_branch`
- 同步调整：
  - `src-tauri/src/models.rs`
  - `src-tauri/src/commands.rs`
  - `src/lib/types.ts`
  - `src/lib/tauri/api.ts`
  - `src/routes/+page.svelte`
  - Rust / Vitest 测试

验收：

```bash
pnpm test:run
pnpm typecheck
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

### 4.3 修复格式门禁

实施项：

```bash
pnpm prettier --write src-tauri/tauri.conf.json
pnpm format:check
```

验收：

- `pnpm format:check` 通过。

### 4.4 增加日常 CI

现状：

- `release.yml` 只在 `v*` tag 触发。
- `manual-package.yml` 只手动触发。
- 缺少 push / PR 阶段质量门禁。

建议新增 `.github/workflows/ci.yml`：

触发：

```yaml
on:
  pull_request:
  push:
    branches:
      - main
```

建议步骤：

```bash
pnpm install --frozen-lockfile
pnpm test:run
pnpm typecheck
pnpm build
pnpm format:check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

验收：

- 新 CI 在 main / PR 上通过。
- release workflow 不再是发现基础质量问题的第一道门。

---

## 5. P1：降低维护复杂度

完成状态：已于 2026-06-04 完成。

完成摘要：

- `src-tauri/src/git.rs` 已拆为 `src-tauri/src/git/` 目录，包含 `mod.rs`、`command.rs`、`repository.rs`、`history.rs`、`diff.rs`、`push.rs`、`parse.rs`、`tests.rs`。
- `src/routes/+page.svelte` 的非视觉逻辑已下沉到 `src/lib/tickgit/page-state.ts`、`push-events.ts`、`repository-actions.ts`，并补充对应 Vitest。
- 本地已通过完整质量门禁：`pnpm test:run`、`pnpm typecheck`、`pnpm build`、`pnpm format:check`、`cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`、`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`、`cargo test --manifest-path src-tauri/Cargo.toml`。

### 5.1 拆分 `src-tauri/src/git.rs`

现状：

- `src-tauri/src/git.rs` 已约 1600 行。
- 同时承担 Git 命令封装、仓库校验、分支状态、history、diff、push、安全路径计算和大量测试。

建议拆分方向：

```text
src-tauri/src/git/
  mod.rs          # 对外入口与公共组合逻辑
  command.rs      # git command 执行封装
  repository.rs   # repo path / validate / branch / upstream
  history.rs      # commit history / file list / meta
  diff.rs         # file diff / shortstat
  push.rs         # push current / push to commit / safe target
  parse.rs        # 文本解析函数
```

拆分原则：

- 不改变 command 对外行为。
- 先移动代码，再做少量命名整理。
- 每一步拆分后都跑 Rust 测试。

验收：

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

### 5.2 拆分 `src/routes/+page.svelte`

现状：

- `+page.svelte` 已约 879 行。
- 页面内同时管理仓库、分支、历史、详情、diff、拖拽、resize、toast、push overlay 和 Tauri event。

建议优先抽出：

- `src/lib/tickgit/page-state.ts`
  - 页面状态归类类型
  - loading / push / selection 相关纯函数
- `src/lib/tickgit/push-events.ts`
  - push event payload 到 UI state 的转换
  - overlay 自动关闭规则
- `src/lib/tickgit/repository-actions.ts`
  - bootstrap / loadRepositoryState / refreshRepositories 等 orchestration

注意：

- 不建议此阶段引入全局状态库。
- 优先保持现有 Svelte 组件结构，只减小页面文件职责。

验收：

```bash
pnpm test:run
pnpm typecheck
pnpm build
```

### 5.3 清理未使用或遗留 command/API

现象：

- 前端当前主要使用异步 job 版 push。
- `api.pushCurrentBranch`、同步 `push_to_commit` command 等可能是早期遗留能力。

建议：

- 确认是否仍需保留同步 push command。
- 如果不需要：
  - 从 `api.ts` 移除未使用方法
  - 从 `commands.rs` 移除对应 Tauri command
  - 从 `invoke_handler` 移除注册
  - 保留 Rust 内部函数供 job 使用即可

收益：

- 减少前后端协议面。
- 避免同步 command 绕过后台任务 gate。

---

## 6. P2：核心体验增强

完成状态：6.4 Diff 大文件保护与 6.2 Step push plan preview 已于 2026-06-04 完成。6.1 主要目标已由 P0.1 覆盖，6.3 继续暂缓。

完成摘要：

- `get_commit_file_diff` 保持 command 名不变，返回 `CommitFileDiffResult`：`text`、`isBinary`、`isImage`、`isTooLarge`、`truncated`、`byteCount`、`lineCount`。
- Rust diff 模块按 `1 MiB` patch 字节数与 `5000` 变更行阈值进行保护，二进制/图片/超大 diff 不返回完整 patch。
- 前端 Diff Viewer 对 binary、image、too large、hide whitespace empty diff 均展示明确降级状态，split rows 仅在 split 模式构建。
- 新增 `get_step_push_plan(repo_path, target_hash)` 后端接口，返回当前分支、目标 commit、旧到新的计划列表、`available` 与结构化 blocked reason。
- 前端右键 Step Push 改为先请求后端 plan 并展示 `StepPushPlanDialog`，用户确认后使用同一份 plan hashes 启动 `startStepPush`。
- `start_step_push` 保留原有后端二次校验，避免 stale plan 或前端篡改绕过安全检查。

本地验收命令：

```bash
rtk pnpm test:run
rtk proxy pnpm typecheck
rtk pnpm build
rtk pnpm format:check
rtk cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
rtk cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
rtk cargo test --manifest-path src-tauri/Cargo.toml
```

### 6.1 behind/diverged 同步引导

状态：主要目标已由 P0.1 覆盖，本轮不再作为独立 P2 实施项。

已覆盖：

- behind / diverged 时禁用推送。
- 本地未推送 commit 继续可见，但不标记为 safe push target。
- UI 提示用户使用 GitHub Desktop / SourceTree 同步后回到 TickGit 刷新。
- TickGit 不内置 pull / merge / rebase。

后续仅保留为可选体验增强：

- 更细地区分 behind only、ahead + behind / diverged、upstream missing、upstream is not origin 的文案。
- fetch 后展示远端新增 commit。

### 6.2 Step push plan preview

状态：已完成。

已实现：

- 新增后端 command `get_step_push_plan(repo_path, target_hash)`。
- 返回目标 commit、当前分支、旧到新的 step push commit 列表、`available` 与结构化 blocked reason。
- 前端不再使用本地 `buildStepPushHashes` 构建队列，右键 Step Push 后先展示后端 plan 预览。
- 用户确认后使用 plan 中的 hashes 调用 `startStepPush`。
- `start_step_push` 继续在 job 启动前二次校验 hashes，避免 stale plan 或篡改请求绕过后端安全策略。

已验证：

- safe first-parent target 返回旧到新的 plan。
- merge 侧支 commit 不出现在 plan 中，并返回不可推送原因。
- behind/diverged、missing upstream、non-origin upstream 返回结构化 blocked reason。
- 前端 preview 使用后端返回顺序，blocked plan 不会启动 step push，原 progress overlay 行为保持不变。

### 6.3 推送任务取消与状态查询

状态：暂缓，本轮不实施。

暂缓原因：

- 取消只能在下一步 push 前生效，无法安全强杀正在执行的 `git push`。
- running job query / cancel 会扩大后台任务状态模型和前后端 event 协议。
- 当前优先处理更直接的稳定性风险和 step push plan 真相来源问题。

触发条件：

- 用户反馈误触分步推送后需要停止。
- 窗口刷新或事件丢失导致推送 UI 状态无法恢复。

### 6.4 Diff 大文件保护

状态：已完成。

已实现：

- `get_commit_file_diff` 返回 `CommitFileDiffResult`，包含文本与 binary/image/tooLarge/truncated/byteCount/lineCount metadata。
- Rust 侧默认保护阈值：最大 patch 字节数 `1 MiB`，最大变更行数 `5000`。
- 图片按常见扩展名识别：png/jpg/jpeg/gif/webp/svg/bmp/ico/avif。
- 二进制、图片、超大 diff 不返回完整 patch，避免前端误展示空白或解析大文本。
- hide whitespace 后 empty diff 保持正常空文本结果，不误标 too large/binary。
- 前端 Diff Viewer 展示 binary、image、too large 与 whitespace-only 的明确降级提示。
- Split rows 仅在 `diffViewMode === "split"` 且 diff ready 时构建。

已验证：

- 普通文本 diff 返回 text。
- 二进制文件标记 `isBinary`。
- 图片文件标记 `isImage`。
- 超大 diff 标记 `isTooLarge` 与 `truncated`。
- hide whitespace empty diff 不标记 too large/binary。

---

## 7. P3：产品能力扩展

### 7.1 工作区变更视图

目标：

- 补齐接近 GitHub Desktop 的基础体验。

已实施（2026-06-04）：

- 显示 staged / unstaged / untracked 工作区变更，覆盖 modified / added / deleted / untracked 等状态
- 工作区文件 Diff 复用现有 Unified / Split、Hide Whitespace 与 binary / image / tooLarge 降级保护
- 支持文件级 stage / unstage
- 支持 commit message 输入并提交已 staged 内容

本轮明确不纳入：

- hunk / 行级暂存
- discard changes
- 冲突解决
- pull / merge / rebase

注意：

- Git 逻辑仍应全部在 Rust 侧。

已验证：

- Rust 工作区 status / diff / stage / unstage / commit 行为测试通过。
- 前端 workspace 选择、guard、提交 effect 与页面 helper 测试通过。
- 完整门禁见本节实施提交。

### 7.2 仓库管理能力

已实施（2026-06-05）：

- 从列表移除仓库：只删除 TickGit 记录，不删除磁盘文件；移除当前仓库后自动选中最近打开的剩余仓库
- 仓库路径失效提示：missing / invalid 项保留在列表中，并在当前仓库失效时显示修复入口
- 最近打开排序优化：后端按 `lastOpenedAt desc` 返回，时间相同时按路径稳定排序
- 重新定位已移动仓库：通过系统文件夹选择器选择新路径，后端校验有效 Git work tree 后替换记录
- 搜索仓库：前端按仓库名称和路径进行大小写不敏感过滤，并补充 helper 测试

已验证：

- Rust repo store 状态、移除、重新定位、排序测试通过。
- 前端仓库搜索、状态文案、管理 guard、失效 current bootstrap 测试通过。
- 完整门禁见本节实施提交。

### 7.3 历史和 Diff 增强

建议功能：

- Commit 搜索
- 文件路径过滤
- 作者过滤
- 语法高亮
- 图片 diff
- Copy file path / copy diff hunk

### 7.4 国际化与文案统一

现状：

- UI 中英文混用。
- 错误消息多为中文，按钮和部分状态为英文。

建议：

- 先统一产品默认语言。
- 若目标用户包含中英文用户，再引入轻量 i18n。

---

## 8. 发布工程化优化

### 8.1 发布脚本与 release workflow 对齐

现状：

- `scripts/release-version.sh` 会推当前分支。
- `release.yml` 要求 tag commit 可从 `origin/main` 到达。

风险：

- 如果在非 main 分支运行发布脚本，脚本会成功 push branch/tag，但 release workflow 会失败。

建议：

- 发布脚本强制要求当前分支为 `main`。
- 或允许通过参数指定发布分支，并与 workflow 校验保持一致。

验收：

- 非 main 分支执行发布脚本会在本地提前失败。

### 8.2 依赖升级策略

本次 `pnpm outdated` 查询显示：

- 多数依赖是小版本落后
- `vite`、`vitest`、`typescript`、`@sveltejs/vite-plugin-svelte`、`prettier-plugin-svelte` 存在跨大版本更新

建议：

- Tauri 2 相关依赖优先保持前后端 minor 对齐。
- 小版本升级可独立一批。
- 大版本升级单独分支处理，并完整跑：

```bash
pnpm test:run
pnpm typecheck
pnpm build
pnpm format:check
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

不建议在功能开发 PR 中顺手升级大版本依赖。

### 8.3 Tauri 安全加固

建议：

- 评估是否可以从 `csp: null` 改为最小可用 CSP。
- 清理未使用 command，缩小 webview 可调用面。
- 对所有会执行 Git 写操作的 command 做后端强校验，而不是只依赖前端 disabled 状态。

---

## 9. 推荐执行顺序

### 第一阶段：质量基线修复

1. 修正 behind/diverged safe push 测试与文案
2. 收紧后端 push branch 参数
3. 修复 `pnpm format:check`
4. 新增 CI

完成标准：

- 所有默认验证命令全绿
- main / PR 自动跑 CI

### 第二阶段：复杂度拆分

1. 拆 `src-tauri/src/git.rs`
2. 拆 `src/routes/+page.svelte`
3. 清理未使用同步 command/API

完成标准：

- 无行为变化
- 测试全绿
- 文档无需因纯拆分而改变约束

### 第三阶段：推送体验增强

1. behind/diverged 同步引导
2. step push plan preview
3. 推送任务取消与状态恢复

完成标准：

- 用户在危险分支状态下不会误操作
- step push 前能看到明确计划
- 后端仍是安全策略唯一真相来源

### 第四阶段：GitHub Desktop 体验补齐

1. 工作区变更视图
2. staged / unstaged
3. commit 创建
4. 历史搜索与 Diff 增强

完成标准：

- 能覆盖日常“查看历史 + 查看改动 + 提交 + 推送”的完整桌面工作流

---

## 10. 每个任务的默认交付要求

每个后续优化任务完成时，应说明：

- 改了什么
- 为什么这样改
- 跑了哪些验证
- 哪些验证未跑以及原因
- 是否需要更新 `README.md` / `ARCHITECTURE.md` / `AI_DEVELOPMENT.md`

默认验证优先级：

```bash
pnpm test:run
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
pnpm typecheck
pnpm build
pnpm format:check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

可以按改动范围裁剪，但需要在交付说明中明确说明。
