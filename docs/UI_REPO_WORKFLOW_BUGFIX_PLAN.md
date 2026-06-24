# UI、仓库操作与分步推送 Bug 修复计划

日期：2026-06-24

## 1. 背景

本计划覆盖当前反馈的 5 类问题：

1. 页面文本只能右键选词，不能像 macOS 三指拖动 / Windows 左键拖拽那样选中连续文本。
2. 当前仓库区域需要右键菜单，支持打开仓库文件夹、在仓库路径打开 Terminal，并参考截图补充常用仓库动作。
3. 某些仓库切换分支失败时，用户不知道是否由未提交文件阻止。
4. 右键点击“分步推送”后，等待分步推送预览窗口出现期间疑似 UI 卡死。
5. 点击“开始分步推送”后，开始推送前也疑似 UI 卡死。

目标不是一次性重做交互，而是把阻塞用户的 bug 和反馈缺口先修掉，并保留现有架构边界：前端不直接执行 Git，Git / 文件系统操作仍通过 Tauri Rust 侧完成。

## 2. 当前代码观察

- 当前仓库选择区域由 `src/lib/components/RepositorySwitcher.svelte` 渲染；现在只有左键打开下拉、切换、重新定位和移除，没有仓库级右键菜单。
- Commit 右键菜单由 `src/lib/components/CommitContextMenu.svelte` 提供，分步推送入口在这里。
- 分支切换从 `src/routes/+page.svelte` 调用 `api.checkoutBranch()`，后端 `src-tauri/src/git/repository.rs` 直接执行 `git checkout <branch>`。
- 分支切换失败会进入前端 toast，但目前主要展示 Git 原始错误，不会结构化说明“本地未提交变更阻止切换”。
- 分步推送预览由 `startStepPush()` 先打开弹窗 loading 状态，再等待 `api.getStepPushPlan()`；后端 `get_step_push_plan` 会同步执行 fetch、branch status、safe path、dry-run push 和 log 等 Git 命令。
- “开始分步推送”调用 `start_step_push` 前，Rust command 内同步执行当前分支校验与 step push hashes 二次校验；这些校验包含 Git 命令，返回 job id 前可能阻塞。
- 项目已有 `src/lib/tickgit/preferences.ts` 和 `SettingsDialog.svelte`，适合承载“是否允许页面文本拖拽选择”的用户设置。

## 3. 修复原则

- 保持前端只做 UI 与 Tauri command 调用，不直接执行 shell / Git。
- 对能耗时的 Git 操作，前端必须先进入可见 loading 状态，后端尽量把重操作放入后台任务或异步 command。
- 错误信息优先结构化，避免只把 Git stderr 原样抛给用户。
- 仓库右键菜单先做常用动作，不引入 commit / stage / discard / pull / merge 等新 Git 工作流。
- 文本选择能力做成设置项，默认行为需兼顾现有按钮、列表点击、拖拽分隔条和 diff 交互。

## 4. 分阶段计划

### 阶段 A：文本拖拽选择设置

#### 目标

允许用户开启“页面文本可拖拽选择”，便于复制页面中的仓库名、路径、分支名、commit 信息和 diff 文本。

#### 方案

1. 在 `src/lib/tickgit/preferences.ts` 增加布尔偏好，例如 `textSelectionEnabled`，用 localStorage 持久化。
2. 在 `SettingsDialog.svelte` 增加设置项：
   - 名称：允许页面文本拖拽选择
   - 说明：开启后可以拖动选择页面文字；关闭后保持更接近应用控件的交互。
3. 在页面根容器挂载 class，例如：
   - 关闭：保持当前行为
   - 开启：对内容区域设置 `user-select: text`
4. 对必须保持操作手感的控件保留 `user-select: none`：
   - 按钮、下拉项、拖拽 resize handle
   - Commit 列表行可点击区域
   - 菜单遮罩与弹窗控制按钮
5. 对最需要复制的区域明确允许选择：
   - 仓库名 / 仓库路径
   - 分支名 / upstream
   - Commit hash / summary / author / body
   - 文件路径
   - Diff 文本行

#### 验收标准

- macOS 三指拖动或鼠标左键拖动可以选中普通文本。
- 开启后仍可正常点击按钮、切换仓库、切换分支、拖拽调整左侧面板宽度。
- 关闭后恢复当前默认交互。
- 设置重启后仍保留。

### 阶段 B：当前仓库右键菜单

#### 目标

在当前仓库卡片和仓库下拉列表项上支持右键菜单，提供截图中类似的仓库级动作。

#### 首批菜单项

建议按截图顺序实现：

1. Copy Repo Name
2. Copy Repo Path
3. View on GitHub（无 remote URL 或不是 GitHub 时置灰）
4. Open in Terminal
5. Reveal in Finder
6. Open in Visual Studio Code（检测不到 `code` 或无法打开时给 toast）
7. Remove...

