use std::process;
use crate::{command_exists, LOG, SUPPORTED_COMPILERS};

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
                return c.to_string();
            }
        }
    }

    // 4. No compiler found
    String::new()
}


pub fn compile(compiler: &str, exe: &str, source: &str, extra: &str) -> bool {
    use std::path::Path;

    let is_cpp = Path::new(source)
        .extension()
        .map(|ext| ext == "cpp" || ext == "cxx" || ext == "cc")
        .unwrap_or(false);

    let mut args: Vec<String> = match (compiler, is_cpp) {
        // GCC family
        ("gcc", false) | ("g++", true) => vec!["-o".into(), exe.into(), source.into()],

        // Clang family
        ("clang", false) | ("clang++", true) => vec![
            "-o".into(),
            exe.into(),
            source.into(),
            "-Wno-deprecated-declarations".into(),
            "-D_CRT_SECURE_NO_WARNINGS".into(),
        ],

        // MSVC
        ("cl", _) => vec![format!("/Fe:{}", exe), source.into()],

        // Zig as C/C++ compiler
        ("zig", _) => vec!["cc".into(), "-o".into(), exe.into(), source.into()],

        // Intel C/C++ Compiler
        ("icc", false) | ("icpc", true) => vec!["-o".into(), exe.into(), source.into()],

        // TinyCC
        ("tcc", _) => vec!["-o".into(), exe.into(), source.into()],

        // Portable C Compiler
        ("pcc", _) => vec!["-o".into(), exe.into(), source.into()],

        // LLVM's LCC
        ("lcc", _) => vec!["-o".into(), exe.into(), source.into()],

        // Watcom
        ("wcl", _) => vec![format!("-fe={}", exe), source.into()],

        // Borland
        ("bcc32", _) => vec![format!("-e{}", exe), source.into()],

        // Digital Mars
        ("dmc", _) => vec![format!("-o{}", exe), source.into()],

        // SDCC (C only, embedded)
        ("sdcc", false) => vec!["-o".into(), exe.into(), source.into()],

        // Default fallback
        (_, _) => vec!["-o".into(), exe.into(), source.into()],
    };

    // Inject extra flags
    if !extra.is_empty() {
        let extra_args: Vec<String> = extra.split_whitespace().map(String::from).collect();
        match compiler {
            "cl" => args.splice(1..1, extra_args), // insert after /Fe
            "bcc32" | "dmc" | "wcl" => args.extend(extra_args), // append
            _ => args.splice(args.len() - 1..args.len() - 1, extra_args), // before source
        }
    }

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    crate::run_command(compiler, &arg_refs)
}