use crate::{LOG, command_exists};
use std::process;

pub fn detect_compiler(preferred: &str, src_file: &str) -> String {
    use std::path::Path;

    static C_COMPILERS: &[&str] = &["gcc", "clang", "zig", "cl", "icc", "tcc", "pcc"];
    static CPP_COMPILERS: &[&str] = &["g++", "clang++", "cl", "icpc"];

    let ext = Path::new(src_file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let is_cpp = matches!(ext.as_str(), "cpp" | "cc" | "cxx");

    if is_cpp {
        unsafe {
            LOG.lock().unwrap().println(
                &format_args!("Detected C++ source file based on extension '.{}'", ext),
                None,
            );
        }
    } else {
        unsafe {
            LOG.lock().unwrap().println(
                &format_args!("Detected C source file based on extension '.{}'", ext),
                None,
            );
        }
    }

    // 1. Preferred compiler override
    if !preferred.is_empty() {
        if command_exists::command_exists(preferred) {
            return preferred.to_string();
        } else {
            unsafe {
                LOG.lock().unwrap().println(
                    &format_args!("Preferred compiler '{}' not found", preferred),
                    None,
                );
            }
            std::process::exit(1);
        }
    }

    // 2. Try matching compilers
    let candidates = if is_cpp { CPP_COMPILERS } else { C_COMPILERS };
    for &c in candidates {
        if command_exists::command_exists(c) {
            return c.to_string();
        }
    }

    // 3. Fallback: C file but no C compiler → try C++ compilers
    if !is_cpp {
        for &c in CPP_COMPILERS {
            if command_exists::command_exists(c) {
                unsafe {
                    LOG.lock().unwrap().println(
                        &format_args!(
                            "⚠️ No valid C compiler found. Using '{}' (a C++ compiler) for C source",
                            c
                        ),
                        None,
                    );
                }
                println!("Detected compiler: {}", c);
                return c.to_string();
            }
        }
    }

    // 4. No compiler found
    String::new()
}

pub fn compile(compiler: &str, exe: &str, source: &str, extra: &str) -> bool {
    let mut args: Vec<String> = match compiler {
        // GCC family
        "gcc" | "g++" => vec!["-o".into(), exe.into(), source.into()],

        // Clang family
        "clang" | "clang++" => vec![
            "-o".into(),
            exe.into(),
            source.into(),
            "-Wno-deprecated-declarations".into(),
            "-D_CRT_SECURE_NO_WARNINGS".into(),
        ],

        // MSVC
        "cl" => vec![format!("/Fe:{}", exe), source.into()],

        // Zig
        "zig" => vec!["cc".into(), "-o".into(), exe.into(), source.into()],

        // Intel
        "icc" | "icpc" => vec!["-o".into(), exe.into(), source.into()],

        // Lightweight / niche
        "tcc" | "pcc" | "lcc" => vec!["-o".into(), exe.into(), source.into()],
        "wcl" => vec![format!("-fe={}", exe), source.into()],
        "bcc32" => vec![format!("-e{}", exe), source.into()],
        "dmc" => vec![format!("-o{}", exe), source.into()],
        "sdcc" => vec!["-o".into(), exe.into(), source.into()],

        // Fallback
        _ => vec!["-o".into(), exe.into(), source.into()],
    };

    // Inject extra flags
    if !extra.is_empty() {
        let extra_args: Vec<String> = extra.split_whitespace().map(String::from).collect();
        match compiler {
            "cl" => {
                args.splice(1..1, extra_args);
            } // insert after /Fe
            "bcc32" | "dmc" | "wcl" => {
                args.extend(extra_args);
            } // append
            _ => {
                args.splice(args.len() - 1..args.len() - 1, extra_args);
            } // before source
        }
    }

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    crate::run_command(compiler, &arg_refs)
}
