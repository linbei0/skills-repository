use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};
use url::Url;
use uuid::Uuid;

use crate::{
    domain::{
        app_state::AppPaths,
        types::{
            BatchRepositorySkillUpdateResult, InstallSkillRequest, RepositorySkillUpdateItemResult,
            SecurityReport,
        },
    },
    path_utils::display_path,
    repositories::{
        db, security as security_repository,
        skills::{
            self as skills_repository, RepositorySkillUpdateTarget,
            UpdateRepositorySkillRecordInput,
        },
    },
    security::{self, SecurityScanSourceContext},
    services::{
        fs_utils::{copy_dir_all, ensure_clean_dir, remove_dir_if_present},
        install,
    },
};

#[derive(Debug, Clone)]
struct GithubRepoState {
    html_url: String,
    resolved_ref: String,
    description: Option<String>,
    author: Option<String>,
}

#[derive(Debug)]
struct PreparedRepositoryArchive {
    temp_root: PathBuf,
    staged_dir: PathBuf,
}

#[derive(Debug)]
struct IndexedRepositoryUpdateTarget {
    index: usize,
    target: RepositorySkillUpdateTarget,
}

impl PreparedRepositoryArchive {
    fn new(temp_root: PathBuf) -> Self {
        let staged_dir = temp_root.join("staged");
        Self {
            temp_root,
            staged_dir,
        }
    }
}

impl Drop for PreparedRepositoryArchive {
    fn drop(&mut self) {
        let _ = remove_dir_if_present(&self.temp_root);
    }
}

fn github_get_json(url: &str) -> Result<Value> {
    let response = ureq::get(url)
        .set("User-Agent", concat!("skills-manager/", env!("CARGO_PKG_VERSION")))
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|error| anyhow!("github request failed: {}", error))?;

    let body = response
        .into_string()
        .context("failed to read GitHub response body")?;
    serde_json::from_str(&body).context("failed to parse GitHub response")
}

fn download_url_bytes(url: &str) -> Result<Vec<u8>> {
    let response = ureq::get(url)
        .set("User-Agent", concat!("skills-manager/", env!("CARGO_PKG_VERSION")))
        .call()
        .map_err(|error| anyhow!("failed to download GitHub skill archive: {}", error))?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .context("failed to read downloaded GitHub archive bytes")?;
    Ok(bytes)
}

fn parse_github_repo_identity(repo_url: &str) -> Result<(String, String)> {
    let parsed = Url::parse(repo_url.trim()).context("invalid GitHub repository URL")?;
    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow!("invalid GitHub repository host"))?;
    if host != "github.com" && host != "www.github.com" {
        return Err(anyhow!(
            "only github.com public repository URLs are supported"
        ));
    }

    let segments = parsed
        .path_segments()
        .ok_or_else(|| anyhow!("invalid GitHub repository path"))?
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.len() < 2 {
        return Err(anyhow!(
            "GitHub repository URL must include owner and repository"
        ));
    }

    Ok((
        segments[0].to_string(),
        segments[1].trim_end_matches(".git").to_string(),
    ))
}

fn fetch_github_repo_state_with<F>(repo_url: &str, fetch_json: &F) -> Result<GithubRepoState>
where
    F: Fn(&str) -> Result<Value>,
{
    let (owner, repo) = parse_github_repo_identity(repo_url)?;
    let repo_api_url = format!("https://api.github.com/repos/{owner}/{repo}");
    let payload = fetch_json(&repo_api_url)?;
    let default_branch = payload
        .get("default_branch")
        .and_then(Value::as_str)
        .unwrap_or("main")
        .to_string();
    let branch_api_url =
        format!("https://api.github.com/repos/{owner}/{repo}/branches/{default_branch}");
    let branch_payload = fetch_json(&branch_api_url)?;
    let resolved_ref = branch_payload
        .get("commit")
        .and_then(|commit| commit.get("sha"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("failed to resolve GitHub default branch head commit"))?;

    Ok(GithubRepoState {
        html_url: payload
            .get("html_url")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("https://github.com/{owner}/{repo}")),
        resolved_ref,
        description: payload
            .get("description")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        author: payload
            .get("owner")
            .and_then(|owner| owner.get("login"))
            .and_then(Value::as_str)
            .map(ToString::to_string),
    })
}

