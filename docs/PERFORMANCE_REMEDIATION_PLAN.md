# TickGit 性能卡顿分析与整改计划

> 基于 2026-06-22 当前工作区（包含未提交改动）的静态代码分析。本文只描述性能问题、风险和整改计划，不改变现有实现。

## 1. 总体结论

当前卡顿不是单一“Rust 后端阻塞主线程”问题，而是前端主线程、Tauri invoke 等待、Git 子进程数量和加载策略共同造成的体感卡顿。

已确认的核心问题：

1. **Diff Viewer 虚拟列表没有真正生效**：组件计算了 `visibleRows/topSpacerHeight/bottomSpacerHeight`，但模板仍然直接渲染全部 `parsedDiff.hunks` 或全部 `splitRows`。大 diff 会一次性创建大量 DOM，必然卡 UI。
2. **diff 解析/高亮链路未闭环**：worker 已返回 `line.html`，但模板没有消费它，反而调用当前文件内未定义的 `highlightedLineContent(...)`。这既是潜在构建问题，也说明“把高亮移到 worker”的优化没有实际落地。
3. **首屏和刷新存在重复请求与重复状态替换**：启动时会先应用缓存，再触发 commit 详情加载、仓库状态后台刷新和完整仓库状态刷新，短时间内多次替换 `commits/commitFiles/diffResult`，造成 UI 重排与 spinner 抖动。
4. **后端 Git 命令已多数放入 `spawn_blocking`，但 Git 子进程数量过多**：仓库概览、commit 详情、diff、step push plan 都由多个 `git` 子进程串行组成。UI 不一定被 Rust 主线程卡死，但用户必须等待 invoke 返回，表现就是“转菊花”。
5. **历史过滤是全仓库扫描**：启用过滤后会读取全量 `git log`，路径过滤还会先跑一遍全量 `git log --name-status`，在大仓库上会显著变慢。
6. **文件列表和 diff 行列表缺少 DOM 级虚拟化**：commit 文件列表对全部文件逐项渲染；diff 的虚拟化变量未被使用。巨型提交会导致文件列表和 diff 两侧同时施压主线程。
7. **push/step push 校验重复做远端同步与安全性检查**：打开 step push plan、启动 step push、每个 commit 推送前都会重复 fetch/dry-run/安全路径检查，网络慢时体感非常差。

因此整改方向应是：**先让 UI 主线程不再一次性处理/渲染大对象，再减少后端 Git 子进程和重复请求，最后建立可量化的性能预算与回归测试。**

## 2. 关键证据与热点文件

