# TickGit UI 改造计划

## 1. 目标与现状分析

本计划基于当前 TickGit UI 截图与目标设计稿截图整理，供后续 UI 开发按阶段实施。

### 当前 UI 主要问题

- 顶部信息区过重：仓库、分支、语言、推送入口占用同一行，视觉分隔强但缺少整体层次。
- 语言切换放在主界面，打断核心 Git 工作流；当前 select 样式也与整体 UI 不协调。
- 主界面缺少设置入口、刷新入口等全局操作区，导致顶部操作组织不清晰。
- 整体视觉偏“表格/面板拼接”，缺少目标稿中的卡片、阴影、渐变、留白和视觉焦点。
- History 侧栏筛选控件较基础，没有图标、快捷键提示、筛选按钮组和结果计数层级。
- Commit 列表信息密度高但缺少更明确的选中态、时间 badge、状态点和 hover 层次。
- Commit 详情区目前信息分布偏左，hash、复制、日期等元信息没有形成目标稿中的横向信息带。
- Changed Files 面板只展示文件名与状态，缺少每个文件的增删行摘要、文件类型图标和底部折叠操作。
- Diff 区功能可用，但按钮、标题、行号、copy hunk 的视觉样式不够接近目标稿。
- 目标稿底部有 Commit Message 区域；当前 History 模式没有显示提交信息正文区域。

### 目标 UI 方向

- 使用更现代的深色玻璃感界面：深色背景、弱边框、柔和阴影、蓝色高亮、卡片化模块。
- 顶部改成 “仓库卡片 + 分支卡片 + 推送卡片 + 刷新 + 设置” 的工作流工具栏。
- 语言切换移入设置，不再放在主界面。
- 左侧 History 面板承载搜索、筛选、commit 列表、分页和总数。
- 中间 / 右侧形成“Commit 详情 + 文件列表 + Diff + Commit Message”的主工作区。
- 产品范围已收敛为安全推送工作流：不再规划 Changes segment、工作区 stage / unstage 或本地 git commit 能力。
- 保留现有安全推送能力，不在本轮 UI 改造中新增 Git 行为语义。

---

## 2. 信息架构与布局改造

### 2.1 顶部窗口与全局工具栏

目标结构：

```text
macOS traffic lights + centered app title/logo
┌────────────────────────────────────────────────────────────┐
│ Repository Card │ Branch Card │ Push Card │ Refresh │ Settings │
└────────────────────────────────────────────────────────────┘
```

实施要求：

- 保留 Tauri 窗口标题区域，但视觉上模拟目标稿中的顶部暗色标题栏。
- App 标题居中显示，建议加入小型 Git/branch 风格图标和 `TickGit` 文案。
- 顶部工具栏拆成 5 个区域：
  - Current Repository 卡片
  - Current Branch 卡片
  - Push origin 卡片
  - Refresh 按钮
  - Settings 按钮
- 仓库与分支之间、分支与推送之间可保留小型连接符号或箭头，表达工作流顺序。
- 移除主界面的 Language 区域。

### 2.2 Repository Card

目标样式：

- 卡片深色半透明背景，圆角约 `10-12px`，弱边框。
- 左侧放仓库图标，可用蓝色渐变方块 + database/repository icon。
- 主标题显示仓库名。
- 仓库可用时显示绿色 `ACTIVE` badge。
- 副标题显示压缩后的路径，例如 `~/Desktop/Squad/otaServer/ota_admin`。
- 右侧保留下拉箭头，点击仍打开现有 RepositorySwitcher 能力。

功能约束：

- 不改变仓库选择、添加、移除、重新定位逻辑。
- missing / invalid 仓库仍要显示明确状态 badge，可用 `MISSING` / `INVALID`。

### 2.3 Branch Card

目标样式：

- 左侧蓝色渐变 branch icon。
- 主标题显示当前分支，例如 `main`。
- 副标题显示 upstream，例如 `origin/main`。
- 右侧保留下拉箭头，点击仍打开现有 BranchSwitcher。
- 分支不可切换时保留 disabled 态和 tooltip / reason。

### 2.4 Push Card

目标样式：

- 左侧圆形向上箭头图标。
- 主标题：`Push origin` / `推送 origin`。
- 副标题：`Ahead 36 commits` / `领先 36 个 Commit`。
- 右侧显示蓝色 pill：`36 ↑`。
- 可推送时整体可点击；不可推送时使用 muted/disabled 样式，并显示 blocked reason。

功能约束：

- 继续使用现有 `pushCurrentBranch()` 和后台 job 事件。
- behind/diverged、missing upstream、detached 等后端状态仍是禁用依据。
- 推送中状态显示 loading / progress，不要只依赖按钮 disabled。