fn github_download_url(repo_url: &str, resolved_ref: &str) -> String {
    format!(
        "{}/archive/{}.zip",
        repo_url.trim_end_matches('/'),
        resolved_ref
    )
}

fn github_source_url_for(repo: &GithubRepoState, skill_root: &str) -> String {
    if skill_root.trim().is_empty() {
        repo.html_url.clone()
    } else {
        format!(
            "{}/tree/{}/{}",
            repo.html_url,
            repo.resolved_ref,
            skill_root.trim_matches('/').replace('\\', "/")
        )
    }
}

fn github_package_ref(repo_url: &str, skill_root: &str) -> Option<String> {
    let parsed = Url::parse(repo_url).ok()?;
    let mut segments = parsed.path_segments()?.collect::<Vec<_>>();
    if segments.len() < 2 {
        return None;
    }

    let owner = segments.remove(0);
    let repo = segments.remove(0).trim_end_matches(".git");
    let base = format!("{owner}/{repo}");

    if skill_root.trim().is_empty() {
        Some(base)
    } else {
        Some(format!("{}@{}", base, skill_root.replace('\\', "/")))
    }
}

fn build_update_install_request(
    target: &RepositorySkillUpdateTarget,
    repo: &GithubRepoState,
) -> InstallSkillRequest {
    let skill_root = target.skill_root.clone().unwrap_or_default();
    InstallSkillRequest {
        provider: "github".to_string(),
        market_skill_id: target.slug.clone(),
        source_type: target.source_type.clone(),
        source_url: github_source_url_for(repo, &skill_root),
        repo_url: Some(repo.html_url.clone()),
        download_url: Some(github_download_url(&repo.html_url, &repo.resolved_ref)),
        package_ref: github_package_ref(&repo.html_url, &skill_root),
        manifest_path: target.manifest_path.clone(),
        skill_root: target.skill_root.clone(),
        name: target.name.clone(),
        slug: target.slug.clone(),
        description: repo.description.clone(),
        version: Some(repo.resolved_ref.clone()),
        author: repo.author.clone(),
        requested_targets: Vec::new(),
    }
}

fn build_update_result(
    target: &RepositorySkillUpdateTarget,
    status: &str,
    reason_code: &str,
    details: Value,
    previous_version: Option<String>,
    current_version: Option<String>,
    copy_distribution_count: usize,
) -> RepositorySkillUpdateItemResult {
    RepositorySkillUpdateItemResult {
        skill_id: target.skill_id.clone(),
        skill_name: target.name.clone(),
        status: status.to_string(),
        reason_code: reason_code.to_string(),
        details,
        previous_version,
        current_version,
        copy_distribution_count,
    }
}

fn restore_backup(current_path: &Path, backup_path: &Path) -> Result<()> {
    if current_path.exists() {
        remove_dir_if_present(current_path)?;
    }
    if backup_path.exists() {
        fs::rename(backup_path, current_path).with_context(|| {
            format!(
                "failed to restore canonical skill directory from backup {}",
                display_path(&backup_path.to_string_lossy())
            )
        })?;
    }
    Ok(())
}

fn display_update_path(path: &Path) -> String {
    display_path(&path.to_string_lossy())
}

fn build_swap_dir_path(canonical_path: &Path, prefix: &str) -> Result<PathBuf> {
    let parent = canonical_path
        .parent()
        .ok_or_else(|| anyhow!("canonical skill path has no parent directory"))?;
    let name = canonical_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("canonical skill path has invalid directory name"))?;

    Ok(parent.join(format!(".{name}.{prefix}.{}", Uuid::new_v4())))
}

fn prepare_repository_archive_with<D>(
    paths: &AppPaths,
    repo: &GithubRepoState,
    download_bytes: &D,
) -> Result<PreparedRepositoryArchive>
where
    D: Fn(&str) -> Result<Vec<u8>>,
{
    let prepared = PreparedRepositoryArchive::new(
        paths
            .temp_dir
            .join(format!("repository-update-{}", Uuid::new_v4())),
    );
    ensure_clean_dir(&prepared.temp_root)?;

    let download_url = github_download_url(&repo.html_url, &repo.resolved_ref);
    let downloaded_bytes = download_bytes(&download_url)?;
    install::extract_zip_bytes(&downloaded_bytes, &prepared.staged_dir)?;
    Ok(prepared)
}

