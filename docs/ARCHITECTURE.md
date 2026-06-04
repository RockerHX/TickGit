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
- 分支状态
- Commit 历史分页
- 当前选中 Commit / 文件 / Diff
- 拖拽添加仓库
- 右键动作
- 分步提交进度状态

当前项目未引入全局状态库，页面级主状态保留在这里；非视觉的页面辅助逻辑优先下沉到普通 TypeScript helper，而不是继续堆积在页面文件内。

### `src/lib/tickgit/*`

页面编排层使用的非视觉 helper，负责：

- 仓库启动与快照加载辅助
- 分步提交 hash 计算
- unified diff 解析与 split 视图数据派生
- 错误消息归一化
- 页面纯函数单元测试承载

### `src/lib/components/*`

负责展示和局部交互，不直接承担后端协议封装。

当前 Diff Viewer 约束：

- 前端先解析 unified diff，再派生 unified / split 两种视图
- hide whitespace 会重新请求后端 diff，而不是在前端做字符串过滤
- 当前只覆盖文本 diff，不处理图片 / 二进制专用展示

### `src/lib/tauri/api.ts`

前端唯一 command 调用入口。

### `src/lib/tauri/events.ts`

前端唯一事件监听入口，当前仅封装分步提交相关事件。

### `src/lib/types.ts`

前端 DTO 类型定义，需与 Rust 序列化结果保持一致。

---

## 4. Rust 职责

### `src-tauri/src/commands.rs`

Tauri command 边界层，只做参数接收和模块转发。

### `src-tauri/src/git.rs`

Git 领域核心模块，统一负责：

- 仓库校验
- 当前分支 / upstream 状态
- ahead / behind 计算
- 全量未推送 Commit 与 first-parent 安全推送集合计算
- Commit 历史、文件列表、Diff
- 常规推送
- 推送到指定 Commit
- 刷新 origin 远端跟踪信息，用于判断 ahead / behind 状态
- 统一封装 Git 命令执行环境，避免受本地语言、颜色、分页器和交互提示影响

所有 Git 操作都从这里进入。

Git 命令执行规则：

- 所有通过 `std::process::Command` 调用的 Git 命令必须设置固定环境：
  - `LC_ALL=C`
  - `GIT_TERMINAL_PROMPT=0`
- 所有需要解析纯文本输出的 Git 命令必须显式禁用分页器与颜色输出，避免解析受干扰
- 仓库有效性校验优先通过 Git 自身判断当前路径是否为 work tree，不依赖 `.git` 目录是否直接存在

### `src-tauri/src/jobs.rs`

后台任务模块。当前只负责分步提交：

- 单任务模式
- 不可取消
- 顺序推送 commit
- 通过 Tauri event 回传进度 / 完成 / 失败

### `src-tauri/src/repo_store.rs`

仓库持久化模块，负责：

- 仓库列表
- 当前选中仓库
- `repositories.json` 读写

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
- 本地分支落后远端或与远端分叉时禁用推送，并引导用户使用 GitHub Desktop / SourceTree 等外部工具同步
- `origin` 缺失、upstream 缺失或 detached HEAD 时禁用推送
- 前端不能直接执行 Git
- 不引入 `libgit2`
- Diff 当前为结构化文本视图，支持 unified / split 与 hide whitespace
- 分步提交为单任务、不可取消

---

## 6. 核心数据流

### 启动

1. 读取仓库列表与当前仓库
2. 刷新 origin 远端跟踪信息并读取分支状态（包含全量 ahead 数、behind 数与 safe step-push 数）
3. 拉取完整 Commit 历史，并标记哪些未推送 Commit 位于 first-parent 安全路径上
4. 自动加载当前选中 Commit 的文件与 Diff

### 分步提交

用户层面的目标是：

> 选中一个未推送 Commit 后，把到这个目标为止该推的 Commit 按旧到新顺序逐个推送。

为了保证这个体验在 merge / 分叉历史下依然安全成立，技术上不是简单把“所有未推送 Commit”按时间排序后硬推，而是按下面步骤处理：

1. 后端读取当前分支的 first-parent 历史
2. 后端同时计算：
   - 相对 upstream 的全量未推送 Commit 集合
   - 基于 first-parent 路径的“可安全分步推送”未推送 Commit 集合
3. 前端历史默认展示完整历史，但只在可安全路径上开放 step push / push to commit
4. 前端只在这条可安全路径上整理要推送的 commit hash（旧 -> 新）
5. 调用 `start_step_push`
6. 后端逐个执行 `git push origin <hash>:refs/heads/<branch>`
7. 每成功一步，就把远端分支推进到下一个安全目标
8. 通过 event 推送进度、完成或失败

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
- 用户可基于剩余未推送部分重新发起分步提交

---

## 7. 新功能放置规则

- 新的 Git 查询或操作：`src-tauri/src/git.rs`
- 新的后台异步任务：`src-tauri/src/jobs.rs`
- 新的持久化配置：优先 `src-tauri/src/repo_store.rs`
- 新的前后端接口：同步修改 `commands.rs`、`models.rs`、`api.ts`、`types.ts`
- 新的前端事件：同步修改 `jobs.rs`、`events.ts`、相关类型
