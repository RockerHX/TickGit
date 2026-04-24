# TickGit AI 开发文档

## 1. 定位与使用时机

本文档定义 TickGit 的本地开发约束、改动放置规则和默认验证方式。

使用原则：

- 小任务：默认以仓库根 `AGENTS.md` 为主
- 重构、架构调整、接口变更、长期规范变更：同时参考本文档和 `docs/ARCHITECTURE.md`

如果实现与正式文档冲突，必须明确选择：

- 改实现以匹配文档，或
- 改文档以反映新的正式决策

不允许长期漂移。

---

## 2. 不可违反的规则

### 2.1 工具与边界

- 包管理器固定为 `pnpm`
- Git 逻辑只能在 Rust 侧实现
- Rust 通过系统原生 `git` 执行命令
- 前端不能直接执行 Git
- 默认不引入 `libgit2`
- Rust 侧处理文件系统路径时，优先使用 `std::path::Path` / `PathBuf`，不要用字符串拼接路径
- 前后端交换仓库等文件系统路径时，优先保持系统原生绝对路径
- Git 仓库内部文件路径如果需要跨平台统一表示，优先使用 `/` 作为逻辑分隔符

### 2.2 前后端通信

- command 统一放在 `src/lib/tauri/api.ts`
- event 统一放在 `src/lib/tauri/events.ts`
- 不要在页面或组件中散写 `invoke("...")`

### 2.3 接口变更

字段、参数、payload 或返回结构变化时，至少同步检查：

- `src-tauri/src/models.rs`
- `src-tauri/src/commands.rs`
- `src/lib/types.ts`
- `src/lib/tauri/api.ts`
- `src/lib/tauri/events.ts`

### 2.4 控制范围

只做当前任务需要的最小修改，不借题发挥：

- 不顺手重写 UI
- 不顺手替换状态管理
- 不顺手重做 Git 执行模型

---

## 3. 改动放置规则

- UI 组件：`src/lib/components/*`
- 页面编排：`src/routes/+page.svelte`
- 页面逻辑辅助与纯函数：`src/lib/tickgit/*`
- Tauri command / event 封装：`src/lib/tauri/*`
- Git 逻辑：`src-tauri/src/git.rs`
- 后台任务：`src-tauri/src/jobs.rs`
- 仓库持久化：`src-tauri/src/repo_store.rs`
- DTO / 公共模型：`src-tauri/src/models.rs` 与 `src/lib/types.ts`

---

## 4. 默认开发流程

1. 先读相关代码，再改
2. 先判断改动属于哪一层
3. 优先小步修改
4. 按改动范围做验证
5. 只在长期规则变化时更新文档

---

## 5. 默认验证

代码改动后，默认按需执行：

```bash
pnpm test:run
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
pnpm typecheck
pnpm build
pnpm format:check
```

如果任务非常局部，可以减少验证，但需要在总结中说明原因。

文档改动只需做内容自检和 diff 检查。

当前默认要求：

- 前端纯 TypeScript 逻辑优先补 `Vitest` 单元测试
- Rust 核心逻辑优先补 `cargo test` 单元测试
- 不要求为当前项目默认加入 Svelte 组件测试，除非任务明确需要

---

## 6. 文档更新规则

以下情况应更新至少一份正式文档：

- 架构边界变了
- 核心约束变了
- Git 执行策略变了
- 持久化方式变了
- 后台任务模型变了
- 对后续开发方式有长期影响

以下情况通常不更新文档：

- 单个 Bug 修复
- 局部样式调整
- 小型性能优化
- 不影响长期边界的局部重构

---

## 7. 交付要求

完成任务后应明确说明：

- 改了什么
- 验证了什么
- 哪些内容未验证
- 是否更新文档，以及为什么