#[derive(Debug, Clone)]
struct PreparedRemoteUpdate {
    repo: GithubRepoState,
    update_request: InstallSkillRequest,
    previous_version: Option<String>,
    copy_distribution_count: usize,
}

struct StagedRepositoryUpdate {
    prepared: PreparedRemoteUpdate,
    security_report: SecurityReport,
    canonical_path: PathBuf,
    next_dir: PathBuf,
    backup_dir: PathBuf,
}

enum StageNextVersionOutcome {
    Blocked(RepositorySkillUpdateItemResult),
    Ready(StagedRepositoryUpdate),
}

fn build_failed_update_result(
    target: &RepositorySkillUpdateTarget,
    error: anyhow::Error,
) -> RepositorySkillUpdateItemResult {
    build_update_result(
        target,
        "failed",
        "update_failed",
        json!({
            "error": error.to_string(),
        }),
        target.version.clone(),
        None,
        target.copy_distribution_count,
    )
}

fn format_update_failure(
    action: &str,
    error: anyhow::Error,
    restore_result: Result<()>,
) -> anyhow::Error {
    anyhow!(
        "failed to {}: {}{}",
        action,
        error,
        restore_result
            .err()
            .map(|restore_error| format!("; restore also failed: {}", restore_error))
            .unwrap_or_default()
    )
}

fn prepare_remote_update(
    target: &RepositorySkillUpdateTarget,
    repo: GithubRepoState,
) -> PreparedRemoteUpdate {
    PreparedRemoteUpdate {
        update_request: build_update_install_request(target, &repo),
        previous_version: target.version.clone(),
        copy_distribution_count: target.copy_distribution_count,
        repo,
    }
}

fn persist_blocked_update_state(
    paths: &AppPaths,
    target: &RepositorySkillUpdateTarget,
    prepared: &PreparedRemoteUpdate,
    security_report: &SecurityReport,
) -> Result<RepositorySkillUpdateItemResult> {
    let mut blocked_report = security_report.clone();
    blocked_report.id = Uuid::new_v4().to_string();
    blocked_report.skill_id = Some(target.skill_id.clone());
    blocked_report.skill_name = Some(target.name.clone());
    blocked_report.source_path = Some(target.canonical_path.clone());

    let mut conn = db::open_connection(&paths.db_file)?;
    let tx = conn.transaction()?;
    security_repository::save_security_report_in_tx(&tx, &blocked_report)?;
    skills_repository::save_operation_log_in_tx(
        &tx,
        "update",
        "skill",
        Some(&target.skill_id),
        "failed",
        "repository skill update blocked by security scan",
        Some(json!({
            "previousVersion": prepared.previous_version.clone(),
            "currentVersion": prepared.repo.resolved_ref.clone(),
            "copyDistributionCount": prepared.copy_distribution_count,
            "securityReportId": blocked_report.id,
        })),
    )?;
    tx.commit()?;

    Ok(build_update_result(
        target,
        "failed",
        "blocked_by_security_scan",
        Value::Null,
        prepared.previous_version.clone(),
        Some(prepared.repo.resolved_ref.clone()),
        prepared.copy_distribution_count,
    ))
}

fn stage_next_version(
    paths: &AppPaths,
    target: &RepositorySkillUpdateTarget,
    prepared: PreparedRemoteUpdate,
    staged_dir: &Path,
) -> Result<StageNextVersionOutcome> {
    let skill_root = install::resolve_requested_skill_root(staged_dir, &prepared.update_request)?;
    let security_report = security::scan_skill_directory_with_context(
        &skill_root,
        Some(target.skill_id.clone()),
        "repository_update",
        &SecurityScanSourceContext {
            source_url: Some(prepared.update_request.source_url.clone()),
            repo_url: prepared.update_request.repo_url.clone(),
            download_url: prepared.update_request.download_url.clone(),
            version: prepared.update_request.version.clone(),
            manifest_path: prepared.update_request.manifest_path.clone(),
            skill_root: prepared.update_request.skill_root.clone(),
        },
    )?;

    if security_report.blocked {
        return persist_blocked_update_state(paths, target, &prepared, &security_report)
            .map(StageNextVersionOutcome::Blocked);
    }

    let canonical_path = PathBuf::from(&target.canonical_path);
    if !canonical_path.exists() {
        return Err(anyhow!(
            "canonical skill path does not exist: {}",
            display_update_path(&canonical_path)
        ));
    }

    let next_dir = build_swap_dir_path(&canonical_path, "update-next")?;
    let backup_dir = build_swap_dir_path(&canonical_path, "update-backup")?;
    ensure_clean_dir(&next_dir)?;
    copy_dir_all(&skill_root, &next_dir)?;
    remove_dir_if_present(&backup_dir)?;

    Ok(StageNextVersionOutcome::Ready(StagedRepositoryUpdate {
        prepared,
        security_report,
        canonical_path,
        next_dir,
        backup_dir,
    }))
}

