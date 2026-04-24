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
- 错误消息归一化
- 页面纯函数单元测试承载

### `src/lib/components/*`

负责展示和局部交互，不直接承担后端协议封装。

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
- 未推送 Commit 判断
- Commit 历史、文件列表、Diff
- 常规推送
- 推送到指定 Commit
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
- `origin` 缺失、upstream 缺失或 detached HEAD 时禁用推送
- 前端不能直接执行 Git
- 不引入 `libgit2`
- Diff 当前为纯文本
- 分步提交为单任务、不可取消

---

## 6. 核心数据流

### 启动

1. 读取仓库列表与当前仓库
2. 读取分支状态
3. 拉取 Commit 历史
4. 自动加载当前选中 Commit 的文件与 Diff

### 分步提交

1. 前端整理要推送的 commit hash（旧 -> 新）
2. 调用 `start_step_push`
3. 后端逐个执行 `git push origin <hash>:refs/heads/<branch>`
4. 通过 event 推送进度、完成或失败

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