“Create Alias”可作为后续增强项；如果当前项目没有 alias 数据模型，不建议和本轮 bugfix 混在一起。

#### 方案

1. 新增仓库右键菜单组件，例如 `RepositoryContextMenu.svelte`。
2. `RepositorySwitcher.svelte` 增加 `contextmenu` 事件：
   - 当前仓库卡片右键：目标为当前仓库。
   - 下拉列表项右键：目标为该列表项仓库。
3. 复用现有 `writeClipboardText()` 处理复制。
4. 增加 Tauri command：
   - `reveal_repository_in_file_manager(repo_path)`
   - `open_terminal_at_repository(repo_path)`
   - `open_repository_in_vscode(repo_path)`
   - 可选：`get_repository_remote_url(repo_path)` 或在后端直接返回 GitHub URL 可用性。
5. Rust 侧按平台实现：
   - macOS：Finder 使用 `open -R` 或 opener；Terminal 使用 `open -a Terminal <repo_path>`；VS Code 优先调用 `code <repo_path>`，失败时提示。
   - Windows：Explorer 使用 `explorer`; Terminal 使用 Windows Terminal / PowerShell fallback。
   - Linux：文件管理器使用 `xdg-open`; Terminal 需要按可用终端 fallback。
6. 所有路径先走现有仓库路径校验，missing / invalid 仓库菜单项只允许复制与移除，打开类动作置灰。

#### 验收标准

- 当前仓库卡片右键能打开菜单。
- 仓库下拉列表中任一仓库右键能打开菜单。
- Copy Repo Name / Path 能复制。
- Reveal in Finder 能定位到仓库。
- Open in Terminal 打开的终端当前目录为仓库路径。
- 不可用动作有明确置灰或 toast 原因。

### 阶段 C：分支切换失败原因提示

#### 目标

当本地未提交修改阻止分支切换时，明确告诉用户原因和处理方式，而不是只表现为“不能切换”。

#### 方案

1. 保持 `git checkout <branch>` 的真实语义，不强制在任何 dirty 状态下都禁止切换，因为 Git 允许不冲突的本地修改跨分支保留。
2. 在 Rust `checkout_branch` 中捕获 Git checkout 失败：
   - 如果 stderr 包含典型提示，例如 `Your local changes to the following files would be overwritten by checkout`，转换为结构化错误：
     - code: `checkout_blocked_by_local_changes`
     - message: `本地未提交修改会被目标分支覆盖，Git 已阻止切换。请先提交、暂存、stash 或丢弃这些修改后重试。`
   - 如果是未跟踪文件会被覆盖，使用 code: `checkout_blocked_by_untracked_files`。
   - 其他错误继续返回 Git 原始错误。
3. 前端 `getErrorMessage()` / i18n 增加上述 code 的中文文案。
4. 分支切换 UI 在失败 toast 中展示：
   - 目标分支名
   - 阻止原因
   - 建议动作：打开 Terminal / Reveal in Finder，用外部 Git 工具处理工作区。
5. 可选增强：失败 toast 增加“Open in Terminal”快捷按钮；如果当前 toast 组件不支持 action，先不做，避免扩大范围。

#### 针对反馈仓库的解释

`/Users/rockerhx/Desktop/Squad/otaServer/ota_admin` 只能在部分分支之间切换，回档变动后又能切换，符合 Git 的典型行为：未提交修改只有在会被目标分支覆盖时才阻止 checkout；切到不冲突的分支仍可能成功。因此需要提示用户“不是分支列表问题，而是本地工作区状态阻止了特定目标分支”。

#### 验收标准

- 有会被覆盖的 tracked 修改时，切换到冲突分支失败并显示结构化原因。
- 有会被覆盖的 untracked 文件时，显示对应原因。
- 不冲突的 dirty worktree 仍允许 Git 正常切换。
- 正常 clean worktree 切换不受影响。

### 阶段 D：分步推送预览弹窗卡顿

#### 目标

用户右键点击“分步推送”后，预览弹窗应立即出现 loading 状态；耗时的 plan 计算不能让主窗口看起来卡死。

#### 可能根因

`get_step_push_plan` 当前同步执行多条 Git 命令，包括远端刷新、状态检查、safe path 计算、dry-run push 和 commit item 读取。大仓库、慢远端或网络波动时，Tauri command 返回较慢。即使前端先设置了 `stepPushPlanOpen = true`，如果状态刷新没有先让浏览器完成一次渲染，用户会感觉右键点击后窗口冻结。

#### 方案

1. 前端在设置 `stepPushPlanOpen = true` 后，先 `await tick()` 或把请求延后到下一个 microtask / animation frame，确保 loading 弹窗先渲染。
2. 弹窗 loading 态增加明确文案：
   - `正在检查远端状态并生成分步推送预览...`
3. 为 `get_step_push_plan` 加性能日志或开发日志，统计每个 Git 步骤耗时：
   - `sync_origin_tracking`
   - `branch_status_for_path`
   - `safe_unpushed_hashes_in_push_order`
   - `ensure_remote_fast_forward_target`
   - `step_push_plan_items`
