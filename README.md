# TickGit

TickGit 是一款基于 **Tauri + Svelte + TypeScript + Rust** 的 Git 增强型桌面工具，目标是提供接近 GitHub Desktop 的桌面体验，同时增加对“未推送 Commit 精细控制”的能力，尤其是 **按 Commit 顺序逐条推送** 的场景。

当前项目已完成首批骨架与核心闭环：

- 拖拽添加本地 Git 仓库
- 仓库列表持久化与切换
- 左侧 Commit 历史分页加载
- 已推送 / 未推送 Commit 区分
- Commit 详情、文件列表、纯文本 Diff 查看
- 常规 `git push`
- 右键“提交到当前 Commit”
- 右键“分步提交 Commit”与实时进度事件

## 文档与规范

以下文档是后续开发的基础规范。**默认以文档为准实现；只有在架构、约束或流程发生变化时才更新文档。**

- [架构文档](./docs/ARCHITECTURE.md)
- [AI 开发文档](./docs/AI_DEVELOPMENT.md)

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
- 分步提交任务为 **单任务、不可取消**
- Diff 当前为 **纯文本展示**，暂不做语法高亮与超大文件优化

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

### 常用命令

```bash
pnpm test:run
pnpm typecheck
pnpm build
pnpm format
pnpm format:check
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

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
│       ├── git.rs        # Git 命令执行与解析
│       ├── jobs.rs       # 分步提交后台任务与事件
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
