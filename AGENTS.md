# AGENTS.md

## 项目概览

`skills-manager` 是一个基于 `Tauri v2 + React 19 + TypeScript + Vite` 的桌面应用，用于管理本地技能仓库、搜索并安装市场技能、执行安全扫描、维护模板，并将技能分发到不同 Agent 目录。

| 维度 | 说明 |
| --- | --- |
| 前端 | `React 19`、`TypeScript`、`React Router`、`Zustand`、`i18next`、`Tailwind CSS v4`、`daisyUI` |
| 后端 | `Rust`、`Tauri v2`、`rusqlite`、`ureq`、`walkdir` |
| 包管理 | 根目录使用 `pnpm@10.0.0`，Rust 依赖由 `cargo` 管理 |
| 桌面壳 | `src-tauri/tauri.conf.json` 配置应用窗口、构建流程与打包目标 |
| 主要能力 | 仓库技能管理、市场搜索/安装、导入、分发、安全扫描、模板注入、设置维护 |

## 目录速览

| 路径 | 作用 |
| --- | --- |
| `src/main.tsx` | 前端入口，挂载应用并初始化样式与 i18n |
| `src/App.tsx` | 启动引导，拉取 bootstrap 数据并应用主题/语言 |
| `src/app/router.tsx` | 顶层路由，当前使用 `createHashRouter` |
| `src/pages/` | 页面级视图：仓库、技能、市场、安全、设置、模板 |
| `src/components/` | 共享组件与弹窗，页面尽量保持薄层 |
| `src/stores/` | Zustand 状态管理，异步保存/同步逻辑集中在 store |
| `src/lib/` | 前端工具与 Tauri IPC 封装，尤其是 `src/lib/tauri-client.ts` |
| `src/types/app.ts` | 前后端共享的数据契约入口，新增字段时优先同步这里 |
| `src/locales/*/common.json` | 多语言文案，当前包含 `zh-CN`、`en-US`、`ja-JP` |
| `src-tauri/src/commands/` | Tauri command 暴露层，面向前端 IPC |
| `src-tauri/src/services/` | 业务逻辑层，放核心流程而不是 UI 适配代码 |
| `src-tauri/src/repositories/` | SQLite 与持久化逻辑 |
| `src-tauri/src/domain/` | 领域状态与类型 |
| `docs/API.md` | Tauri command 总览，修改接口面时要同步更新 |
| `tep-docs/` | 产品/设计/技术参考文档，不是运行时代码 |

## 环境准备

| 项目 | 要求 |
| --- | --- |
| Node.js | 建议使用当前 LTS；本机已验证 `v22.22.1` 可运行 |
| pnpm | 通过 `corepack` 调用，锁定版本为 `pnpm@10.0.0` |
| Rust | `src-tauri/Cargo.toml` 声明 `rust-version = 1.77.2`，当前稳定版可编译通过 |
| Tauri 先决条件 | 需要可用的 WebView2 与 MSVC 工具链；可通过 `corepack pnpm tauri info` 检查 |

首次进入仓库时建议执行：

- `corepack enable`
- `corepack pnpm install`
- `corepack pnpm tauri info`

## 常用命令

以下命令均应从仓库根目录执行；其中 `corepack pnpm lint`、`corepack pnpm typecheck`、`corepack pnpm build`、`corepack pnpm tauri info`、`cargo test --manifest-path src-tauri/Cargo.toml` 已在当前仓库实际验证通过。`corepack pnpm tauri:dev` 与 `corepack pnpm tauri:build` 属于交互式、长时间运行或高开销命令。对于 Codex 等代理，默认都不应主动执行，尤其不要把 `corepack pnpm tauri:build` 当作常规验证步骤；只有在用户明确要求代理执行，且任务确实需要桌面联调或安装包构建时，才可考虑运行。

| 目的 | 命令 | 说明 |
| --- | --- | --- |
| 安装依赖 | `corepack pnpm install` | 安装前端依赖 |
| 启动前端开发服务器 | `corepack pnpm dev` | 仅启动 Vite |
| 启动桌面应用开发模式 | `corepack pnpm tauri:dev` | 会启动 Vite + Tauri 桌面壳；默认由用户自行执行，代理不主动启动 |
| 前端静态检查 | `corepack pnpm lint` | 运行 ESLint |
| TypeScript 类型检查 | `corepack pnpm typecheck` | 分别检查 `tsconfig.app.json` 与 `tsconfig.node.json` |
| 前端生产构建 | `corepack pnpm build` | 执行 `tsc -b && vite build` |
| 检查 Tauri 环境 | `corepack pnpm tauri info` | 检查 WebView2 / Rust / CLI 先决条件 |
| 运行 Rust 测试 | `cargo test --manifest-path src-tauri/Cargo.toml` | 当前后端主要自动化测试入口 |
| 构建桌面安装包 | `corepack pnpm tauri:build` | 生成桌面应用包，耗时较长，且会显著增大 `src-tauri/target/` 占用；默认由用户自行执行，代理不主动运行 |