fn swap_canonical_dir(staged: StagedRepositoryUpdate) -> Result<StagedRepositoryUpdate> {
    fs::rename(&staged.canonical_path, &staged.backup_dir).with_context(|| {
        format!(
            "failed to move canonical skill directory into backup {}",
            display_update_path(&staged.backup_dir)
        )
    })?;

    if let Err(error) = fs::rename(&staged.next_dir, &staged.canonical_path) {
        let restore_result = restore_backup(&staged.canonical_path, &staged.backup_dir);
        return Err(format_update_failure(
            "replace canonical skill directory",
            error.into(),
            restore_result,
        ));
    }

    Ok(staged)
}

fn persist_update_state<H, S>(
    paths: &AppPaths,
    target: &RepositorySkillUpdateTarget,
    staged: StagedRepositoryUpdate,
    after_replace: H,
    after_report_saved: S,
) -> Result<RepositorySkillUpdateItemResult>
where
    H: FnOnce() -> Result<()>,
    S: FnOnce() -> Result<()>,
{
    let persist_result = (|| -> Result<()> {
        after_replace()?;

        let mut persisted_report = staged.security_report.clone();
        persisted_report.id = Uuid::new_v4().to_string();
        persisted_report.skill_id = Some(target.skill_id.clone());
        persisted_report.skill_name = Some(target.name.clone());
        persisted_report.source_path = Some(staged.canonical_path.to_string_lossy().to_string());

        let mut conn = db::open_connection(&paths.db_file)?;
        let tx = conn.transaction()?;
        security_repository::save_security_report_in_tx(&tx, &persisted_report)?;
        after_report_saved()?;
        skills_repository::update_repository_skill_record_in_tx(
            &tx,
            &target.skill_id,
            &UpdateRepositorySkillRecordInput {
                description: staged.prepared.repo.description.clone(),
                version: Some(staged.prepared.repo.resolved_ref.clone()),
                author: staged.prepared.repo.author.clone(),
                source_url: staged.prepared.update_request.source_url.clone(),
                repo_url: staged.prepared.repo.html_url.clone(),
                download_url: staged.prepared.update_request.download_url.clone(),
                package_ref: staged.prepared.update_request.package_ref.clone(),
                manifest_path: staged.prepared.update_request.manifest_path.clone(),
                skill_root: staged.prepared.update_request.skill_root.clone(),
                security_level: persisted_report.level.clone(),
                blocked: false,
                scanned_at: persisted_report.scanned_at,
            },
        )?;
        skills_repository::save_operation_log_in_tx(
            &tx,
            "update",
            "skill",
            Some(&target.skill_id),
            "success",
            "repository skill updated from GitHub",
            Some(json!({
                "previousVersion": staged.prepared.previous_version.clone(),
                "currentVersion": staged.prepared.repo.resolved_ref.clone(),
                "copyDistributionCount": staged.prepared.copy_distribution_count,
                "securityReportId": persisted_report.id,
            })),
        )?;
        tx.commit()?;
        Ok(())
    })();

    if let Err(error) = persist_result {
        let restore_result = restore_backup(&staged.canonical_path, &staged.backup_dir);
        return Err(format_update_failure(
            "persist repository skill update",
            error,
            restore_result,
        ));
    }

    remove_dir_if_present(&staged.backup_dir)?;
    let _ = remove_dir_if_present(&staged.next_dir);

    Ok(build_update_result(
        target,
        "updated",
        "updated_to_latest",
        Value::Null,
        staged.prepared.previous_version.clone(),
        Some(staged.prepared.repo.resolved_ref.clone()),
        staged.prepared.copy_distribution_count,
    ))
}

