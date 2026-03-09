# skills管理器

`skills管理器` 是一个面向多 Agent 开发者的桌面应用，目标是把本地 skills 的扫描、统一仓库、分发、安全、模板注入、国际化与主题切换整合到同一个控制台中。

## 当前阶段

当前仓库已进入第二阶段功能闭环开发，已完成以下主链路：

| 模块 | 当前已闭环 | 当前仍未闭环 |
|---|---|---|
| 扫描与概览 | 系统级 / 项目级 / 自定义目录扫描、SQLite 快照恢复、任务中心事件总线 | 空间治理统计、模板数量聚合仍未回填首页概览 |
| 市场 | GitHub provider 真实搜索、SQLite 缓存、provider 独立状态展示 | 多 provider 聚合、市场详情增强 |
| 安装与安全 | 下载 / 解包 / 识别 SKILL.md / 安全预扫描 / canonical store / 落库 / 安全报告写入 | 更新、卸载、差异对比 |
| 分发 | 已安装 skill 的真实分发、Windows `symlink` 权限失败显式报错 | 分发向导完善、自定义批量分发 |
| 模板 | 模板 CRUD、模板注入、逐项 `installed/skipped/failed` 结果 | `market_ref` 模板项接通、模板分享/市场化 |

## 运行方式

```bash
pnpm install
pnpm tauri dev
```

## 关键文档

- `tep-docs/PRD.md`：产品需求真源
- `tep-docs/TechDesign.md`：技术设计真源
- `tep-docs/PrototypeDesign.md`：页面与原型设计说明
- `tep-docs/PrototypePromptPack.md`：AI 设计工具提示词包
- `AGENTS.md`：项目协作规范

## 注意事项

- 当前实现以第二阶段主链路为准，README 仅做入口说明，产品/技术真源仍以 `tep-docs/PRD.md` 与 `tep-docs/TechDesign.md` 为准。
- 当前仍未接通更新治理、空间治理统计、模板市场化与 `market_ref` 模板项安装。
- 重要变更必须优先回写文档，不要只改代码不改真源文档。
