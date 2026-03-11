use anyhow::Result;
use rusqlite::params;
use std::path::Path;

use crate::domain::types::{SecurityIssue, SecurityRecommendation, SecurityReport};

use super::db::open_connection;

pub fn save_security_report(path: &Path, report: &SecurityReport) -> Result<()> {
    let mut conn = open_connection(path)?;
    let tx = conn.transaction()?;

    if let Some(skill_id) = report.skill_id.as_deref() {
        tx.execute(
            "DELETE FROM security_reports WHERE skill_id = ?1",
            params![skill_id],
        )?;
    }

    tx.execute(
        "
        INSERT INTO security_reports (
            id,
            skill_id,
            scan_scope,
            level,
            score,
            blocked,
            issues_json,
            recommendations_json,
            scanned_files_json,
            engine_version,
            scanned_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ",
        params![
            report.id,
            report.skill_id,
            report.scan_scope,
            report.level,
            report.score,
            report.blocked as i64,
            serde_json::to_string(&report.issues)?,
            serde_json::to_string(&report.recommendations)?,
            serde_json::to_string(&report.scanned_files)?,
            report.engine_version,
            report.scanned_at,
        ],
    )?;

    tx.commit()?;
    Ok(())
}

pub fn list_security_reports(path: &Path) -> Result<Vec<SecurityReport>> {
    let conn = open_connection(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT
            sr.id,
            sr.skill_id,
            s.name,
            COALESCE(s.canonical_path, (
                SELECT target_path
                FROM skill_distributions
                WHERE skill_id = s.id
                ORDER BY created_at ASC
                LIMIT 1
            )),
            sr.scan_scope,
            sr.level,
            sr.score,
            sr.blocked,
            sr.issues_json,
            sr.recommendations_json,
            sr.scanned_files_json,
            sr.engine_version,
            sr.scanned_at
        FROM security_reports sr
        LEFT JOIN skills s ON s.id = sr.skill_id
        WHERE sr.skill_id IS NOT NULL
        ORDER BY sr.scanned_at DESC
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        let issues_json: String = row.get(8)?;
        let recommendations_json: String = row.get(9)?;
        let scanned_files_json: String = row.get(10)?;

        Ok(SecurityReport {
            id: row.get(0)?,
            skill_id: row.get(1)?,
            skill_name: row.get(2)?,
            source_path: row.get(3)?,
            scan_scope: row.get(4)?,
            level: row.get(5)?,
            score: row.get(6)?,
            blocked: row.get::<_, i64>(7)? != 0,
            issues: serde_json::from_str::<Vec<SecurityIssue>>(&issues_json).unwrap_or_default(),
            recommendations: serde_json::from_str::<Vec<SecurityRecommendation>>(
                &recommendations_json,
            )
            .unwrap_or_default(),
            scanned_files: serde_json::from_str::<Vec<String>>(&scanned_files_json)
                .unwrap_or_default(),
            engine_version: row.get(11)?,
            scanned_at: row.get(12)?,
        })
    })?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::InstallSkillRequest;
    use crate::repositories::db::run_migrations;
    use crate::repositories::skills as skills_repository;
    use tempfile::tempdir;

    fn install_request(slug: &str, name: &str) -> InstallSkillRequest {
        InstallSkillRequest {
            provider: "github".into(),
            market_skill_id: format!("{slug}-market"),
            source_type: "github".into(),
            source_url: format!("https://github.com/demo/{slug}"),
            repo_url: Some(format!("https://github.com/demo/{slug}")),
            download_url: Some(format!("https://github.com/demo/{slug}/archive/main.zip")),
            package_ref: Some(format!("demo/{slug}")),
            manifest_path: None,
            skill_root: None,
            name: name.into(),
            slug: slug.into(),
            version: Some("main".into()),
            author: Some("tester".into()),
            requested_targets: Vec::new(),
        }
    }

    #[test]
    fn saves_and_lists_security_reports() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("security.db");
        run_migrations(&db_path).unwrap();

        let skill_id = skills_repository::save_installed_skill(
            &db_path,
            &install_request("demo", "Demo Skill"),
            "E:/skills/demo",
            "medium",
            false,
        )
        .unwrap();

        let report = SecurityReport {
            id: "report-1".into(),
            skill_id: Some(skill_id),
            skill_name: None,
            source_path: Some("E:/tmp/demo".into()),
            scan_scope: "temp_install".into(),
            level: "medium".into(),
            score: 40,
            blocked: false,
            issues: vec![SecurityIssue {
                rule_id: "network_fetch".into(),
                severity: "medium".into(),
                title: "Review required".into(),
                description: "demo".into(),
                file_path: Some("E:/tmp/demo/install.sh".into()),
            }],
            recommendations: vec![SecurityRecommendation {
                action: "review_files".into(),
                description: "review".into(),
            }],
            scanned_files: vec!["E:/tmp/demo/install.sh".into()],
            engine_version: "phase2-rules-v1".into(),
            scanned_at: 100,
        };

        save_security_report(&db_path, &report).unwrap();
        let loaded = list_security_reports(&db_path).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].level, "medium");
        assert_eq!(loaded[0].issues.len(), 1);
    }

    #[test]
    fn lists_only_latest_persisted_report_per_skill() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("security.db");
        run_migrations(&db_path).unwrap();

        let alpha_id = skills_repository::save_installed_skill(
            &db_path,
            &install_request("alpha", "Alpha"),
            "E:/skills/alpha",
            "safe",
            false,
        )
        .unwrap();
        let beta_id = skills_repository::save_installed_skill(
            &db_path,
            &install_request("beta", "Beta"),
            "E:/skills/beta",
            "safe",
            false,
        )
        .unwrap();

        save_security_report(
            &db_path,
            &SecurityReport {
                id: "alpha-old".into(),
                skill_id: Some(alpha_id.clone()),
                skill_name: Some("Alpha".into()),
                source_path: Some("E:/skills/alpha".into()),
                scan_scope: "temp_install".into(),
                level: "safe".into(),
                score: 0,
                blocked: false,
                issues: Vec::new(),
                recommendations: Vec::new(),
                scanned_files: vec!["E:/skills/alpha/SKILL.md".into()],
                engine_version: "phase2-rules-v1".into(),
                scanned_at: 100,
            },
        )
        .unwrap();
        save_security_report(
            &db_path,
            &SecurityReport {
                id: "alpha-new".into(),
                skill_id: Some(alpha_id),
                skill_name: Some("Alpha".into()),
                source_path: Some("E:/skills/alpha".into()),
                scan_scope: "rescan".into(),
                level: "safe".into(),
                score: 0,
                blocked: false,
                issues: Vec::new(),
                recommendations: Vec::new(),
                scanned_files: vec!["E:/skills/alpha/SKILL.md".into()],
                engine_version: "phase2-rules-v1".into(),
                scanned_at: 200,
            },
        )
        .unwrap();
        save_security_report(
            &db_path,
            &SecurityReport {
                id: "beta-only".into(),
                skill_id: Some(beta_id),
                skill_name: Some("Beta".into()),
                source_path: Some("E:/skills/beta".into()),
                scan_scope: "temp_install".into(),
                level: "medium".into(),
                score: 40,
                blocked: false,
                issues: vec![SecurityIssue {
                    rule_id: "network_fetch".into(),
                    severity: "medium".into(),
                    title: "Review required".into(),
                    description: "demo".into(),
                    file_path: Some("E:/skills/beta/install.sh".into()),
                }],
                recommendations: vec![SecurityRecommendation {
                    action: "review_files".into(),
                    description: "review".into(),
                }],
                scanned_files: vec!["E:/skills/beta/install.sh".into()],
                engine_version: "phase2-rules-v1".into(),
                scanned_at: 150,
            },
        )
        .unwrap();
        save_security_report(
            &db_path,
            &SecurityReport {
                id: "transient-temp-scan".into(),
                skill_id: None,
                skill_name: None,
                source_path: None,
                scan_scope: "temp_install".into(),
                level: "high".into(),
                score: 90,
                blocked: true,
                issues: vec![SecurityIssue {
                    rule_id: "shell_destructive".into(),
                    severity: "high".into(),
                    title: "Blocked".into(),
                    description: "danger".into(),
                    file_path: Some("E:/tmp/install.sh".into()),
                }],
                recommendations: vec![SecurityRecommendation {
                    action: "block_install".into(),
                    description: "blocked".into(),
                }],
                scanned_files: vec!["E:/tmp/install.sh".into()],
                engine_version: "phase2-rules-v1".into(),
                scanned_at: 300,
            },
        )
        .unwrap();

        let loaded = list_security_reports(&db_path).unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "alpha-new");
        assert_eq!(loaded[0].scan_scope, "rescan");
        assert_eq!(loaded[1].id, "beta-only");
        assert!(loaded.iter().all(|report| report.skill_id.is_some()));
    }

    #[test]
    fn save_security_report_replaces_existing_report_for_same_skill() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("security.db");
        run_migrations(&db_path).unwrap();

        let skill_id = skills_repository::save_installed_skill(
            &db_path,
            &install_request("demo", "Demo Skill"),
            "E:/skills/demo",
            "safe",
            false,
        )
        .unwrap();

        save_security_report(
            &db_path,
            &SecurityReport {
                id: "report-old".into(),
                skill_id: Some(skill_id.clone()),
                skill_name: Some("Demo Skill".into()),
                source_path: Some("E:/skills/demo".into()),
                scan_scope: "canonical".into(),
                level: "safe".into(),
                score: 0,
                blocked: false,
                issues: Vec::new(),
                recommendations: Vec::new(),
                scanned_files: vec!["E:/skills/demo/SKILL.md".into()],
                engine_version: "phase2-rules-v1".into(),
                scanned_at: 100,
            },
        )
        .unwrap();
        save_security_report(
            &db_path,
            &SecurityReport {
                id: "report-new".into(),
                skill_id: Some(skill_id.clone()),
                skill_name: Some("Demo Skill".into()),
                source_path: Some("E:/skills/demo".into()),
                scan_scope: "rescan".into(),
                level: "medium".into(),
                score: 30,
                blocked: false,
                issues: vec![SecurityIssue {
                    rule_id: "network_fetch".into(),
                    severity: "medium".into(),
                    title: "Review required".into(),
                    description: "demo".into(),
                    file_path: Some("E:/skills/demo/install.sh".into()),
                }],
                recommendations: vec![SecurityRecommendation {
                    action: "review_files".into(),
                    description: "review".into(),
                }],
                scanned_files: vec!["E:/skills/demo/install.sh".into()],
                engine_version: "phase2-rules-v1".into(),
                scanned_at: 200,
            },
        )
        .unwrap();

        let conn = open_connection(&db_path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM security_reports WHERE skill_id = ?1", [skill_id], |row| {
                row.get(0)
            })
            .unwrap();
        let loaded = list_security_reports(&db_path).unwrap();

        assert_eq!(count, 1);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "report-new");
        assert_eq!(loaded[0].scan_scope, "rescan");
    }
}