### 2.5 Refresh 与 Settings

- Refresh 按钮从主内容区域抽到顶部工具栏右侧。
- Settings 按钮新增为顶部右侧独立入口。
- Settings 点击打开设置弹窗或右侧抽屉，第一版建议弹窗，范围更小。

Settings 第一版内容：

- Language / 语言设置：`简体中文`、`English`。
- 使用更美观的选项列表或 segmented control，不再使用原生 select。
- 当前语言用 check icon / active background 表示。
- 可预留后续设置区，但不要实现未定义功能。

---

## 3. 主界面区域改造

### 3.1 左侧 History Panel

目标结构：

```text
Search commits [⌘K]
Filter buttons: Author | Path | Message | More
COMMITS                         152
Commit list
Pagination / Showing 1-20 of 152 commits
```

实施要求：

- 搜索框增加 search icon 和快捷键提示 `⌘K`。
- 将当前 `Author`、`File path` 输入框改造成筛选按钮组：
  - `Author`
  - `Path`
  - `Message`
  - filter icon 按钮
- 第一版可以点击筛选按钮后展开 popover 或 inline filter row；不要把三个输入框常驻铺开。
- 增加 commit 总数显示：后端当前 `CommitHistoryPage` 没有 total count，第一版可显示当前已加载数量；如要精确 `152`，需要新增后端 total 字段。
- Commit 列表底部增加分页区域：
  - `Showing 1-20 of N commits`
  - 页码按钮 `1 2 3 ... 8`
  - 左右翻页按钮
- 如果不新增 total/page API，分页 UI 第一阶段可先保留 “Load more” 行为，但视觉上预留目标结构。

### 3.2 Commit List Item

目标样式：

- 每项高度略增，信息层级更明确。
- 左侧 avatar 叠加绿色向下图标，表示未推送/可推送状态。
- 标题完整显示一行，超长省略。
- 第二行显示 author、相对时间、short hash。
- 右侧显示时间 badge，例如 `49d ago`，并保留小状态点。
- 当前选中项使用蓝色渐变背景和左侧高亮条。
- hover 态使用轻微蓝色/白色透明叠加。

功能约束：

- 保留右键菜单、push to commit、step push 行为。
- 保留当前 safe/unsafe push target 的状态判断。

### 3.3 Commit Detail Header

目标结构：

```text
summary                                                   collapse icon
avatar author <email>       hash pill + copy              date
+ added lines    - removed lines
```

实施要求：

- Summary 字号略大，使用更强白色。
- author 行左侧 avatar 继续使用 initials。
- hash 改成 pill 样式，右侧内嵌 copy icon。
- 日期移动到右侧，与 hash 同一行形成信息带。
- 增删统计用 `+` / `-` 符号和绿色/红色文本。
- 右上角保留折叠 icon，但第一版可以只做视觉，不实现折叠；如实现，需要保持 diff 区可恢复。

### 3.4 Changed Files Panel

目标结构：

```text
Changed Files                        5
selected file row: icon file name   +57 -57 copy
file row: icon file name            +12 -8 copy
...
Collapse All Files
```

实施要求：

- 文件列表改为更卡片化的列表项。
- 左侧显示文件类型图标：
  - JSON / package 可用 `{}` 或 `JQ` 风格 badge。
  - YAML / lockfile 可用 `YAML` badge。
  - 普通文件回退为状态 badge。
- 文件行显示增删行统计，需要后端补充每个文件的 additions/deletions。
- 第一阶段如果不改后端，可只在当前选中文件通过 diff meta 展示，其他文件暂不展示或显示占位；正式实现建议扩展 `CommitFileChange`。
- 复制路径按钮保留，但视觉改为右侧轻量 icon button。
- 底部增加 `Collapse All Files` 按钮；第一版可以先折叠全部文件详情区域，或仅作为禁用/占位不建议。若无明确行为，先不展示。

建议接口扩展：

```ts
export type CommitFileChange = {
  status: string;
  path: string;
  previousPath: string | null;
  displayPath: string;
  additions?: number;
  deletions?: number;
  language?: string | null;
};
```

Rust 侧可通过 `git show --numstat --name-status` 或单独 `numstat` 解析补齐。

### 3.5 Diff Panel

目标样式：

- Diff header 左侧显示文件类型 icon + 文件名。
- 右侧显示 `Unified` 下拉、settings icon、更多下拉。
- `Copy hunk` 按钮改成目标稿中的胶囊按钮，带 copy icon。
- 行号区背景更暗，删除/新增行使用更柔和的红绿背景。
- 当前 hunk header 使用蓝灰色背景。
- 保留现有 Unified / Split、Hide whitespace、binary/image/too large 降级状态。