## 开发工作流

| 场景 | 建议做法 |
| --- | --- |
| 改页面或交互 | 优先从 `src/pages/`、`src/components/`、`src/stores/` 入手，避免把业务逻辑塞进组件 |
| 改 IPC 接口 | 同步更新 `src-tauri/src/commands/app.rs`、`src/lib/tauri-client.ts`、`src/types/app.ts`、`docs/API.md` |
| 改业务流程 | 优先修改 `src-tauri/src/services/`，仓储层 `repositories/` 只负责持久化 |
| 改数据库结构 | 修改 `src-tauri/src/repositories/db.rs` 的迁移逻辑，并补充/更新对应 Rust 测试 |
| 新增页面 | 同步更新 `src/app/router.tsx`、导航壳 `src/components/layout/AppShell.tsx` 与多语言文案 |
| 新增可见文案 | 同步维护 `src/locales/zh-CN/common.json`、`src/locales/en-US/common.json`、`src/locales/ja-JP/common.json` |
| 本地运行与验证 | 代理默认使用 `corepack pnpm lint`、`corepack pnpm typecheck`、`corepack pnpm build`、`cargo test --manifest-path src-tauri/Cargo.toml` 等非交互命令完成验证；不要把 `corepack pnpm tauri:build` 作为常规验证步骤 |

## 代码约定

### 前端

- 使用函数组件与 Hooks；共享状态优先进入 `Zustand store`，不要在多个页面重复维护同一份远程状态。
- IPC 调用统一经由 `src/lib/tauri-client.ts`，不要在组件中直接散落 `invoke(...)`。
- 类型定义优先收敛到 `src/types/app.ts`；前后端契约变更时先改类型再改实现。
- 保持页面组件薄、弹窗组件专注、工具函数纯净；已有结构已经按页面 / 组件 / store / lib 分层。
- 路由是 Hash 模式；除非明确需要，不要擅自切换为 Browser Router。

### 后端

- `commands` 负责边界适配，`services` 负责业务逻辑，`repositories` 负责 SQLite 读写；不要跨层混写。
- 新增命令时优先沿用现有错误传播方式，返回明确错误，不要吞错。
- 本项目对失败可见性要求较高：不要引入“静默降级”“假成功”“copy 代替 symlink 但不告知”等隐藏 fallback。
- 与文件系统、分发、模板注入相关的逻辑已有“不静默回退”的测试约束，改动时请保留这一行为特征。

## 测试与验证

| 类型 | 命令 | 何时必须运行 |
| --- | --- | --- |
| 前端 lint | `corepack pnpm lint` | 改动 `src/` 下 TS/TSX 文件后 |
| 前端类型检查 | `corepack pnpm typecheck` | 改动 TypeScript 类型、store、页面、IPC 封装后 |
| 前端构建 | `corepack pnpm build` | 改动打包、资源、入口、样式或发布前 |
| Rust 单元/集成测试 | `cargo test --manifest-path src-tauri/Cargo.toml` | 改动 `src-tauri/` 任意 Rust 文件后 |

当前仓库**没有独立的前端测试框架目录**；前端回归主要依赖 `lint + typecheck + build`，后端回归主要依赖 `cargo test`。

## 构建产物与非源码目录

- 不要手工编辑 `dist/`、`src-tauri/target/`、`src-tauri/target-codex*/`、`target/`、`target-codex*/`。
- `.agents/`、`skills-lock.json`、`tep-docs/` 在 `.gitignore` 中被视为生成物或参考资料；除非任务明确要求，否则不要把它们当作主实现入口。
- `src-tauri/gen/` 为生成目录，通常不应手改。

## 提交前检查

提交或宣称完成前，至少按改动范围运行对应命令：

| 改动范围 | 最低检查集 |
| --- | --- |
| 仅前端 | `corepack pnpm lint` + `corepack pnpm typecheck` + `corepack pnpm build` |
| 仅 Rust/Tauri | `cargo test --manifest-path src-tauri/Cargo.toml` |
| 前后端接口联动 | 上述两组全部执行 |

如果修改了 Tauri command、共享类型、数据库 schema、模板/分发/安全扫描逻辑，默认视为“前后端接口联动”。

## 额外提示

- `docs/API.md` 是当前 command 面的速查表；任何 IPC 面变更都应一起更新，避免文档与实现漂移。
- `eslint.config.js` 已忽略 `dist/`、`src-tauri/target/**`、`src-tauri/target-codex*/**`、`tep-docs/**`；排查 lint 结果时注意范围。
- `vite.config.ts` 已为 Tauri 开发模式配置端口、HMR 与 `src-tauri` 目录监听忽略规则，修改 dev server 行为时要兼顾桌面壳联调；实际桌面联调默认由用户自行执行 `corepack pnpm tauri:dev`。
- `corepack pnpm tauri:build` 会生成较大的 Rust/Tauri 构建产物，并持续增加 `src-tauri/target/` 目录体积；除非用户明确要求且确实需要安装包或构建产物，否则代理不应主动运行该命令。
