use std::path::PathBuf;
use crate::command_exists;
use crate::doctor::{log_fail, log_pass};
use crate::fs_and_path_helpers::{get_home_dir, resolve_targets};

fn detect_formatter() -> Option<&'static str> {
    for tool in ["clang-format", "astyle", "uncrustify", "indent"] {
        if command_exists::command_exists(tool) {
            return Some(tool);
        }
    }
    None
}

fn format_file(tool: &str, path: &PathBuf) -> bool {
    use std::process::Command;

    let config = ensure_formatter_config(tool).unwrap();  // This returns the config path as a PathBuf.

    let status = match tool {
        "clang-format" => {
            let config_path = config.to_str().unwrap(); // Convert PathBuf to &str for args.

            // Use the absolute path to the clang-format config
            Command::new(tool)
                .args([
                    "-i",
                    path.to_str().unwrap(),  // The file you want to format
                    "--style=file",          // Tells clang-format to use the configuration file
                    &format!("--style=file:{}", config_path), // Pass the absolute path to the config file
                    "--fallback-style=LLVM",  // Optional fallback style in case config is missing
                ])
                .status()
        },

        "astyle" => Command::new(tool)
            .arg(path.to_str().unwrap())
            .status(),

        "indent" => Command::new(tool)
            .arg(path.to_str().unwrap())
            .status(),

        "uncrustify" => {
            let config_path = config.to_str().unwrap();
            Command::new(tool)
                .args([
                    "-c",
                    config_path,  // Config path
                    "-f",
                    path.to_str().unwrap(),
                    "-o",
                    path.to_str().unwrap(),
                ])
                .status()
        },

        _ => return false,
    };

    status.map(|s| s.success()).unwrap_or(false)
}




pub fn run_fmt(args: &[String]) {
    let files = resolve_targets(args);
    if files.is_empty() {
        log_fail("No C/C++ source files found to format.");
        return;
    }

    let formatter = detect_formatter();
    if let Some(tool) = formatter {
        for file in files {
            if format_file(tool, &file) {
                log_pass(&format!("Formatted {}", file.display()));
            } else {
                log_fail(&format!("Failed to format {}", file.display()));
            }
        }
    } else {
        log_fail("No supported formatter found (clang-format, astyle, uncrustify, indent).");
    }
}


fn ensure_formatter_config(tool: &str) -> Option<PathBuf> {
    use std::fs;
    use std::path::PathBuf;
    use std::env;

    let home = get_home_dir()?;
    let config_dir = home.join("crun/configs/formatters");
    let _ = fs::create_dir_all(&config_dir);

    let config_path = match tool {
        "clang-format" => config_dir.join(".clang-format"),
        "uncrustify" => config_dir.join("uncrustify.cfg"),
        _ => return None, // astyle and indent don't require config by default
    };

    if !config_path.exists() {
        let default = match tool {
            "clang-format" => r#"
BasedOnStyle: LLVM
IndentWidth: 4
ColumnLimit: 100
"#,
            "uncrustify" => r#"
indent_columns = 4
sp_brace_open = add
nl_after_func_proto = true
"#,
            _ => "",
        };
        let _ = fs::write(&config_path, default);
    }

    Some(config_path)
}
