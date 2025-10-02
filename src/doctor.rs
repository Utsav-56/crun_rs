use crate::command_exists::find_command;

pub fn log_pass(message: &str) {
    // [✓] with green ✓ and white brackets
    let tick = "\x1b[32m✓\x1b[0m"; // green tick
    let prefix = format!("[{}]", tick); // white brackets
    println!("{} \x1b[32m{}\x1b[0m", prefix, message); // green message
}

pub fn log_critical(message: &str) {
    // [-] all white, yellow message
    let prefix = "[-]";
    println!("{} \x1b[33m{}\x1b[0m", prefix, message); // yellow message
}

pub fn log_fail(message: &str) {
    // [✗] with red ✗ and white brackets
    let cross = "\x1b[31m✗\x1b[0m"; // red cross
    let prefix = format!("[{}]", cross); // white brackets
    println!("{} \x1b[31m{}\x1b[0m", prefix, message); // red message
}

pub fn list_compilers(src_type: &str) {
    let compilers: &[&str] = match src_type {
        "c" => &["gcc", "clang", "zig", "cl", "icc", "tcc", "pcc"],
        "cpp" => &["g++", "clang++", "cl", "icpc"],
        "all" => &[
            "gcc", "clang", "zig", "cl", "icc", "tcc", "pcc", "g++", "clang++", "icpc",
        ],
        _ => &[],
    };

    for &compiler in compilers {
        if !find_command(compiler).is_empty() {
            println!("{}", compiler);
        }
    }
}

use crate::compile_helpers::compile;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn prepare_dummy_sources() -> (PathBuf, PathBuf) {
    let home = if cfg!(windows) {
        env::var("USERPROFILE").expect("USERPROFILE not set")
    } else {
        env::var("HOME").expect("HOME not set")
    };

    let base = Path::new(&home).join(".crun/doctor/dummy");
    let _ = fs::create_dir_all(&base);

    let c_path = base.join("dummy.c");
    let cpp_path = base.join("dummy.cpp");

    let c_code = r#"#include <stdio.h>
int main() {
    printf("Hello, C World!\n");
    return 0;
}
"#;

    let cpp_code = r#"#include <iostream>
int main() {
    std::cout << "Hello, C++ World!" << std::endl;
    return 0;
}
"#;

    fs::write(&c_path, c_code).unwrap();
    fs::write(&cpp_path, cpp_code).unwrap();

    (c_path, cpp_path)
}

fn check_compiler_validity(compiler: &str, source: &Path) -> bool {
    use std::process::Command;

    let exe_path = source.with_extension("out");

    let status = compile(
        compiler,
        exe_path.to_str().unwrap(),
        source.to_str().unwrap(),
        "",
    );

    if status {
        let run_status = Command::new(&exe_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        let _ = fs::remove_file(&exe_path);
        run_status
    } else {
        false
    }
}

pub(crate) fn run_doctor() {
    static C_COMPILERS: &[&str] = &["gcc", "clang", "zig", "cl", "icc", "tcc", "pcc"];
    static CPP_COMPILERS: &[&str] = &["g++", "clang++", "cl", "icpc"];

    let (c_src, cpp_src) = prepare_dummy_sources();

    println!("\x1b[1mRunning doctor...\n\x1b[0m");

    println!("Checking for C compilers...");
    let mut found_c = 0;
    for &compiler in C_COMPILERS {
        let path = find_command(compiler);
        if !path.is_empty() {
            if check_compiler_validity(compiler, &c_src) {
                log_pass(&format!("{} found at {}", compiler, path));
                found_c += 1;
            } else {
                log_fail(&format!(
                    "{} found but failed to compile/run C code.",
                    compiler
                ));
            }
        }
    }
    if found_c == 0 {
        log_fail("No working C compilers found.");
    }

    println!("\nChecking for C++ compilers...");
    let mut found_cpp = 0;
    for &compiler in CPP_COMPILERS {
        let path = find_command(compiler);
        if !path.is_empty() {
            if check_compiler_validity(compiler, &cpp_src) {
                // let version = detect_compiler_version(compiler);
                log_pass(&format!("{} found at {}", compiler, path));
                found_cpp += 1;
            } else {
                log_fail(&format!(
                    "{} found but failed to compile/run C code.",
                    compiler
                ));
            }
        }
    }
    if found_cpp == 0 {
        log_fail("No working C++ compilers found.");
    }

    println!("\nDoctor finished.");
}

//
// fn detect_compiler_version(compiler: &str) -> String {
//     use std::process::Command;
//
//     let version_args = match compiler {
//         "gcc" | "g++" | "clang" | "clang++" | "icc" | "icpc" => vec!["--version"],
//         "cl" => vec!["/Bv"], // MSVC version
//         "zig" => vec!["version"],
//         "tcc" => vec!["-v"],
//         "pcc" => vec!["-v"],
//         "lcc" => vec!["-version"],
//         "wcl" => vec!["-h"],
//         "bcc32" => vec!["-v"],
//         "dmc" => vec!["-v"],
//         "sdcc" => vec!["--version"],
//         _ => return "unknown".into(),
//     };
//
//     let output = Command::new(compiler)
//         .args(&version_args)
//         .output()
//         .ok()
//         .and_then(|o| String::from_utf8(o.stdout).ok())
//         .unwrap_or_default();
//
//     // Extract first line or fallback
//     output
//         .lines()
//         .next()
//         .unwrap_or("unknown")
//         .trim()
//         .to_string()
// }
