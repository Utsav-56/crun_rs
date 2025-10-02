mod command_exists;
mod run_in_terminal;
mod ulog;
mod fs_and_path_helpers;
mod doctor;
mod compile_helpers;

use std::env;
use std::io::Write;
use std::path::Path;
use std::process::{self, Command, Stdio};
use std::sync::LazyLock;
use ulog::Ulog;

static SUPPORTED_COMPILERS: &[&str] = &["clang", "gcc", "zig", "cl", "bytes"];

static LOG: LazyLock<std::sync::Mutex<Ulog>> = LazyLock::new(|| std::sync::Mutex::new(Ulog::new()));

#[derive(Default)]
struct Flags {
    verbose: bool,
    no_cache: bool,
    help: bool,
    compiler: String,
    extra_flags: String,
    output_name: String,
    output_dir: String,
    run_args: String,
    run_in_new_terminal: bool,
    check_only : bool,
    list_only : bool,
}

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
    }else {
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
    let (flags, mut args) = parse_flags();

    if flags.list_only{
        doctor::list_compilers();
        return;
    }

    if flags.check_only {
        doctor::run_doctor();
        return;
    }



    if args.is_empty() {
        show_help();
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

    let needs_recompile = flags.no_cache || fs_and_path_helpers::get_mod_time(&src) > fs_and_path_helpers::get_mod_time(&exe);
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

// --- Flag Parsing ---
fn flag_alias(arg: &str) -> &str {
    match arg {
        "--verbose" => "-v",
        "--recompile" => "-n",
        "--help" => "-h",
        "--compiler" => "-c",
        "--extra" => "-e",
        "--output" => "-o",
        "--directory" => "-d",
        "--run-args" => "-r",
        "--new-terminal" => "-ntw",
        "--doctor" => "-check",
        other => other,
    }
}

fn parse_flags() -> (Flags, Vec<String>) {
    let args: Vec<String> = env::args()
        .skip(1)
        .map(|a| flag_alias(&a).to_string())
        .collect();
    let mut flags = Flags::default();
    let mut non_flags = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-v" => flags.verbose = true,
            "-n" => flags.no_cache = true,
            "-h" => flags.help = true,
            "-c" => flags.compiler = args.get(i + 1).cloned().unwrap_or_default(),
            "-e" => flags.extra_flags = args.get(i + 1).cloned().unwrap_or_default(),
            "-o" => flags.output_name = args.get(i + 1).cloned().unwrap_or_default(),
            "-d" => flags.output_dir = args.get(i + 1).cloned().unwrap_or_default(),
            "-r" => flags.run_args = args.get(i + 1).cloned().unwrap_or_default(),
            "-ntw" => flags.run_in_new_terminal = true,
            "-check"=> flags.check_only = true,
            "-list"=> flags.list_only = true,
            s if s.starts_with('-') => {
                println!("Unknown flag {}", s);
                process::exit(1);
            }
            other => non_flags.push(other.to_string()),
        }
        i += if matches!(args[i].as_str(), "-c" | "-e" | "-o" | "-d" | "-r") {
            2
        } else {
            1
        };
    }
    (flags, non_flags)
}

fn show_help() {
    println!("crun - Compile and run C/C++ files quickly");
    println!("\nUsage: crun [flags] <filename>");
    println!("\nFlags:");
    println!("  -v, --verbose        Verbose mode");
    println!("  -n, --recompile      Always recompile");
    println!("  -h, --help           Show help");
    println!("  -c, --compiler <c>   Choose compiler");
    println!("  -e, --extra <flags>  Extra compiler flags");
    println!("  -o, --output <name>  Output binary name");
    println!("  -d, --directory <d>  Output directory");
    println!("  -r, --run-args <a>   Args to binary");
    println!("  -ntw, --new-terminal Run in new terminal");
    println!("  -check, --doctor     Only check for any problem in your machine");
}


