use std::path::Path;
use std::env;

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    use std::fs;
    fs::metadata(path)
        .map(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()),
        Some(ext) if ["exe", "bat", "cmd", "com"].contains(&ext.as_str())
    ) && path.is_file()
}

pub fn command_exists(cmd: &str) -> bool {
    let path = Path::new(cmd);
    if path.components().count() > 1 && is_executable(path) {
        return true;
    }
    env::var_os("PATH")
        .into_iter()
        .flat_map(|unparsed: std::ffi::OsString| env::split_paths(&unparsed).collect::<Vec<_>>())
        .any(|dir: std::path::PathBuf| {
            let candidate = dir.join(cmd);
            if is_executable(&candidate) {
                return true;
            }
            #[cfg(windows)]
            {
                return ["exe", "bat", "cmd", "com"]
                    .iter()
                    .any(|ext| is_executable(&candidate.with_extension(ext)));
            }
            #[allow(unreachable_code)]
            false
        })
}