### 3.6 Commit Message 区域

目标稿底部显示 Commit Message：

- 在 History commit detail 下方展示当前 commit 的 body/message。
- 标题为 `Commit Message`。
- 右侧显示状态 badge：`Safe step-push`、`Behind 0`。
- 内容区使用 monospace 卡片，显示 commit body；如果 body 为空，可显示 summary 或空态。

功能约束：

- History 模式只读，不提供编辑、复制、提交等工作区写操作。

---

## 4. 视觉系统与组件拆分

### 4.1 Design Tokens

状态：已建立 TickGit Design Tokens 基础层，并补充通用视觉 utility；后续 UI 改造应优先复用 token，避免新增散落硬编码基础色值。

建议新增前端样式常量或 Tailwind class 约定：

- 背景：
  - app background: `#090d16` / `#0b1020`
  - panel background: `rgba(15, 23, 42, 0.72)`
  - elevated card: `rgba(30, 41, 59, 0.72)`
- 边框：`rgba(148, 163, 184, 0.12)`
- 主蓝：`#3b82f6` / `#60a5fa`
- 绿色：`#34d399`
- 红色：`#fb7185`
- 文本：
  - primary: `#f8fafc`
  - secondary: `#cbd5e1`
  - muted: `#64748b`
- 圆角：
  - card: `12px`
  - small control: `8px`
  - pill: `999px`
- 阴影：弱蓝色 glow，只用于选中态和顶部卡片 hover。

### 4.2 建议新增 / 调整组件

新增组件：

- `TopToolbar.svelte`
  - 组合 RepositoryCard、BranchCard、PushCard、RefreshButton、SettingsButton。
- `RepositoryCard.svelte`
  - 包装现有 RepositorySwitcher 的触发区域。
- `BranchCard.svelte`
  - 包装现有 BranchSwitcher 的触发区域。
- `PushCard.svelte`
  - 替代当前普通 push button 展示。
- `SettingsDialog.svelte`
  - 放语言切换，后续扩展设置项。
- `LanguageOptionList.svelte`
  - 替代当前主界面 select。
- `HistoryFilters.svelte`
  - 搜索框、快捷键提示、筛选按钮组。
- `CommitMessagePanel.svelte`
  - History 模式只读 commit message 区域。
- `FileTypeIcon.svelte`
  - 根据文件扩展名显示文件类型 badge/icon。

调整组件：

- `LanguageSwitcher.svelte`
  - 不再被 `+page.svelte` 顶部直接使用。
  - 可改造成 Settings 内部的 option list，或删除后由 `LanguageOptionList` 替代。
- `CommitHistoryList.svelte`
  - 调整 item 视觉、选中态、状态点、分页区域。
- `CommitDetailPanel.svelte`
  - 拆分 Header、ChangedFiles、Diff、CommitMessage，降低单文件复杂度。
- `DiffViewer.svelte`
  - 调整 header、hunk、行号、copy hunk 按钮样式。

---

## 5. 分阶段实施计划

### Phase 1：顶部工具栏与设置入口

目标：先移除主界面语言切换，完成目标稿顶部结构。

小任务：

1. 新增 `SettingsDialog.svelte` 和语言选项列表。
2. 从 `+page.svelte` 顶部移除 `LanguageSwitcher`。
3. 新增设置按钮，点击打开 SettingsDialog。
4. 新增 TopToolbar 视觉结构，逐步迁移仓库、分支、推送、刷新入口。
5. 保持现有 RepositorySwitcher / BranchSwitcher 逻辑不变，只替换触发器视觉。

验收：

- 主界面不再出现 Language select。
- 设置弹窗中可以切换 `zh-CN` / `en-US`。
- 语言选择控件不是原生 select，选中态清晰美观。
- 仓库切换、分支切换、刷新、推送功能不退化。

### Phase 2：全局视觉系统与左侧 History 面板

目标：让整体观感接近目标稿的深色卡片化风格。

小任务：

1. 统一 app 背景、panel 背景、边框、圆角、阴影、选中蓝色。
2. 改造 History 面板头部与计数层级。
3. 改造搜索框：增加 icon 和 `⌘K` 提示。
4. 改造 Author / Path / Message 筛选为按钮组或 popover。
5. 改造 Commit list item 的选中态、hover 态、时间 badge、状态点。
6. 增加 commit count / pagination 视觉；如果后端暂不支持 total，先展示已加载范围。

验收：

