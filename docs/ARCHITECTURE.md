# TickGit 架构文档

## 1. 定位

本文档只定义 TickGit 当前稳定的架构边界、模块职责和固定约束。

只有在以下情况才更新本文档：

- 模块边界变化
- 数据流变化
- 持久化方案变化
- 后台任务模型变化
- Git 执行策略变化

普通 UI 调整、局部 Bug 修复、样式优化通常不需要改本文档。

---

## 2. 系统结构

TickGit 当前采用：

- 前端：SvelteKit + TypeScript
- 桌面壳：Tauri 2
- 后端：Rust
- Git 执行：Rust 调用系统原生 `git`
- 持久化：Tauri 配置目录下的 `repositories.json`

整体分层：

```text
Svelte 页面与组件
  -> src/lib/tauri/api.ts / events.ts
  -> Tauri commands
  -> Rust 领域模块
  -> system git / local json / tauri events
```

---

## 3. 前端职责

### `src/routes/+page.svelte`

当前页面主编排层，负责：

- 当前仓库与仓库列表
- 仓库搜索、失效路径提示、移除和重新定位
- 分支状态
- Commit 历史分页
- 当前选中 Commit / 文件 / Diff
- 拖拽添加仓库
- 右键动作
- 分步推送进度状态

当前项目未引入全局状态库，页面级主状态保留在这里；非视觉的页面辅助逻辑优先下沉到普通 TypeScript helper，而不是继续堆积在页面文件内。

### `src/lib/tickgit/*`

页面编排层使用的非视觉 helper，负责：

- `page-data.ts` / `repository-actions.ts`：仓库索引、远端刷新、快照加载、详情加载编排，以及 overview/details/diff 的 inflight 去重、LRU 缓存和 repository generation 失效
- `page-state.ts`：loading / push / selection / focus revision refresh 等页面 guard 纯函数
- `push-events.ts`：push / step push event payload 到 UI state 的转换和 overlay 关闭规则
- `page-helpers.ts`：错误消息和 toast 数据辅助
- `step-push-plan.ts`：分步推送 plan hashes 提取与确认后启动 job 的状态转换
- `diff.ts`：unified diff 解析、diff 降级状态与 split 视图数据派生
- 对应 Vitest 单元测试承载

### `src/lib/components/*`

负责展示和局部交互，不直接承担后端协议封装。

当前不提供工作区 Changes 视图，不读取或展示 staged / unstaged / untracked 文件，也不提供 stage / unstage / git commit / discard / 冲突处理能力。

当前 Diff Viewer 约束：

- 后端 diff 返回 `CommitFileDiffResult`，包含文本与 binary/image/tooLarge/truncated/byteCount/lineCount metadata
- 大 diff 在前端使用虚拟滚动，只渲染可视区与 overscan 行；文本解析和高亮优先走 worker，worker 失败时才做主线程降级
- 前端先判断 diff metadata，binary/image/tooLarge 进入降级提示，不解析为文本 diff
- 文本 diff 再解析 unified diff，并派生 unified / split 两种视图
- split rows 仅在 split 模式且 diff ready 时构建
- hide whitespace 会重新请求后端 diff，而不是在前端做字符串过滤

### `src/lib/tauri/api.ts`

前端唯一 command 调用入口。

### `src/lib/tauri/events.ts`

前端唯一事件监听入口，封装 step push 与 push to commit/current branch 的进度、完成、失败事件。

### `src/lib/types.ts`

前端 DTO 类型定义，需与 Rust 序列化结果保持一致。

---

## 4. Rust 职责

### `src-tauri/src/commands.rs`

Tauri command 边界层，只做参数接收和模块转发。

### `src-tauri/src/git/`

Git 领域核心模块目录，统一负责 Git 相关查询、解析和安全推送操作。当前模块边界：

```text
src-tauri/src/git/
  mod.rs          # 对外 re-export 与 Git 固定约束常量
  command.rs      # git command 执行封装
  repository.rs   # repo path / validate / branch / upstream / tracking refresh
  history.rs      # commit history / file list / meta
  diff.rs         # file diff / metadata / 大文件保护
  push.rs         # push current / push to commit / step push plan / safe target
  parse.rs        # 文本解析函数
  tests.rs        # Git 行为集成测试
```