| 区域             | 证据                                                                                                                                                                 | 影响                                                    |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| Diff 渲染        | `src/lib/components/DiffViewer.svelte:78-98` 计算虚拟窗口，但 `:580-731` 仍渲染全部行                                                                                | 大 diff DOM 爆炸，滚动/切换 split/unified 卡顿          |
| Diff worker      | `DiffViewer.svelte:287-305` 创建 worker；`diff-worker.ts:96-110` 返回高亮后的 `parsedDiff`；模板 `DiffViewer.svelte:647`、`:721` 调用未定义 `highlightedLineContent` | worker 优化未闭环，可能构建失败或高亮回退到主线程       |
| Commit 文件列表  | `src/lib/components/ChangedFilesPanel.svelte:50-127` 对全部文件 `#each`                                                                                              | 大提交文件列表会一次性创建大量 DOM                      |
| 首屏加载         | `src/routes/+page.svelte:442-486` 应用缓存后继续 `loadCommitFiles`、刷新仓库状态、再 `loadRepositoryState`                                                           | 启动阶段短时间多次加载和状态替换                        |
| 仓库刷新         | `src/routes/+page.svelte:533-599` 每次概览刷新后替换完整页面状态，并触发详情加载                                                                                     | repo 切换/焦点恢复/刷新时 UI 大面积重绘                 |
| Commit 详情/diff | `src/routes/+page.svelte:821-956` commit 详情和 diff 分两段加载，缓存命中时也会同步推进状态                                                                          | 选择 commit/file 时容易触发同步重渲染                   |
| 焦点刷新         | `src/routes/+page.svelte:1418-1432` app focus 后 500ms 自动刷新当前仓库                                                                                              | 切回应用即卡顿，尤其是大仓库                            |
| 后端 blocking    | `src-tauri/src/commands.rs:52-60` 提供 `spawn_blocking`；概览/详情/diff 走 blocking                                                                                  | Rust 侧大部分 Git 不直接卡 async 线程，但仍会让前端等待 |
| Push 命令入口    | `src-tauri/src/commands.rs:240-283` push 启动命令直接调用 `jobs::*`，preflight 不走 `run_blocking`                                                                   | push 前置校验会阻塞命令响应                             |
| 仓库概览         | `src-tauri/src/git/history.rs:213-300`、`:486-512` 组合分支状态、push 集合、分支列表、历史                                                                           | 一个概览由多次 git 命令串行组成                         |
| 历史过滤         | `src-tauri/src/git/history.rs:125-191`、`:302-400` 过滤时全量扫描                                                                                                    | 大仓库搜索/路径过滤慢                                   |
| Commit 详情      | `src-tauri/src/git/history.rs:423-483` meta + files 至少 4 次 git 调用                                                                                               | 选中 commit 后等待明显                                  |
| Diff 后端        | `src-tauri/src/git/diff.rs:236-265` 每个 diff 先取 parent，再 numstat，再 patch；图片再取对象                                                                        | 文件切换慢，不能渐进显示                                |
| Step push plan   | `src-tauri/src/git/push.rs:286-338` fetch、branch status、安全 hash、dry-run、逐 commit `git show`                                                                   | 打开 plan 弹窗等待长                                    |
| Step push 执行   | `src-tauri/src/jobs.rs:100-180` 启动前校验，线程中每个 hash 又 `push_to_commit`                                                                                      | 每步重复校验/fetch/dry-run，网络慢时明显                |

## 3. 典型卡顿链路

### 3.1 启动 / 切换仓库

当前流程大致为：

1. 前端拿 fast repository index。
2. 尝试读取 cached overview。
3. 如果缓存命中，先把缓存写入 UI。
4. 立刻加载当前 selected commit 的文件详情。
5. 后台刷新所有仓库状态。
6. 同时对当前仓库重新拉完整 overview。
7. overview 返回后再次替换 `branchStatus/localBranches/commits/selectedCommit/...`，并再次加载 commit 详情和 diff。

问题：缓存本来应当让首屏稳定，但当前会马上进入多条刷新链路，导致“看到了内容但又开始转菊花/卡顿”。

### 3.2 选择 commit / 文件

当前流程大致为：

1. 设置 `selectedCommit`。
2. `getCommitDetails` 拉 meta + files。
3. 整体替换 `commitFiles/selectedCommitMeta/selectedFilePath`。
4. 自动拉第一个文件 diff。
5. 设置 `diffResult` 后，`DiffViewer` 解析、生成 splitRows/unifiedRows、渲染 DOM。

问题：后端等待 + 前端解析 + 全量 DOM 渲染串在同一个用户动作后面。大 commit 会先卡文件列表，大 diff 会再卡 diff 区域。

### 3.3 历史过滤

启用任何过滤后：

- metadata 过滤会读取全量 commit record，再在 Rust 内过滤和分页；
- path 过滤会额外读取全量 `git log --name-status -z HEAD`，先算匹配 hash 集合，再过滤全量 commit record。

问题：分页只发生在过滤完成之后，无法做到“先返回第一页”。大仓库上用户每次输入过滤条件都可能触发全仓扫描。

### 3.4 Step push plan / Push

打开 step push plan 时会：

1. `git fetch --prune origin`。
2. 读取分支状态。
3. 计算 safe unpushed hashes。
4. 对目标做 dry-run push。
5. 对 plan 内每个 hash 单独 `git show` 拿摘要。

启动 step push 后又会再次校验，并且每个 commit 推送调用 `push_to_commit`，内部继续做安全校验和远端校验。

问题：网络 I/O 和 Git 子进程重复过多，弹窗 spinner 和 push spinner 都会非常长。

## 4. 整改优先级

### P0：先止住 UI 主线程卡顿

这些改动优先级最高，因为它们直接决定“现代应用体感”。

