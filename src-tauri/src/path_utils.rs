pub fn display_path(value: &str) -> String {
    if let Some(stripped) = value.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{}", stripped)
    } else if let Some(stripped) = value.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::display_path;

    #[test]
    fn strips_windows_verbatim_prefix_for_local_and_unc_paths() {
        assert_eq!(
            display_path(r"\\?\C:\Users\jiang\AppData\Roaming\app\skills"),
            r"C:\Users\jiang\AppData\Roaming\app\skills"
        );
        assert_eq!(
            display_path(r"\\?\UNC\server\share\skills"),
            r"\\server\share\skills"
        );
        assert_eq!(display_path(r"D:\skills"), r"D:\skills");
    }
}