4. 如果确认 Rust 同步 command 会阻塞 Tauri UI 线程，改为 async command 或 `tauri::async_runtime::spawn_blocking` 包裹重 Git 操作。
5. 对重复点击同一 commit 的 plan 请求做短期 inflight 去重或缓存，但必须在分支 head / upstream 变化后失效。

#### 验收标准

- 点击“分步推送”后 100ms 内出现预览弹窗或 loading shell。
- 网络慢或 fetch 慢时，窗口仍可移动、按钮 hover 状态正常、loading 动画正常。
- plan 成功、blocked 和失败都能落到同一个弹窗里展示，不出现静默等待。

### 阶段 E：点击“开始分步推送”后的卡顿

#### 目标

点击确认后，UI 应立即显示“准备中 / 已启动”状态；后台推送校验和实际 push 不应让 UI 卡住。

#### 可能根因

`start_step_push` 当前在返回 job id 前同步执行：

1. `validate_current_branch`
2. `validate_step_push_hashes`
3. `ensure_safe_push_target`
4. fetch / branch status / dry-run / safe path 相关 Git 命令

这些步骤如果慢，前端会停留在提交按钮 loading 状态，且用户感觉 UI 卡住。

#### 方案

1. 调整后台任务边界：`start_step_push` command 只做参数形状检查、单任务 gate reserve，然后立即返回 job id。
2. 把当前分支校验和 step push hashes 二次校验移动到后台线程开始阶段。
3. 后台线程启动后先发 `step-push-progress`，状态为 `preparing`，前端 overlay 显示“正在校验分支和远端状态”。
4. 校验失败时发 `step-push-failed`，并释放 gate / running job。
5. 校验成功后再逐个 push，并继续发 `running` progress。
6. 前端 `confirmStepPushPlan()` 调用返回后立刻关闭 plan dialog，展示 step push overlay。
7. 确保 command 返回成功但后台校验失败时，用户仍能看到失败原因，不会误以为已经推送。

#### 验收标准

- 点击“开始分步推送”后 100ms 内关闭预览弹窗并出现 step push overlay。
- 慢 fetch / dry-run 时 overlay 显示 preparing，窗口无明显卡死。
- 后台校验失败时 overlay 显示失败原因。
- 已有 push job 运行时仍能立即返回 `push_busy` 并提示。

## 5. 测试计划

### 前端单元测试

- `preferences.test.ts`：
  - 新增文本选择设置的 parse / read / write。
- `page-state.test.ts` 或相关 helper 测试：
  - 分步推送 preparing / running 状态对按钮禁用逻辑的影响。
- 如果抽出仓库右键菜单纯函数：
  - missing / invalid 仓库的菜单项 enable / disable 规则。

### Rust 测试

- `src-tauri/src/git/tests.rs`：
  - dirty tracked file 阻止 checkout 时返回 `checkout_blocked_by_local_changes`。
  - untracked file 阻止 checkout 时返回 `checkout_blocked_by_untracked_files`。
  - dirty 但不冲突时 checkout 仍成功。
- `src-tauri/src/jobs.rs` 相关测试或集成测试：
  - `start_step_push` 尽快返回 job id。
  - 后台准备阶段校验失败会发 failed event 并释放 gate。

### 手工验证

1. macOS：
   - 三指拖动 / 鼠标拖动选择页面文本。
   - 当前仓库右键 Reveal in Finder。
   - 当前仓库右键 Open in Terminal，并确认 `pwd` 是仓库路径。
2. 反馈仓库：
   - 在 `/Users/rockerhx/Desktop/Squad/otaServer/ota_admin` 制造会冲突的本地修改，确认切 main / release 给出明确提示。
   - 回档后确认 main / release 可正常切换。
3. 分步推送：
   - 慢网络或模拟慢 Git 命令下，点击右键菜单后 loading 弹窗立即出现。
   - 点击开始后 overlay 立即进入 preparing。

## 6. 建议实施顺序

1. 先修分支切换错误提示，因为这是数据安全相关，且范围小。
2. 再修分步推送两个卡顿点，因为影响主流程体验。
3. 再做仓库右键菜单，先完成复制 / Finder / Terminal / Remove。
4. 最后做文本拖拽选择设置，因为需要较多 UI 回归，避免影响点击和拖拽交互。

## 7. 风险与取舍

- 文本选择开启后，可能影响列表点击和拖拽分隔条手感，因此必须做成可关闭设置。
- Terminal / VS Code 打开方式跨平台差异较大，首轮可优先保证 macOS，然后补 Windows / Linux fallback。
- 分步推送 command 后台化后，错误从“command reject”变成“event failed”，前端状态机必须覆盖这条路径。
- 不应为了提示未提交修改而新增工作区 Changes 视图；当前架构文档明确不支持 stage / discard / commit。
