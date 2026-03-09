use anyhow::Result;
use std::{fs, path::Path};
use walkdir::WalkDir;

use crate::domain::types::{SecurityIssue, SecurityRecommendation, SecurityReport};

const HIGH_RISK_PATTERNS: &[(&str, &str)] = &[
    ("rm -rf", "destructive_shell_command"),
    ("invoke-expression", "powershell_invoke_expression"),
    ("powershell -enc", "encoded_powershell"),
];

const MEDIUM_RISK_PATTERNS: &[(&str, &str)] = &[
    ("curl ", "network_fetch"),
    ("wget ", "network_fetch"),
    ("chmod +x", "permission_change"),
    ("sudo ", "elevated_command"),
];

fn classify_level(score: u32) -> &'static str {
    match score {
        80..=u32::MAX => "high",
        30..=79 => "medium",
        1..=29 => "low",
        _ => "safe",
    }
}

pub fn scan_skill_directory(
    path: &Path,
    skill_id: Option<String>,
    scan_scope: &str,
) -> Result<SecurityReport> {
    let mut score = 0;
    let mut issues = Vec::new();
    let mut scanned_files = Vec::new();

    for entry in WalkDir::new(path) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let file_path = entry.path();
        scanned_files.push(file_path.to_string_lossy().to_string());

        let Ok(content) = fs::read_to_string(file_path) else {
            continue;
        };
        let lower_content = content.to_ascii_lowercase();

        for (pattern, rule_id) in HIGH_RISK_PATTERNS {
            if lower_content.contains(pattern) {
                score += 90;
                issues.push(SecurityIssue {
                    rule_id: (*rule_id).to_string(),
                    severity: "high".to_string(),
                    title: "High-risk command detected".to_string(),
                    description: format!("Detected high-risk pattern `{}`", pattern),
                    file_path: Some(file_path.to_string_lossy().to_string()),
                });
            }
        }

        for (pattern, rule_id) in MEDIUM_RISK_PATTERNS {
            if lower_content.contains(pattern) {
                score += 30;
                issues.push(SecurityIssue {
                    rule_id: (*rule_id).to_string(),
                    severity: "medium".to_string(),
                    title: "Review required".to_string(),
                    description: format!("Detected medium-risk pattern `{}`", pattern),
                    file_path: Some(file_path.to_string_lossy().to_string()),
                });
            }
        }
    }

    let level = classify_level(score).to_string();
    let blocked = matches!(level.as_str(), "high" | "critical");
    let recommendations = if blocked {
        vec![SecurityRecommendation {
            action: "block_install".to_string(),
            description: "Remove or review high-risk commands before installing this skill."
                .to_string(),
        }]
    } else if !issues.is_empty() {
        vec![SecurityRecommendation {
            action: "review_files".to_string(),
            description:
                "Review the matched files before using this skill in production workflows."
                    .to_string(),
        }]
    } else {
        vec![SecurityRecommendation {
            action: "proceed".to_string(),
            description: "No risky patterns were detected in the current scan.".to_string(),
        }]
    };

    Ok(SecurityReport {
        id: uuid::Uuid::new_v4().to_string(),
        skill_id,
        skill_name: None,
        source_path: Some(path.to_string_lossy().to_string()),
        scan_scope: scan_scope.to_string(),
        level,
        score,
        blocked,
        issues,
        recommendations,
        scanned_files,
        engine_version: "phase2-rules-v1".to_string(),
        scanned_at: time::OffsetDateTime::now_utc().unix_timestamp(),
    })
}
