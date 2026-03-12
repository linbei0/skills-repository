use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use tauri::{AppHandle, Runtime};
use tauri_plugin_opener::OpenerExt;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
enum SourceReference {
    ExternalUrl(String),
    LocalPath(PathBuf),
}

fn is_local_absolute_path(value: &str) -> bool {
    Path::new(value).is_absolute()
        || value.starts_with(r"\\")
        || (value.len() > 2
            && value.as_bytes()[1] == b':'
            && matches!(value.as_bytes()[2], b'\\' | b'/'))
}

fn parse_file_url(value: &str) -> Result<PathBuf> {
    let parsed = Url::parse(value).context("failed to parse file source url")?;
    parsed
        .to_file_path()
        .map_err(|_| anyhow!("failed to convert file source url to local path"))
}

fn parse_source_reference(reference: &str) -> Result<SourceReference> {
    let normalized = reference.trim();
    if normalized.is_empty() {
        return Err(anyhow!("source reference is empty"));
    }

    if normalized.starts_with("file://") {
        return Ok(SourceReference::LocalPath(parse_file_url(normalized)?));
    }

    if is_local_absolute_path(normalized) {
        return Ok(SourceReference::LocalPath(PathBuf::from(normalized)));
    }

    if let Ok(parsed) = Url::parse(normalized) {
        if matches!(parsed.scheme(), "http" | "https" | "mailto" | "tel") {
            return Ok(SourceReference::ExternalUrl(normalized.to_string()));
        }

        return Err(anyhow!(
            "unsupported source url scheme: {}",
            parsed.scheme()
        ));
    }
    Err(anyhow!("unsupported source reference: {}", normalized))
}

fn canonicalize_open_path(path: PathBuf) -> Result<PathBuf> {
    path.canonicalize()
        .with_context(|| format!("source path does not exist: {}", path.display()))
}

pub fn open_source_reference<R: Runtime>(app: &AppHandle<R>, reference: &str) -> Result<()> {
    match parse_source_reference(reference)? {
        SourceReference::ExternalUrl(url) => app
            .opener()
            .open_url(url, None::<&str>)
            .context("failed to open external source url"),
        SourceReference::LocalPath(path) => app
            .opener()
            .open_path(
                canonicalize_open_path(path)?.to_string_lossy().to_string(),
                None::<&str>,
            )
            .context("failed to open local source path"),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_source_reference, SourceReference};

    #[test]
    fn parses_external_https_source_reference() {
        let parsed = parse_source_reference("https://github.com/demo/repo").unwrap();
        assert_eq!(
            parsed,
            SourceReference::ExternalUrl("https://github.com/demo/repo".to_string())
        );
    }

    #[test]
    fn parses_windows_local_source_reference() {
        let parsed = parse_source_reference(r"C:\skills\teach-impeccable").unwrap();
        assert_eq!(
            parsed,
            SourceReference::LocalPath(r"C:\skills\teach-impeccable".into())
        );
    }

    #[test]
    fn parses_file_url_source_reference() {
        let parsed = parse_source_reference("file:///C:/skills/teach-impeccable").unwrap();
        assert_eq!(
            parsed,
            SourceReference::LocalPath(r"C:\skills\teach-impeccable".into())
        );
    }

    #[test]
    fn rejects_relative_source_reference() {
        let error = parse_source_reference("./skills/teach-impeccable").unwrap_err();
        assert!(error
            .to_string()
            .contains("unsupported source reference: ./skills/teach-impeccable"));
    }
}
