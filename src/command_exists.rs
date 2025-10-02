use std::env;
use std::path::Path;

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
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

pub fn find_command(cmd: &str) -> String {
    if let Some(paths) = env::var_os("PATH") {
        for dir in env::split_paths(&paths) {
            let candidate = dir.join(cmd);
            if is_executable(&candidate) {
                return candidate.to_string_lossy().to_string();
            }
            #[cfg(windows)]
            {
                for ext in ["exe", "bat", "cmd", "com"] {
                    if is_executable(&candidate.with_extension(ext)) {
                        return candidate.with_extension(ext).to_string_lossy().to_string();
                    }
                }
            }
        }
    }
    String::new()
}