1. **修正 DiffViewer 虚拟化**
   - 模板只能渲染 `visibleRows`。
   - 使用 `topSpacerHeight/bottomSpacerHeight` 撑开滚动高度。
   - unified/split 共用同一个虚拟行模型，避免两套模板都绕开虚拟化。
   - 滚动事件用 `requestAnimationFrame` 节流，避免每个 scroll event 都触发 Svelte 更新。

2. **修正 diff worker 与高亮消费**
   - 模板使用 worker 返回的 `line.html`，不要调用未定义的 `highlightedLineContent`。
   - `processDiffOnMainThread` 仅作为 worker 创建失败的降级路径，且需要明确不做 highlight 或限制大小。
   - worker 增加 try/catch，返回 error 状态，避免 worker 内异常静默导致 UI 卡死在旧状态。
   - 对 stale request 只忽略结果还不够；需要在 worker 侧记录最新 request id，尽早丢弃明显过期任务。

3. **给 ChangedFilesPanel 做虚拟列表或分批渲染**
   - 大提交只渲染可见文件行。
   - 文件数超过阈值时显示搜索/分页/虚拟列表，不一次性渲染全部。

4. **让 loading 状态先绘制出来**
   - 缓存命中但后续会触发重解析/重渲染时，用 `await tick()` 或 `requestAnimationFrame` 给浏览器一次绘制机会。
   - 重点处理 `loadCommitFiles`、`loadDiff`、切换 diff mode、切换 whitespace 过滤。

5. **临时降低大 diff 阈值**
   - 在虚拟化彻底完成前，把 `MAX_DIFF_LINES` / `MAX_DIFF_BYTES` 下调，避免 5000 行/1MiB patch 直接压垮 DOM。
   - 虚拟化完成后再按实际 profiling 数据调回。

### P1：减少重复请求和重复状态替换

1. **重写启动加载策略**
   - 缓存 overview 命中后，首屏只做必要详情加载。
   - 完整 refresh 放到 idle 队列，或仅在 cache 的 HEAD/upstream 与当前不一致时执行。
   - 避免 `bootstrap()` 中缓存详情加载与随后 `loadRepositoryState()` 再次详情加载同一个 commit。

2. **建立 request de-duplication**
   - 同一个 repoPath + skip + filters 的 overview 请求只允许一个 inflight。
   - 同一个 repoPath + hash 的 commit details 请求复用 promise。
   - 同一个 repoPath + hash + filePath + whitespace 的 diff 请求复用 promise。

3. **缓存增加容量和失效规则**
   - `overviewCache/commitDetailsCache/diffCache` 改为 LRU，并设置容量上限。
   - key 中加入 HEAD hash、branch、upstream 或 repository generation。
   - branch switch、push finished/failed 后刷新、manual remote refresh 后，按 repo generation 清理旧缓存。

4. **限制 focus 自动刷新**
   - focus 刷新加最短间隔，例如 30-60 秒。
   - 如果当前仓库 HEAD 未变，不刷新详情和 diff。
   - 提供用户可见的手动刷新入口，focus refresh 只做轻量 HEAD 检查。

5. **拆分 applyRepositoryState**
   - 不要每次 overview 返回都替换所有子状态。
   - 对比旧值，仅更新变化的 branchStatus/history/selection。
   - 如果 selected commit 未变，不清空 diff，不触发详情重新加载。

### P2：压缩后端 Git 命令数量

1. **优化 repository overview**
   - `branch_status_and_push_sets` 内部减少重复命令：当前分支、upstream、ahead/behind、unpushed、安全路径应尽量批量或复用结果。
   - `safe_unpushed_hashes_in_push_order` 不应对每个 hash 调 `merge-base --is-ancestor`。可先判断一次 `upstream` 是否为 `HEAD` 祖先；behind/diverged 情况已由 ahead/behind 处理。
   - 普通历史分页不一定每次都需要 `rev-list --count HEAD`。可采用 `limit + 1` 判断 `hasMore`，totalCount 异步补齐或缓存。

