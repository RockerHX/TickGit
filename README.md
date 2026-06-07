# TickGit

特别感谢：[https://linux.do/](https://linux.do/)

TickGit 是一款基于 **Tauri + Svelte + TypeScript + Rust** 的 Git 增强型桌面工具，目标是提供接近 GitHub Desktop 的桌面体验，同时增加对“未推送 Commit 精细控制”的能力，尤其是 **按 Commit 顺序逐条推送** 的场景。

当前项目已完成首批骨架与核心闭环：

- 拖拽添加本地 Git 仓库
- 仓库列表持久化与切换
- 仓库搜索、从列表移除、失效路径提示和重新定位
- 当前仓库本地分支切换与整页刷新
- 左侧 Commit 历史分页加载
- 已推送 / 未推送 Commit 区分
- 历史默认展示完整 Commit 历史，并标记可安全分步推送的 Commit
- Commit 详情、文件列表、结构化文本 Diff 查看（Unified / Split）
- 常规 `git push`
- 右键“推送到这个 Commit”
- 右键“分步推送到 Commit”与实时进度事件

## 文档与规范

以下文档是后续开发的基础规范。**默认以文档为准实现；只有在架构、约束或流程发生变化时才更新文档。**

- [架构文档](./docs/ARCHITECTURE.md)
- [AI 开发文档](./docs/AI_DEVELOPMENT.md)
- [优化与后续开发计划](./docs/OPTIMIZATION_AND_DEVELOPMENT_PLAN.md)

## 技术栈

- 前端：SvelteKit + TypeScript + TailwindCSS 4
- 前端适配：使用 `@sveltejs/adapter-static`，产物以纯静态资源方式提供，不依赖 Node.js SSR 运行时
- 桌面壳：Tauri 2
- 后端：Rust
- Git 执行方式：Rust 使用 `std::process::Command` 调用系统原生 `git`
- 包管理器：pnpm

## 当前范围

当前版本聚焦于以下范围：

- 仅支持 **当前检出分支**
- 远端固定为 **`origin`**
- 未推送 Commit 的判断基于当前分支的 **upstream**
- 顶部 Push 计数显示的是相对 upstream 的 **全量未推送 Commit 数**
- “分步推送 / Push to Commit” 的可操作目标只来自 **first-parent 安全路径**
- 分步推送任务为 **单任务、不可取消**
- 当前不内置 `git pull` / `merge` / `rebase`，远端已有更新时只禁用推送并提示用户使用 GitHub Desktop 或 SourceTree 同步
- 当前不内置 `git commit` / `stage` / `unstage`，不提供工作区 Changes 视图；需要整理工作区或创建提交时，请使用外部 Git 工具
- 从 TickGit 移除仓库只删除列表记录，不删除本地仓库文件
- 仓库路径失效时会保留列表项，并提供重新定位或移除入口
- Diff 当前为 **结构化文本展示**，支持 **Unified / Split** 与 **Hide Whitespace Changes**，对图片、二进制和超大 Diff 做保护性降级，暂不做语法高亮和图片专用 Diff

## 关于拉取和同步

TickGit 的核心目标是让“当前分支的安全推送”和“按 Commit 分步推送”更简单可控，不替代完整 Git 客户端。

当远端分支已经有新提交，或本地与远端出现分叉时，TickGit 会禁用推送，避免产生不安全的 non-fast-forward 操作。此时请先使用 GitHub Desktop、SourceTree 或命令行完成拉取、合并、变基和冲突处理，再回到 TickGit 刷新状态后继续推送。

如果需要暂存文件、取消暂存或创建本地 Git commit，也请使用外部 Git 工具；TickGit 只负责读取历史与执行安全推送。

## 分步推送如何理解

### 用户感知

当你在未推送的 Commit 里选中某一个目标后，TickGit 会自动找到：

- 为了把这个目标安全推上远端
- 必须先推的那些 Commit

然后按照 **从旧到新** 的顺序，一个一个推送。

你可以把它理解成：

> 我选中一个未推送 Commit，点击“分步推送”，系统就会把到这个目标为止该推的 Commit，按顺序逐个推上去。

### 修复后的保证

即使你之前做过：

- 从 upstream 拉代码
- merge 到当前分支
- 导致本地历史里出现穿插、分叉、merge 提交

TickGit 也会自动避开那些不能直接线性推送的危险路径，保证分步推送仍然表现为：

- 选中一个目标 Commit
- 系统自动判断应该先推哪些
- 按 **从旧到新** 的顺序逐步推送

而不会再因为 merge 后的穿插历史，莫名出现 `non-fast-forward` 这类错误。

同时，历史列表会默认展示完整历史；如果某个未推送 Commit 不在这条 first-parent 安全路径上，TickGit 会把它标记为 **不可安全分步推送**，避免用户逐个尝试。

## 快速开始

### 前置要求

- Node.js 18+
- pnpm
- Rust 工具链
- 系统已安装 `git`
- 本机已满足 Tauri 桌面开发依赖

### 安装依赖

```bash
pnpm install
```

### 启动开发环境

```bash
pnpm tauri dev
```

开发模式启动后，可在终端直接按 `q` 退出 Tauri dev 进程。

### 常用命令

```bash
pnpm test:run
pnpm check
pnpm typecheck
pnpm build
pnpm format
pnpm format:check
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

`pnpm install` 会自动注册仓库内的 pre-commit hook。提交前 hook 会格式化已暂存文件并运行 `pnpm check`，避免格式问题推送到 CI 后才失败。

## 项目结构

```text
.
├── src/                  # 前端 Svelte 代码
│   ├── lib/
│   │   ├── components/   # UI 组件
│   │   ├── tauri/        # invoke / event 封装
│   │   ├── tickgit/      # 页面逻辑辅助与纯函数测试
│   │   ├── types.ts      # 前后端共享 DTO 对应前端类型
│   │   └── utils.ts      # 前端展示工具函数
│   └── routes/+page.svelte
├── src-tauri/            # Rust / Tauri 后端
│   └── src/
│       ├── commands.rs   # Tauri command 入口
│       ├── git/          # Git 命令执行、解析、历史、Diff 与安全推送能力
│       ├── jobs.rs       # 分步推送后台任务与事件
│       ├── repo_store.rs # 仓库持久化
│       ├── models.rs     # DTO / 数据模型
│       └── error.rs      # 统一错误类型
└── docs/                 # 项目规范文档
```

## 开发原则

- 所有 Git 核心能力统一在 Rust 侧实现
- 前端只通过 Tauri command / event 与后端通信
- 不引入复杂 Git C 库或 `libgit2` 方案
- 修改接口时必须同步更新前后端类型
- 默认需要同时维护前端纯逻辑单元测试与 Rust 单元测试
- 只有当规范本身发生变化时才更新文档

## 后续开发要求

后续无论是人工开发还是 AI 开发，都应先阅读：

1. `README.md`
2. `docs/ARCHITECTURE.md`
3. `docs/AI_DEVELOPMENT.md`

如果实现与文档不一致，默认需要先修正文档或先修正实现，不允许长期漂移。
