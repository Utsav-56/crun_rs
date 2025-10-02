use crate::{Flags, LOG};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, process};

pub fn get_mod_time(path: &str) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(UNIX_EPOCH)
}

fn must_make_dir(path: &str) {
    if let Err(e) = fs::create_dir_all(path) {
        unsafe {
            LOG.lock().unwrap().println(
                &format_args!("Failed to create directory {}: {}", path, e),
                None,
            );
        }
        process::exit(1);
    }
}

pub fn find_source(file: &str) -> Option<String> {
    if Path::new(file).extension().is_some() {
        return Some(file.to_string());
    }
    for ext in [".c", ".cpp", ".cc", ".cxx"] {
        let candidate = format!("{}{}", file, ext);
        if Path::new(&candidate).exists() {
            unsafe {
                LOG.lock()
                    .unwrap()
                    .println(&format_args!("No extension was provided, detected source file is '{}'. If incorrect, please specify the full filename.", candidate), None);
            }
            return Some(candidate);
        }
    }
    None
}

pub fn setup_exe_path(flags: &Flags, src: &str, build_dir: &str) -> String {
    let abs_src = fs::canonicalize(src).unwrap();
    let mut name = if flags.output_name.is_empty() {
        abs_src.file_stem().unwrap().to_string_lossy().to_string()
    } else {
        flags.output_name.clone()
    };
    if cfg!(windows) && !name.ends_with(".exe") {
        name.push_str(".exe");
    }
    must_make_dir(build_dir);
    PathBuf::from(build_dir)
        .join(name)
        .to_string_lossy()
        .into_owned()
}