2. **优化历史过滤**
   - 能交给 Git 的过滤交给 Git：`--author`、`--grep`、pathspec、`--regexp-ignore-case`。
   - 只取 `skip + limit + 1` 所需窗口，不要先取全量再分页。
   - totalCount 对过滤结果可异步计算或显示“超过 N 条”，避免每次输入都全仓统计。
   - 对 body 搜索设置明确开关或阈值，因为 `%B` 全量读取成本很高。

3. **优化 commit details**
   - `get_commit_details` 当前 meta/files 组合仍包含多次 git 调用。考虑用一次 `git show` 或 `git diff-tree --numstat --name-status` 组合解析。
   - 对巨型 commit 的文件列表分页/懒加载：先返回前 N 个文件和总数，滚动或搜索时再加载更多。

4. **优化 diff 后端**
   - parent/base 可在 commit details 中返回，diff 请求不必每次 `git show -s --format=%P`。
   - `numstat` 和 patch 可以合并为一次 git 输出再解析，或先用更便宜的 stat 判断是否超阈值。
   - 对文本 diff 支持取消/超时，用户快速切文件时终止旧 git 进程。

5. **优化 repository status refresh**
   - `refresh_repository_statuses` 当前按路径串行检查；多仓库时可并发限制执行。
   - fast index 先展示，状态刷新分批回传，避免一次返回所有状态才更新 UI。

### P3：Push / Step Push 专项整改

1. **preflight 移入 blocking / 后台 job**
   - `start_push_current_branch/start_push_to_commit/start_step_push` 的前置 Git 校验不要同步阻塞 command 返回。
   - 更好的体验：command 立即返回 job id，job 状态先是 `preparing`，校验失败通过事件返回。

2. **减少 fetch 次数**
   - `sync_origin_tracking` 不应在 plan、validate、每个 push step 中反复执行。
   - 引入 remote tracking freshness：一次手动刷新或 action preflight 刷新后，在同一 job 内复用。

3. **Step push plan 批量读取摘要**
   - plan items 不要每个 hash 单独 `git show`。
   - 使用一次 `git show -s --format=... <hash...>` 或 `git log --format=... --no-walk` 批量返回。

4. **Step push 执行避免每步重复完整校验**
   - 启动 job 前校验 plan hashes 连续性和远端状态。
   - job 内每步只做必要的 branch/head guard，不重复 fetch/dry-run/safe hash 全量计算。
   - 如果需要强安全性，可在每步失败后停止并提示远端状态变化。

## 5. 性能仪表盘与验收标准

先加指标，再改代码。否则无法判断是否真正解决“卡”。

### 5.1 前端指标

建议增加轻量 instrumentation：

- `bootstrap` 总耗时、缓存命中耗时、冷启动耗时。
- `loadRepositoryState`、`loadHistory`、`loadCommitFiles`、`loadDiff` 耗时。
- `DiffViewer` parse/highlight 耗时、worker 往返耗时、渲染行数。
- Long Task 监控：记录超过 50ms 的主线程任务。
- DOM 行数：diff 当前实际渲染 row 数、file list 实际渲染 row 数。

目标预算：

| 交互                      | 目标                                                           |
| ------------------------- | -------------------------------------------------------------- |
| 点击后 loading 可见       | < 100ms                                                        |
| commit 切换（缓存命中）   | < 100ms 无明显掉帧                                             |
| commit 切换（冷加载）     | UI 不阻塞，详情逐步出现                                        |
| diff 切换到 5000 行 patch | 主线程单次任务 < 50ms，DOM 实际渲染行数不超过可视区 + overscan |
| 焦点回到应用              | 若 HEAD 未变，不触发完整刷新                                   |
| 历史过滤输入              | 输入本身不掉帧；请求 debounce 后可取消                         |

### 5.2 后端指标

在 Rust Git command wrapper 加 duration log：

- command name / args category（不要泄漏完整敏感路径到普通日志）。
- repo id/hash。
- elapsed ms。
- output bytes。
- exit status。

重点观察：

- `get_repository_overview` 总耗时和内部 git 命令数。
- `get_commit_details` 内部 git 命令数。
- `get_commit_file_diff` 内部 git 命令数、patch bytes、lineCount。
- `get_step_push_plan` git 命令数、是否执行 fetch。
- push job 从点击到返回 job id 的耗时。

### 5.3 压测场景

至少准备以下 fixture 仓库：

