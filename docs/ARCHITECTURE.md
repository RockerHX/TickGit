# TickGit 架构文档

## 1. 文档定位

本文档定义 TickGit 的核心架构、模块边界、数据流与扩展约束。

后续开发默认遵循本文档，只有在以下情况才允许更新：

- 架构分层发生变化
- 核心技术选型变化
- 公共接口约束变化
- 状态管理、数据流或模块职责出现结构性调整

如果只是修 Bug、补 UI、优化样式、微调实现细节，**通常不需要更新本文档**。

---

## 2. 产品目标

TickGit 是一个面向桌面端的 Git 增强工具，核心目标不是替代所有 Git 客户端，而是聚焦于：

- 清晰展示当前仓库的 Commit 历史
- 识别哪些 Commit 已推送、哪些仍在本地
- 支持基于未推送 Commit 的精细推送策略
- 提供“分步提交 Commit”这一核心特色能力

---

## 3. 核心架构原则

### 3.1 Git 能力统一在 Rust 端

所有 Git 核心行为必须由 Rust 端统一执行，前端不能直接访问系统 Git。

允许的方式：

- Rust 使用 `std::process::Command`
- 调用系统原生 `git`

禁止的方式：

- 前端直接执行 Git 命令
- 引入 `libgit2` 或其他复杂 Git C 绑定作为主方案
- 在多个层重复拼装 Git 命令

### 3.2 前后端通过 Tauri 明确通信

TickGit 采用两种通信方式：

- **Command**：请求/响应型调用
- **Event**：后台异步进度推送

约束：

- 前端只通过 `src/lib/tauri/api.ts` 发起 invoke
- 前端只通过 `src/lib/tauri/events.ts` 监听事件
- 不允许把 `invoke("xxx")` 分散写在任意 UI 组件中

### 3.3 模块职责清晰，避免大一统文件

Rust 侧按职责拆分：

- `commands.rs`：Tauri 暴露入口
- `git.rs`：Git 命令封装、解析、校验
- `jobs.rs`：后台任务与进度事件
- `repo_store.rs`：仓库持久化
- `models.rs`：公共模型
- `error.rs`：统一错误

前端按职责拆分：

- 页面负责组合与状态编排
- 组件负责展示与交互
- `api.ts` 负责命令调用
- `events.ts` 负责事件订阅
- `types.ts` 负责 DTO 映射

### 3.4 先保证可维护性，再考虑抽象升级

当前项目规模仍处于 v1 阶段，因此：

- 优先使用清晰、直接、低心智负担的实现
- 暂不引入复杂状态管理库
- 暂不引入过度抽象的 service/repository/store 体系

只有当复杂度明显提升时，才允许升级抽象层级。

---

## 4. 当前系统分层

```text
UI Components
    ↓
Route Page State Orchestration
    ↓
Frontend Tauri API/Event Wrapper
    ↓
Tauri Commands
    ↓
Rust Domain Modules
    ├── git.rs
    ├── jobs.rs
    └── repo_store.rs
    ↓
System Git / Local JSON Store / Tauri Event Bus
```

---

## 5. 前端架构

## 5.1 入口与页面

当前主页面入口：

- `src/routes/+page.svelte`

该页面负责：

- 初始化仓库列表与当前仓库
- 拉取分支状态
- 分页加载 Commit 历史
- 管理当前选中的 Commit / 文件 / Diff
- 处理拖拽添加仓库
- 处理右键菜单动作
- 订阅分步提交进度事件

规范：

- 页面可以持有页面级状态
- 展示组件尽量保持无副作用
- 跨组件共享但仅限当前页面的状态，优先放在页面层而非额外建全局 store

## 5.2 前端模块职责

### `src/lib/components/`

用于放置可复用 UI 组件。

当前职责包括：

- 仓库切换器
- 左侧 Commit 历史列表
- 右侧 Commit 详情面板
- 自绘右键菜单
- 拖拽覆盖层
- Toast 提示
- 分步提交进度层

规则：

- 组件尽量通过 `props + events` 通信
- 组件内不要直接调用后端命令，除非它是一个非常明确的基础交互组件且调用被批准

### `src/lib/tauri/api.ts`

唯一的前端命令调用入口。

新增 command 时：

- 必须先在 Rust 侧定义
- 再在 `api.ts` 中补充统一方法
- 最后由页面或业务层调用

### `src/lib/tauri/events.ts`

唯一的前端事件订阅入口。

当前规范事件：

- `step-push-progress`
- `step-push-finished`
- `step-push-failed`

后续新增后台事件时，也必须先在此集中封装。

### `src/lib/types.ts`

定义前端对 Rust DTO 的映射类型。

规则：

- 字段命名保持与 Tauri 序列化后的 camelCase 一致
- 接口变更时必须同步更新

---

## 6. Rust / Tauri 后端架构

## 6.1 `commands.rs`

只负责暴露 `#[tauri::command]`。

禁止：

- 在 command 层编写复杂业务逻辑
- 在 command 层重复解析 Git 输出

command 层应尽量只是参数接收与模块转发。

## 6.2 `git.rs`

这是当前最核心的领域模块，负责：

