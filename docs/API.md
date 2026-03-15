# skills-manager API Notes

本文件汇总当前桌面应用对前端开放的 Tauri command，
便于在开发与联调时快速查阅。

## Frontend Entry

- IPC wrapper: `src/lib/tauri-client.ts`
- Command registration: `src-tauri/src/lib.rs`
- Command implementation: `src-tauri/src/commands/app.rs`
- `bootstrap_app` / `save_settings` 的 `AppSettings` 负载包含 `visibleSkillsTargetsVersion`，用于一次性迁移新增的内置 skills 标签可见性。

## Command Surface

| Command | Purpose |
| --- | --- |
| `bootstrap_app` | 读取应用版本、系统信息、设置与仓库存储信息 |
| `save_settings` | 保存语言、主题、目标路径等设置 |
| `migrate_repository_storage` | 迁移统一仓库存储目录 |
| `open_source_reference` | 用系统默认应用打开源码引用 |
| `search_market_skills` | 搜索 market skills 并返回 provider 状态 |
| `install_skill` | 从市场结果安装 skill 并执行安全预扫描 |
| `resolve_repository_import_source` | 解析 GitHub、本地目录或 ZIP 导入候选 |
| `import_repository_skill` | 从指定候选导入 skill 到统一仓库 |
| `list_repository_skills` | 列出统一仓库中的已安装技能 |
| `get_repository_skill_detail` | 获取技能详情与 `skillMarkdown` |
| `get_repository_skill_deletion_preview` | 查看卸载前将删除的目录与分发路径 |
| `uninstall_repository_skill` | 删除统一仓库 skill 与相关记录 |
| `scan_agent_global_skills` | 扫描指定 Agent 的全局 skills 目录 |
| `distribute_skill` | 将单个 skill 分发到目标路径 |
| `batch_distribute_repository_skills` | 批量分发统一仓库中的多个 skills |
| `get_security_reports` | 读取安全扫描报告 |
| `rescan_security` | 重新执行安全扫描并刷新报告 |
| `list_templates` | 列出模板记录 |
| `get_template` | 读取模板详情 |
| `save_template` | 创建或更新模板 |
| `delete_template` | 删除模板 |
| `inject_template` | 将模板中的 skills 批量注入项目目录 |

## Example

```ts
import { listRepositorySkills } from '../src/lib/tauri-client'

async function loadSkills() {
  const skills = await listRepositorySkills()
  return skills.map((item) => ({
    id: item.id,
    name: item.name,
    blocked: item.blocked,
  }))
}
```

## Reference

- Tauri command docs: <https://v2.tauri.app/develop/calling-rust/>

## Repository Update IPC

- `update_repository_skill(skillId)`
  - input: `skillId: string`
  - output: `RepositorySkillUpdateItemResult`
  - behavior: updates one repository skill that can resolve to a public GitHub repo URL; preserves repository identity and canonical path
- `update_github_repository_skills()`
  - input: none
  - output: `BatchRepositorySkillUpdateResult`
  - behavior: updates every repository skill whose persisted metadata can resolve to a public GitHub repo URL
- `RepositorySkillSummary` / `RepositorySkillDetail`
  - new field: `canUpdate: boolean`
- `RepositorySkillUpdateItemResult`
  - fields: `skillId`, `skillName`, `status`, `reasonCode`, `details`, `previousVersion`, `currentVersion`, `copyDistributionCount`
  - `reasonCode` values currently include `updated_to_latest`, `already_up_to_date`, `blocked_by_security_scan`, `update_failed`
  - `details` is structured metadata for UI composition, for example `{ "error": "..." }` on generic failures
- `BatchRepositorySkillUpdateResult`
  - fields: `updated`, `skipped`, `failed`