所有 Git 操作都从这里进入，并由 `commands.rs` 或 `jobs.rs` 调用公开函数。

Git 命令执行规则：

- 所有通过 `std::process::Command` 调用的 Git 命令必须设置固定环境：
  - `LC_ALL=C`
  - `GIT_TERMINAL_PROMPT=0`
- 所有需要解析纯文本输出的 Git 命令必须显式禁用分页器与颜色输出，避免解析受干扰
- 仓库有效性校验优先通过 Git 自身判断当前路径是否为 work tree，不依赖 `.git` 目录是否直接存在
- 普通历史分页使用 `limit + 1` 判断 `hasMore`，首屏不阻塞等待 `rev-list --count`
- 历史过滤只将 author/message 作为 Git 预过滤下推；query/filePath 仍保留 Rust 端现有语义
- commit meta 的 body 与 shortstat 使用一次 `git show --shortstat --format=...` 合并读取
- diff 请求可携带 `baseHash` 复用 `CommitListItem.parents[0]`，未传时后端保留旧 parent 查询兼容逻辑
- safe step-push 路径使用一次 ancestor 判断结合 `rev-list --first-parent --ancestry-path --reverse` 计算，避免逐 hash `merge-base`

### `src-tauri/src/jobs.rs`

后台任务模块。当前负责异步推送任务：

- 单任务 gate，避免多个 push job 并发执行
- 分步推送不可取消
- 分步推送按 plan hashes 顺序推送 commit
- push current branch / push to commit command 只做参数形状检查和任务 gate reserve，立即返回 job id；后台线程先发 `push-to-commit-progress` 的 `preparing`，再执行 Git 校验与实际 push
- push to commit/current branch 通过 `push-to-commit-progress` / finished / failed event 回传准备中、运行、完成、失败
- step push 启动前做一次当前分支与安全路径校验；job 循环内复用预检结果，只保留当前分支 guard 和实际 push
- 分步推送不可取消

### `src-tauri/src/repo_store.rs`

仓库持久化模块，负责：

- 仓库列表
- 当前选中仓库
- 仓库 runtime 状态标记：available / missing / invalid（列表刷新使用固定 4 worker 并发限流，返回顺序保持 store 排序）
- 从列表移除仓库
- 重新定位已移动仓库
- `repositories.json` 读写

持久化文件只保存仓库记录本身；missing / invalid 状态在 list / get current 时动态计算，不写入配置文件。

### `src-tauri/src/models.rs`

前后端共享 DTO。

### `src-tauri/src/error.rs`

统一错误结构：`code` + `message`。

---

## 5. 当前固定约束

在有新决策前，默认以下约束成立：

- 只支持当前检出分支
- 远端固定为 `origin`
- 未推送判断依赖 upstream
- 仅通过 `fetch --prune origin` 刷新远端跟踪信息；不内置 `git pull` / `merge` / `rebase`
- 不内置 `git commit` / `stage` / `unstage`，不提供工作区 Changes 视图
- 本地分支落后远端或与远端分叉时禁用推送，并引导用户使用 GitHub Desktop / SourceTree 等外部工具同步
- `origin` 缺失、upstream 缺失或 detached HEAD 时禁用推送
- 前端不能直接执行 Git
- 不引入 `libgit2`
- Diff 为结构化文本视图，支持 unified / split 与 hide whitespace，并对 binary/image/tooLarge 做降级展示
- 仓库列表保留失效路径，通过 runtime status 标记 missing / invalid，不自动清理用户配置
- 从 TickGit 移除仓库只移除持久化记录，不删除磁盘文件
- 重新定位仓库通过 Tauri dialog 选择目录，后端仍负责校验新路径是 Git work tree
- 分步推送为单任务、不可取消
- Step push plan 由后端生成，前端只展示并在用户确认后传回 plan hashes；job 启动前仍由后端二次校验

### Tauri 插件与权限

当前启用插件：