fn update_repository_skill_target_from_archive_with_hooks<H, S>(
    paths: &AppPaths,
    target: &RepositorySkillUpdateTarget,
    prepared: PreparedRemoteUpdate,
    staged_dir: &Path,
    after_replace: H,
    after_report_saved: S,
) -> Result<RepositorySkillUpdateItemResult>
where
    H: FnOnce() -> Result<()>,
    S: FnOnce() -> Result<()>,
{
    match stage_next_version(paths, target, prepared, staged_dir)? {
        StageNextVersionOutcome::Blocked(result) => Ok(result),
        StageNextVersionOutcome::Ready(staged) => {
            let swapped = swap_canonical_dir(staged)?;
            persist_update_state(paths, target, swapped, after_replace, after_report_saved)
        }
    }
}

fn update_repository_skill_target_with_hooks<F, D, H, S>(
    paths: &AppPaths,
    target: &RepositorySkillUpdateTarget,
    fetch_json: &F,
    download_bytes: &D,
    after_replace: H,
    after_report_saved: S,
) -> Result<RepositorySkillUpdateItemResult>
where
    F: Fn(&str) -> Result<Value>,
    D: Fn(&str) -> Result<Vec<u8>>,
    H: FnOnce() -> Result<()>,
    S: FnOnce() -> Result<()>,
{
    let repo = fetch_github_repo_state_with(&target.repo_url, fetch_json)?;
    let prepared = prepare_remote_update(target, repo);
    if prepared.previous_version.as_deref() == Some(prepared.repo.resolved_ref.as_str()) {
        return Ok(build_update_result(
            target,
            "skipped",
            "already_up_to_date",
            Value::Null,
            prepared.previous_version.clone(),
            prepared.previous_version.clone(),
            prepared.copy_distribution_count,
        ));
    }

    let archive = prepare_repository_archive_with(paths, &prepared.repo, download_bytes)?;
    update_repository_skill_target_from_archive_with_hooks(
        paths,
        target,
        prepared,
        &archive.staged_dir,
        after_replace,
        after_report_saved,
    )
}

fn update_repository_skill_target_with<F, D>(
    paths: &AppPaths,
    target: &RepositorySkillUpdateTarget,
    fetch_json: &F,
    download_bytes: &D,
) -> Result<RepositorySkillUpdateItemResult>
where
    F: Fn(&str) -> Result<Value>,
    D: Fn(&str) -> Result<Vec<u8>>,
{
    update_repository_skill_target_with_hooks(
        paths,
        target,
        fetch_json,
        download_bytes,
        || Ok(()),
        || Ok(()),
    )
}

pub fn update_repository_skill(
    paths: &AppPaths,
    skill_id: &str,
) -> Result<RepositorySkillUpdateItemResult> {
    let target = skills_repository::load_repository_skill_update_target(&paths.db_file, skill_id)?;
    let fetch_json = github_get_json;
    let download_bytes = download_url_bytes;
    match update_repository_skill_target_with(paths, &target, &fetch_json, &download_bytes) {
        Ok(result) => Ok(result),
        Err(error) => Ok(build_failed_update_result(&target, error)),
    }
}