- 仓库有效性校验
- 当前分支与 upstream 状态判断
- ahead / behind 计算
- 未推送 Commit 判定
- Commit 历史分页
- Commit 文件变更查询
- 单文件 Diff 查询
- 常规推送
- 推送到指定 Commit

规则：

- 所有 Git 命令统一从这里进入
- 命令失败时统一转成 `AppError`
- Git 参数拼装必须集中，避免在其他文件分散拼字符串

## 6.3 `jobs.rs`

负责后台任务模型，当前核心任务为：

- 分步提交 Commit

当前约束：

- 单任务模式
- 不可取消
- 每次成功推送后 sleep 一段时间
- 通过 Tauri Event 向前端回传实时进度

未来若扩展取消、队列、任务中心，也必须以该模块为中心演进。

## 6.4 `repo_store.rs`

负责仓库持久化管理。

当前设计：

- 使用应用配置目录下的 JSON 文件
- 存储仓库列表
- 存储当前选中仓库
- 写操作受 `Mutex` 保护

当前不引入数据库，不引入外部存储插件。

## 6.5 `models.rs`

统一定义后端数据模型，包括：

- RepositorySummary
- BranchStatus
- CommitHistoryPage
- CommitListItem
- CommitFileChange
- StepPushRequest / Progress / Finished / Failed
- RepositoryConfig

规则：

- 一切跨模块流转的稳定结构都应优先进入 `models.rs`

## 6.6 `error.rs`

统一错误结构：

- `code`
- `message`

规则：

- 面向前端返回的错误必须保持结构统一
- 不要把裸字符串错误散落在 command 边界

---

## 7. 核心数据流

## 7.1 应用启动

1. 前端读取仓库列表
2. 前端读取当前仓库
3. 若存在当前仓库，则获取分支状态
4. 拉取第一页 Commit 历史
5. 自动选中第一条 Commit
6. 拉取该 Commit 的文件列表与首个文件 Diff

## 7.2 拖拽添加仓库

1. 前端监听窗口拖拽事件
2. 接收到目录路径后调用 `add_repository`
3. Rust 校验是否为 Git 仓库
4. 写入本地 JSON
5. 前端设置为当前仓库
6. 全量刷新仓库状态与历史列表

## 7.3 历史分页

1. 前端通过 `get_commit_history(repoPath, skip, limit)` 拉取
2. Rust 使用 `git log --skip <N> -n <M>`
3. 基于 `upstream..HEAD` 判定未推送 Commit
4. 前端滚动到底部继续加载下一页

当前实现为了确保未推送 Commit 区段可被完整操作，会在首次进入仓库时尽量继续拉取，直到覆盖当前 `aheadCount` 对应的未推送范围。

## 7.4 分步提交 Commit

1. 前端从当前未推送列表中计算目标 hash 数组
2. 顺序要求为 **旧 → 新**
3. 调用 `start_step_push`
4. Rust 后台线程逐个执行：
   - `git push origin <hash>:refs/heads/<branch>`
5. 每次成功后发进度事件
6. 任务完成后发完成事件
7. 任意一步失败则发失败事件并终止任务
8. 前端刷新分支状态与 Commit 历史

---

## 8. 状态与持久化

## 8.1 持久化状态

当前持久化内容只有：

- 仓库列表
- 当前选中仓库

持久化位置：

- Tauri 应用配置目录
- 文件名：`repositories.json`

## 8.2 运行时状态

前端运行时状态主要在 `+page.svelte`：

- 当前仓库
- 分支状态
- Commit 列表
- 当前选中 Commit
- 文件列表
- Diff 内容
- Toast
- 右键菜单状态
- 分步提交进度状态

后端运行时状态主要包括：

- `RepositoryStoreState`
- `StepPushManager`

---

## 9. 当前明确约束

以下约束在没有正式决策变更前视为固定规范：

- 只支持当前检出分支
- 远端固定为 `origin`
- 未推送判断依赖 upstream
- 无 upstream、无 origin、detached HEAD 时禁用推送相关动作
- 不支持分步任务取消
- 不支持多任务队列
- 不支持前端直接操作 Git
- 不引入 libgit2 方案

---

## 10. 扩展规则

未来新增功能时，按以下顺序判断放置位置：

### 新的 Git 查询 / Git 操作

放入：

- `src-tauri/src/git.rs`

### 新的后台异步任务

放入：

- `src-tauri/src/jobs.rs`

### 新的持久化配置

优先放入：

- `src-tauri/src/repo_store.rs`

若配置结构明显膨胀，再单独拆出新模块。

### 新的前后端命令接口

需要同步修改：

- `src-tauri/src/commands.rs`
- `src-tauri/src/models.rs`
- `src/lib/tauri/api.ts`
- `src/lib/types.ts`

---

## 11. 文档维护规则

以下变化必须更新本文档：

- 新增核心模块
- 调整模块职责边界
- 修改持久化方案
- 修改 Git 执行策略
- 修改后台任务模型
- 修改全局状态管理方式

以下变化通常不需要更新本文档：

- 纯样式调整
- 组件内部重构
- Bug 修复
- 非结构性命令优化

本文档是 TickGit 的架构基线。
