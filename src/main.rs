mod command_exists;
mod run_in_terminal;

use std::{env, io};
use std::fmt::Arguments;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

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
}

fn get_mod_time(path: &str) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(UNIX_EPOCH)
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

fn detect_compiler(preferred: &str) -> String {
    if !preferred.is_empty() {
        if command_exists::command_exists(preferred) {
            return preferred.to_string(); // Return the preferred compiler if it exists
        } else {
            unsafe {
                LOG.lock().unwrap().println(
                    &format_args!("Preferred compiler '{}' not found", preferred),
                    None,
                );
            }
            process::exit(1);
        }
    }
    for &c in SUPPORTED_COMPILERS {
        if command_exists::command_exists(c) {
            return c.to_string();
        }
    }
    unsafe {
        LOG.lock()
            .unwrap()
            .println(&format_args!("No supported compiler found"), None);
    }
    process::exit(1);
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

fn compile(compiler: &str, exe: &str, source: &str, extra: &str) -> bool {
    let mut args: Vec<String> = match compiler {
        "zig" => vec!["cc".into(), "-o".into(), exe.into(), source.into()],
        "cl" => vec![format!("/Fe:{}", exe), source.into()],
        // include  _CRT_SECURE_NO_WARNINGS for clang
        "clang" => vec![
            "-o".into(),
            exe.into(),
            source.into(),
            "-Wno-deprecated-declarations".into(),
            "-D_CRT_SECURE_NO_WARNINGS".into(),
        ],
        _ => vec!["-o".into(), exe.into(), source.into()],
    };

    if !extra.is_empty() {
        let extra: Vec<String> = extra.split_whitespace().map(String::from).collect();
        if compiler == "cl" {
            args.splice(1..1, extra);
        } else {
            args.splice(args.len() - 1..args.len() - 1, extra);
        }
    }

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run_command(compiler, &arg_refs)
}

fn find_source(file: &str) -> Option<String> {
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

fn setup_exe_path(flags: &Flags, src: &str, build_dir: &str) -> String {
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

fn main() {
    let (flags, mut args) = parse_flags();
    if args.is_empty() {
        show_help();
        return;
    }

    let src = match find_source(&args.remove(0)) {
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

    let exe = setup_exe_path(&flags, &src, &build_dir);

    let needs_recompile = flags.no_cache || get_mod_time(&src) > get_mod_time(&exe);
    let compiler = detect_compiler(&flags.compiler);

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

    if !compile(&compiler, &exe, &src, &flags.extra_flags) {
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
}


struct Ulog {
    count: usize,
}

impl Ulog {
    fn new() -> Self {
        Ulog { count: 0 }
    }

    fn println(&mut self, format: &Arguments, args: Option<&[&dyn std::fmt::Display]>) {
        self.count += 1;
        if let Some(args) = args {
            let formatted = args
                .iter()
                .map(|arg| format!("{}", arg))
                .collect::<Vec<_>>()
                .join(" ");
            println!("{} {}", format, formatted);
        } else {
            println!("{}", format);
        }
    }

    fn clear(&mut self) {
        if self.count > 0 {
            clear_last_lines(self.count);
            self.count = 0;
        }
    }
}

fn clear_last_lines(n: usize) {
    for _ in 0..n {
        // Move cursor up one line
        print!("\x1B[A");
        // Clear the line
        print!("\x1B[2K");
    }
    io::stdout().flush().unwrap();
}