- `tauri-plugin-opener`：保留默认 opener 能力
- `tauri-plugin-dialog`：用于仓库重新定位时选择本地目录

`src-tauri/capabilities/default.json` 只开放当前需要的 dialog open 权限。

---

## 6. 核心数据流

### 启动与焦点刷新

1. 读取仓库列表与当前仓库
2. 读取分支状态（包含全量 ahead 数、behind 数与 safe step-push 数）
3. 拉取当前页 Commit 历史，并标记哪些未推送 Commit 位于 first-parent 安全路径上
4. 自动加载当前选中 Commit 的文件与 Diff
5. 前端记录轻量 `RepositoryRevision { head, branch, upstream }` 作为 focus refresh 基线
6. 应用重新获得焦点时最多 60 秒检查一次 revision；revision 未变则跳过完整仓库刷新，变化时提升 repository generation 并刷新

### 分步推送

用户层面的目标是：

> 选中一个未推送 Commit 后，把到这个目标为止该推的 Commit 按旧到新顺序逐个推送。

为了保证这个体验在 merge / 分叉历史下依然安全成立，技术上不是简单把“所有未推送 Commit”按时间排序后硬推，而是按下面步骤处理：

1. 后端读取当前分支的 first-parent 历史
2. 后端同时计算：
   - 相对 upstream 的全量未推送 Commit 集合
   - 基于 first-parent 路径的“可安全分步推送”未推送 Commit 集合
3. 前端历史默认展示完整历史，但只在可安全路径上开放 step push / push to commit
4. 用户右键 Step Push 时，前端调用 `get_step_push_plan(repo_path, target_hash)`
5. 后端刷新 origin tracking，判断 behind/diverged/upstream/origin/first-parent 安全性
6. 可推送时，后端批量读取旧到新的 plan items；不可推送时，返回结构化 blocked reason
7. 前端展示 `StepPushPlanDialog`，用户确认后把同一份 plan hashes 传给 `start_step_push`
8. `start_step_push` 启动前再次校验 hashes 必须等于后端当前安全路径的连续前缀，避免 stale plan 或篡改请求绕过安全策略
9. 后端逐个执行预检后的 `git push origin <hash>:refs/heads/<branch>`，不在循环内重复 fetch / safe path 全量计算
10. 每成功一步，就把远端分支推进到下一个安全目标
11. 通过 event 推送进度、完成或失败

这样做的核心目的，是保证分步推送始终沿着一条真正可 fast-forward 的主线前进，而不是把 merge 进来的侧支 Commit 误塞进队列。

简单例子：

```text
A --- B --- M   (当前分支 HEAD)
 \         /
  \--- C --/
```

假设：

- `A` 已经在远端
- 本地额外有 `B`、`C`、`M`
- 用户选中 `M` 做分步推送

用户感知上会理解成：

> 系统把到 `M` 为止该推的内容，从旧到新一步步推上去。

技术上实际执行的是：

- 识别当前分支可安全分步推送的主线：`B -> M`
- `C` 虽然是未推送 Commit，但它是 merge 侧支，不会被当成线性 step-push 目标
- 最终顺序是先推 `B`，再推 `M`

这样就不会再因为把 `C` 错误插入队列而触发 `non-fast-forward`。

失败恢复规则：

- 任一步推送失败后，任务立即安全终止，并通过失败事件通知前端
- 已成功推送到远端的 Commit 视为已完成，不要求从头开始
- 前端刷新仓库状态后，重新计算剩余未推送 Commit 列表
- 用户可基于剩余未推送部分重新发起分步推送

---

## 7. 新功能放置规则

- 新的 Git 查询或安全推送操作：`src-tauri/src/git/`，按 command / repository / history / diff / push / parse 职责放置；除非先更新架构边界，否则不得新增 pull / merge / rebase / commit / stage / unstage
- 新的后台异步任务：`src-tauri/src/jobs.rs`
- 新的持久化配置：优先 `src-tauri/src/repo_store.rs`
- 新的前后端接口：同步修改 `commands.rs`、`models.rs`、`api.ts`、`types.ts`
- 新的前端事件：同步修改 `jobs.rs`、`events.ts`、相关类型