1. 10k+ commits 的长历史仓库。
2. 单个 commit 修改 1000+ 文件。
3. 单个文本文件产生 5000+ 行 diff。
4. 图片/binary 大文件变更。
5. 本地 ahead 100+ commits 的 step push 场景。
6. 配置 50+ repository paths 的仓库列表。
7. 远端网络慢或不可达场景。

## 6. 建议实施顺序

### 第 0 阶段：基线和保护网（0.5-1 天）

- 增加前端 performance marks 和 Rust git duration log。
- 固化 3-5 个性能 fixture。
- 跑一次现状 profiling，记录首屏、切 commit、切 diff、过滤、step push plan 的耗时。
- 修复当前 `DiffViewer` 中未定义的 `highlightedLineContent`，保证类型检查可作为回归门禁。

### 第 1 阶段：前端主线程止血（1-2 天）

- DiffViewer 真正虚拟化，只渲染 `visibleRows`。
- 消费 worker 返回的 HTML，高亮不在主线程做。
- ChangedFilesPanel 虚拟化。
- 大 diff 阈值临时下调。
- 加 `requestAnimationFrame`/`tick` 让 spinner 先绘制。

验收：大 diff 和大文件列表滚动不卡，切换 unified/split 不再创建全量 DOM。

### 第 2 阶段：加载策略与缓存（1-2 天）

- 启动流程去重：缓存命中后只做必要 refresh。
- request de-dupe + LRU cache + generation invalidation。
- focus refresh 改为轻量 HEAD 检查。
- `applyRepositoryState` 最小化状态替换。

验收：启动/切仓库不再连续转多轮 spinner；缓存命中路径稳定。

### 第 3 阶段：后端 Git 命令压缩（2-4 天）

- overview 内部复用 branch/upstream/ahead/behind/unpushed 结果。
- normal history 避免每次 `rev-list --count`，或异步 totalCount。
- filter 尽量下推 Git，并支持取消。
- commit details 和 diff 合并/减少 git 调用。
- repository status refresh 并发限流。

验收：大仓库 overview 冷加载耗时明显下降，Git command 数量可观测减少。

### 第 4 阶段：Push 专项优化（1-3 天）

- push command 立即返回 job id，preflight 放 job 内事件化。
- 一次 job 只 fetch 一次或复用 remote freshness。
- step push plan 批量取摘要。
- step push 执行避免每步重复完整校验。

验收：打开 step push plan 和点击 push 后 UI 立即响应；网络慢时用户能看到 preparing/progress，而不是长时间无响应。

## 7. 回归门禁

每次性能整改至少跑：

- `pnpm typecheck`
- `pnpm test:run`
- `cargo test`（在 `src-tauri`）
- 手动 profiling：大 diff、大文件列表、历史过滤、step push plan

新增测试建议：

- DiffViewer 虚拟化：给 5000 行 diff，断言实际 DOM 行数接近可视区行数，而不是 5000。
- ChangedFilesPanel 虚拟化：给 1000 文件，断言实际 DOM 行数受限。
- page-data request de-dupe：同 key 并发只调用一次 API。
- cache invalidation：branch/head generation 变化后不复用旧 diff/details。
- Rust git command count：对 `get_repository_overview` 用可替换 command runner 或集成测试统计调用次数。

## 8. 风险与注意事项

- 当前工作区已有大量未提交改动，性能整改应小步提交，避免和现有功能改动混在一起。
- Diff 虚拟化会影响复制 hunk、scroll 定位和 split/unified 行高假设，需要补交互测试。
- 后端减少安全校验时必须保持 push 安全边界：不能为了性能跳过必要的远端 fast-forward 和 branch guard。
- 历史过滤下推 Git 后，搜索语义可能变化，尤其是 query 同时匹配 hash、summary、body、author、tag 的现有行为；需要明确产品语义。
- 缓存 key 加 HEAD/upstream 后，缓存命中率可能下降，但正确性优先。

## 9. 一句话整改目标

把 TickGit 从“每次操作等待一串 Git 命令 + 前端一次性渲染大 DOM”改成“后台可取消、请求可复用、结果分阶段展示、主线程只渲染可见内容”的桌面应用体验。
