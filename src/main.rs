mod command_exists;
mod compile_helpers;
mod doctor;
mod fs_and_path_helpers;
mod init_file;
mod run_in_terminal;
mod ulog;
mod fmt;
mod flags;

use std::env;
use std::io::Write;
use std::path::Path;
use std::process::{self, Command, Stdio};
use std::sync::LazyLock;
use flags::Flags;
use ulog::Ulog;
use crate::fmt::run_fmt;

static LOG: LazyLock<std::sync::Mutex<Ulog>> = LazyLock::new(|| std::sync::Mutex::new(Ulog::new()));


fn run_command(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_binary(exe: &str, run_args: &str, flags: &Flags) {
    let args: Vec<&str> = if run_args.is_empty() {
        vec![]
    } else {
        run_args.split_whitespace().collect()
    };

    if flags.run_in_new_terminal {
        unsafe {
            LOG.lock()
                .unwrap()
                .println(&format_args!("Running in new terminal..."), None);
        }
        if let Err(e) = run_in_terminal::launch_in_external_terminal(
            &exe,
            &flags.run_args.split_whitespace().collect::<Vec<&str>>(),
        ) {
            unsafe {
                LOG.lock().unwrap().println(
                    &format_args!("Failed to launch in new terminal: {}", e),
                    None,
                );
            }
        }
        return; // Prevent running the binary in the current terminal
    } else {
        unsafe {
            LOG.lock()
                .unwrap()
                .println(&format_args!("Running in current terminal..."), None);

            LOG.lock().unwrap().clear();
        }
    }

    if !Path::new(exe).exists() {
        unsafe {
            LOG.lock()
                .unwrap()
                .println(&format_args!("Executable not found: {}", exe), None);
        }
        return;
    }
    run_command(exe, &args);
}

fn main() {
    let (flags, mut args) = flags::parse_flags();

    if flags.fmt_only{
        run_fmt(flags.fmt_targets.as_slice());
        return;
    }


    if flags.list_only {
        doctor::list_compilers(flags.list_for.as_str());
        return;
    }

    if flags.check_only {
        doctor::run_doctor();
        return;
    }

    if args.is_empty() {
        flags::show_help();
        return;
    }

    let src = match fs_and_path_helpers::find_source(&args.remove(0)) {
        Some(s) => s,
        None => {
            unsafe {
                LOG.lock()
                    .unwrap()
                    .println(&format_args!("No valid source file found"), None);
            }
            process::exit(1);
        }
    };

    let build_dir = env::current_dir()
        .unwrap()
        .join(".crun")
        .to_string_lossy()
        .to_string();

    let exe = fs_and_path_helpers::setup_exe_path(&flags, &src, &build_dir);

    let needs_recompile = flags.no_cache
        || fs_and_path_helpers::get_mod_time(&src) > fs_and_path_helpers::get_mod_time(&exe);
    let compiler = compile_helpers::detect_compiler(&flags.compiler, &src);

    if !needs_recompile {
        unsafe {
            LOG.lock().unwrap().println(&format_args!("No changes detected, skipping recompilation.Extra tip:: Use '-n' flag to always recompile if needed! "), None);
        }
        return run_binary(&exe, &flags.run_args, &flags);
    }

    unsafe {
        LOG.lock()
            .unwrap()
            .println(&format_args!("Using compiler: {}", compiler), None);
    }

    if !compile_helpers::compile(&compiler, &exe, &src, &flags.extra_flags) {
        unsafe {
            LOG.lock()
                .unwrap()
                .println(&format_args!("Compilation failed"), None);
        }
        process::exit(1);
    }

    unsafe {
        LOG.lock()
            .unwrap()
            .println(&format_args!("Compilation succeeded"), None);
    }

    unsafe {
        LOG.lock()
            .unwrap()
            .println(&format_args!("Running binary..."), None);
    }

    run_binary(&exe, &flags.run_args, &flags);
}



