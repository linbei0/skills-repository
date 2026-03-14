<div align="center">
  <img src="./src-tauri/icons/icon.png" alt="skills-manager logo" width="96" height="96" />

# skills-manager

[中文](./README.md) | [English](./README.en.md)

一个用于统一管理 Agent skills 资产的桌面应用。它将仓库管理、市场安装、安全扫描、模板注入与多目标分发整合到同一个 `Tauri + React` 工作台中。

[功能概览](#功能概览) • [快速开始](#快速开始) • [开发命令](#开发命令) • [项目结构](#项目结构) • [技术栈](#技术栈)

<img src="./docs/pics/home.png" alt="skills-manager 应用首页截图" width="960" />
</div>

## 功能概览

- **统一 skills 仓库**：集中管理已导入、已安装的 skills，并查看详情、来源与删除预览。
- **多来源导入**：支持从 `GitHub`、本地目录、`ZIP` 导入 skill 到统一仓库。
- **市场搜索与安装**：搜索外部 skill 源，并在安装前执行安全检查。
- **安全报告中心**：查看风险等级、问题明细、分类统计，并可重新扫描。
- **模板化复用**：把多个仓库 skill 组合成模板，一次性注入到目标项目。
- **多目标分发**：按 Agent 标签目录或自定义相对路径，将 skills 以 `symlink` / `copy` 方式分发。
- **设置与本地化**：支持主题、语言、可见目标管理与仓库存储位置迁移。

> [!TIP]
> 如果你在维护多个 Agent 的技能目录，这个项目最适合用来做“统一仓库 + 分发控制台”。

## 界面模块

| 模块 | 说明 |
| --- | --- |
| `Repository` | 浏览统一仓库中的 skills，查看详情、导入、卸载、分发 |
| `Skills` | 当前技能视图与分发工作流入口 |
| `Market` | 搜索外部 skills provider 并安装 |
| `Security` | 查看安全扫描报告并触发重扫 |
| `Templates` | 创建 skills 模板并注入到项目 |
| `Settings` | 管理语言、主题、目标目录与仓库存储路径 |

## 快速开始

### 环境要求

| 依赖 | 说明 |
| --- | --- |
| `Node.js` | 建议使用当前 LTS，仓库已验证 `Node 22` 可运行 |
| `pnpm` | 通过 `corepack` 使用，项目锁定为 `pnpm@10.0.0` |
| `Rust` | 需要可用的 stable toolchain |
| `Tauri` 先决条件 | Windows 需要 WebView2 与 MSVC 工具链；可用 `tauri info` 检查 |

### 安装

```bash
corepack enable
corepack pnpm install
corepack pnpm tauri info
```

### 本地开发

仅启动前端：

```bash
corepack pnpm dev
```

启动桌面应用开发模式：

```bash
corepack pnpm tauri:dev
```

> [!NOTE]
> `tauri:dev` 会同时拉起 Vite 与 Tauri 桌面壳，适合联调前后端 IPC。

## 开发命令

| 目的 | 命令 |
| --- | --- |
| 安装依赖 | `corepack pnpm install` |
| 前端开发 | `corepack pnpm dev` |
| 桌面开发 | `corepack pnpm tauri:dev` |
| 代码检查 | `corepack pnpm lint` |
| 类型检查 | `corepack pnpm typecheck` |
| 前端构建 | `corepack pnpm build` |
| 桌面打包 | `corepack pnpm tauri:build` |
| 检查 Tauri 环境 | `corepack pnpm tauri info` |
| Rust 测试 | `cargo test --manifest-path src-tauri/Cargo.toml` |

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 桌面壳 | `Tauri v2` |
| 前端 | `React 19`、`TypeScript`、`Vite`、`React Router` |
| 状态管理 | `Zustand` |
| 样式 | `Tailwind CSS v4`、`daisyUI` |
| 国际化 | `i18next`、`react-i18next` |
| 后端 | `Rust`、`rusqlite`、`ureq`、`walkdir` |

## 项目结构

```text
.
├─ src/                     # React 前端
│  ├─ app/                  # 路由
│  ├─ components/           # 组件与弹窗
│  ├─ pages/                # 页面级视图
│  ├─ stores/               # Zustand 状态管理
│  ├─ lib/                  # 工具函数与 Tauri IPC 封装
│  ├─ locales/              # i18n 语言包
│  └─ types/                # 前后端共享类型
├─ src-tauri/               # Tauri / Rust 后端
│  ├─ src/commands/         # Tauri command 边界层
│  ├─ src/services/         # 核心业务逻辑
│  ├─ src/repositories/     # SQLite 与持久化
│  ├─ src/domain/           # 领域状态与类型
│  └─ tauri.conf.json       # 桌面壳配置
├─ docs/API.md              # Tauri command 速查
└─ assets/                  # 设计草稿与资源
```

## 架构说明

前端通过 `src/lib/tauri-client.ts` 统一调用 Tauri IPC，后端在 `src-tauri/src/commands/app.rs` 注册命令，并将实际业务下沉到 `services/` 与 `repositories/`。

典型工作流如下：

1. 用户在前端页面触发操作。
2. `tauri-client` 调用对应 command。
3. Rust `commands` 做输入/输出边界处理。
4. `services` 执行业务逻辑，必要时调用 `repositories` 持久化。
5. 前端 store 更新状态并刷新界面。

如果你要扩展 IPC，请同步关注这些文件：

- `src/lib/tauri-client.ts`
- `src/types/app.ts`
- `src-tauri/src/commands/app.rs`
- `src-tauri/src/lib.rs`
- `docs/API.md`

## 质量检查

当前仓库推荐的最小验证集：

```bash
corepack pnpm lint
corepack pnpm typecheck
corepack pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
```

> [!IMPORTANT]
> 与技能导入、模板注入、分发、仓库迁移、安全扫描相关的变更，应至少运行一次 `cargo test --manifest-path src-tauri/Cargo.toml`。

## 设计与实现说明

- 路由当前使用 `Hash Router`，适合桌面应用分发场景。
- 国际化已内置 `zh-CN`、`en-US`、`ja-JP`。
- 安全相关流程强调**显式失败**，避免静默 fallback 或伪成功路径。
- 数据契约集中在 `src/types/app.ts`，是前后端协作的首要入口。

## 参考文档

- 参考项目：<https://github.com/buzhangsan/skills-manager-client>
- API 速查：`docs/API.md`
- Tauri 官方文档：<https://v2.tauri.app/>
- React 官方文档：<https://react.dev/>