- 左侧区域视觉与目标稿基本一致。
- 所有现有历史筛选能力可用。
- 右键菜单、选中 commit、load more / pagination 行为不退化。

### Phase 3：Commit 详情与 Changed Files 改造

目标：主内容区信息层级和文件列表接近目标稿。

小任务：

1. 重排 Commit Detail Header。
2. hash 改为 pill + copy icon。
3. 日期移动到右侧。
4. 增删统计改为 `+ added` / `- removed` 风格。
5. Changed Files 面板改成卡片化文件列表。
6. 新增 FileTypeIcon。
7. 评估并实现 `CommitFileChange.additions/deletions/language` 扩展。
8. 补 Rust 解析测试与前端类型测试。

验收：

- Commit 详情首屏层级清楚。
- 文件列表有选中态、文件 icon、复制按钮。
- 如实现 additions/deletions，普通文本 diff、rename、binary/image 文件都要有稳定表现。

### Phase 4：Diff 与 Commit Message 区域

目标：完成目标稿右侧 diff 和底部 commit message。

小任务：

1. 改造 Diff header 的文件图标、标题和右侧 controls。
2. 改造 Copy hunk 按钮样式。
3. 优化 hunk header、行号、增删行背景色。
4. 新增 History 模式只读 `CommitMessagePanel`。
5. 显示 Safe step-push / Behind 等状态 badge。

验收：

- Diff 的 Unified / Split、Hide whitespace、copy hunk 功能不退化。
- binary/image/too large 降级状态仍清晰。
- Commit message 展示不影响文件列表和 diff 的滚动布局。

### Phase 5：响应式、可访问性与收尾

小任务：

1. 检查最小窗口宽度下顶部卡片是否溢出。
2. 检查 History 面板、文件列表、Diff 区滚动边界。
3. 补齐按钮 aria-label、dialog focus trap、Esc 关闭设置弹窗。
4. 检查中英文下文案长度，尤其是顶部卡片、筛选按钮、push card。
5. 清理重复 Tailwind class，必要时抽取小组件。
6. 截图回归对比目标稿。

验收：

- macOS 默认窗口尺寸下布局稳定。
- 中文/英文切换后无明显溢出。
- 键盘可操作 Settings、搜索框、主要按钮。

---

## 6. 接口与数据改造建议

本轮 UI 可以先不改后端，但要完全贴近目标稿，建议后续扩展：

### 6.1 Commit history total count

用途：左侧显示 `COMMITS 152` 和分页。

建议：

```ts
export type CommitHistoryPage = {
  items: CommitListItem[];
  nextSkip: number;
  hasMore: boolean;
  unpushedCount: number;
  safeUnpushedCount: number;
  totalCount?: number;
};
```

第一版可以 optional，不破坏现有前端。

### 6.2 File additions / deletions

用途：Changed Files 每行展示 `+57 -57`。

建议：

```ts
export type CommitFileChange = {
  status: string;
  path: string;
  previousPath: string | null;
  displayPath: string;
  additions?: number;
  deletions?: number;
  language?: string | null;
};
```

### 6.3 Commit body 展示

当前已有 `CommitMeta.body`，可直接用于 History 模式底部 Commit Message。

---

## 7. 测试与验收清单

每个阶段至少跑：

```bash
rtk pnpm test:run
rtk proxy pnpm typecheck
rtk pnpm build
rtk pnpm format:check
```

涉及 Rust 数据结构或解析时追加：

```bash
rtk cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
rtk cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
rtk cargo test --manifest-path src-tauri/Cargo.toml
```

UI 手动验收：

- 英文界面顶部不出现 Language 控件。
- 中文界面顶部不出现语言控件。
- Settings 中语言切换可用，选中态美观。
- 仓库、分支、推送、刷新、设置入口与目标稿位置一致。
- History-only 主界面布局稳定，不再出现 Changes segment。
- Commit 搜索、Author / Path / Message 筛选正常。
- Commit 选择、文件选择、Diff 加载正常。
- Unified / Split、Hide whitespace、Copy hunk 正常。
- Push current、Push to commit、Step push plan preview 正常。
- behind/diverged 等不可推送状态仍有明确提示。

---

## 8. 明确不纳入本次 UI 改造的范围

- 不新增 pull / merge / rebase 能力。
- 不新增 git commit / stage / unstage / workspace Changes 能力。
- 不改变 safe step-push 后端策略。
- 不做 hunk / line staging。
- 不重写 i18n 资源架构，只调整语言切换入口和展示组件。
- 不为目标稿中的占位按钮实现未定义功能；例如 Collapse All Files 如无明确行为，不应强行上线。