fn process_repository_group<F, D>(
    paths: &AppPaths,
    group: Vec<IndexedRepositoryUpdateTarget>,
    fetch_json: &F,
    download_bytes: &D,
) -> Vec<(usize, RepositorySkillUpdateItemResult)>
where
    F: Fn(&str) -> Result<Value>,
    D: Fn(&str) -> Result<Vec<u8>>,
{
    let Some(first) = group.first() else {
        return Vec::new();
    };

    let repo = match fetch_github_repo_state_with(&first.target.repo_url, fetch_json) {
        Ok(repo) => repo,
        Err(error) => {
            let message = error.to_string();
            return group
                .into_iter()
                .map(|item| {
                    (
                        item.index,
                        build_failed_update_result(&item.target, anyhow!(message.clone())),
                    )
                })
                .collect();
        }
    };

    let mut archive_result: Option<Result<PreparedRepositoryArchive, String>> = None;
    let mut results = Vec::with_capacity(group.len());

    for item in group {
        let prepared = prepare_remote_update(&item.target, repo.clone());
        let result = if prepared.previous_version.as_deref()
            == Some(prepared.repo.resolved_ref.as_str())
        {
            build_update_result(
                &item.target,
                "skipped",
                "already_up_to_date",
                Value::Null,
                prepared.previous_version.clone(),
                prepared.previous_version.clone(),
                prepared.copy_distribution_count,
            )
        } else {
            if archive_result.is_none() {
                archive_result = Some(
                    prepare_repository_archive_with(paths, &prepared.repo, download_bytes)
                        .map_err(|error| error.to_string()),
                );
            }

            match archive_result.as_ref() {
                Some(Ok(archive)) => match update_repository_skill_target_from_archive_with_hooks(
                    paths,
                    &item.target,
                    prepared,
                    &archive.staged_dir,
                    || Ok(()),
                    || Ok(()),
                ) {
                    Ok(result) => result,
                    Err(error) => build_failed_update_result(&item.target, error),
                },
                Some(Err(message)) => {
                    build_failed_update_result(&item.target, anyhow!(message.clone()))
                }
                None => build_failed_update_result(
                    &item.target,
                    anyhow!("prepared repository archive is unavailable"),
                ),
            }
        };

        results.push((item.index, result));
    }

    results
}

fn update_github_repository_skills_with<F, D>(
    paths: &AppPaths,
    fetch_json: &F,
    download_bytes: &D,
    max_concurrency: usize,
) -> Result<BatchRepositorySkillUpdateResult>
where
    F: Fn(&str) -> Result<Value> + Sync,
    D: Fn(&str) -> Result<Vec<u8>> + Sync,
{
    let targets = skills_repository::list_repository_skill_update_targets(&paths.db_file)?;
    let mut grouped_targets = Vec::<Vec<IndexedRepositoryUpdateTarget>>::new();
    let mut group_indexes = HashMap::<String, usize>::new();

    for (index, target) in targets.into_iter().enumerate() {
        let entry = IndexedRepositoryUpdateTarget { index, target };
        if let Some(group_index) = group_indexes.get(&entry.target.repo_url).copied() {
            grouped_targets[group_index].push(entry);
        } else {
            group_indexes.insert(entry.target.repo_url.clone(), grouped_targets.len());
            grouped_targets.push(vec![entry]);
        }
    }

    let worker_count = grouped_targets.len().min(max_concurrency.max(1));
    let queue = Arc::new(Mutex::new(grouped_targets));
    let mut indexed_results = Vec::<(usize, RepositorySkillUpdateItemResult)>::new();

    thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..worker_count {
            let queue = Arc::clone(&queue);
            handles.push(scope.spawn(move || {
                let mut local_results = Vec::<(usize, RepositorySkillUpdateItemResult)>::new();
                loop {
                    let next_group = {
                        let mut guard = queue.lock().expect("repository update queue poisoned");
                        guard.pop()
                    };

                    let Some(group) = next_group else {
                        break;
                    };

                    local_results.extend(process_repository_group(
                        paths,
                        group,
                        fetch_json,
                        download_bytes,
                    ));
                }

                local_results
            }));
        }

        for handle in handles {
            indexed_results.extend(handle.join().expect("repository update worker panicked"));
        }
    });

    indexed_results.sort_by_key(|(index, _)| *index);
    let mut updated = Vec::new();
    let mut skipped = Vec::new();
    let mut failed = Vec::new();

    for (_, result) in indexed_results {
        match result.status.as_str() {
            "updated" => updated.push(result),
            "skipped" => skipped.push(result),
            _ => failed.push(result),
        }
    }

    Ok(BatchRepositorySkillUpdateResult {
        updated,
        skipped,
        failed,
    })
}

pub fn update_github_repository_skills(
    paths: &AppPaths,
) -> Result<BatchRepositorySkillUpdateResult> {
    let fetch_json = github_get_json;
    let download_bytes = download_url_bytes;
    update_github_repository_skills_with(paths, &fetch_json, &download_bytes, 4)
}


#[cfg(test)]
#[path = "repository_update/tests.rs"]
mod tests;
