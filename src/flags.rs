use std::{env, process};
use crate::init_file;

#[derive(Default)]
pub(crate) struct Flags {
    pub verbose: bool,

    pub no_cache: bool,
    pub help: bool,

    pub compiler: String,
    pub extra_flags: String,

    pub output_name: String,
    pub output_dir: String,

    pub run_args: String,
    pub run_in_new_terminal: bool,


    pub check_only: bool,
    pub list_only: bool,
    pub list_for: String,

    pub init_only: bool,
    pub init_filename: String,

    pub fmt_only: bool,
    pub fmt_targets: Vec<String>,
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
        "format" => "fmt",
        other => other,
    }
}

pub fn parse_flags() -> (Flags, Vec<String>) {
    let args: Vec<String> = env::args()
        .skip(1)
        .map(|a| flag_alias(&a).to_string())
        .collect();
    let mut flags = Flags::default();
    let mut non_flags = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {

            "fmt" => {
                flags.fmt_only = true;
                let mut j = i + 1;
                while j < args.len() && !args[j].starts_with('-') {
                    flags.fmt_targets.push(args[j].clone());
                    j += 1;
                }
                if flags.fmt_targets.is_empty() {
                    // No specific files provided, format all .c/.cpp files in current directory
                    flags.fmt_targets.push(".".to_string());
                }
                break; // No more flags to process after fmt
            }

            "-v" => flags.verbose = true,
            "-n" => flags.no_cache = true,
            "-h" => flags.help = true,
            "-c" => flags.compiler = args.get(i + 1).cloned().unwrap_or_default(),
            "-e" => flags.extra_flags = args.get(i + 1).cloned().unwrap_or_default(),
            "-o" => flags.output_name = args.get(i + 1).cloned().unwrap_or_default(),
            "-d" => flags.output_dir = args.get(i + 1).cloned().unwrap_or_default(),
            "-r" => flags.run_args = args.get(i + 1).cloned().unwrap_or_default(),
            "-ntw" => flags.run_in_new_terminal = true,
            "-check" => flags.check_only = true,
            "-list-for" => {
                flags.list_only = true;
                flags.list_for = args.get(i + 1).cloned().unwrap_or_default();
                if flags.list_for.is_empty() {
                    println!("-list-for requires an argument (c or cpp or all)");
                    println!("Use -list-all to list all compilers");
                    process::exit(1);
                }
            }

            "-list-all" => {
                flags.list_only = true;
                flags.list_for = "all".to_string();
            }

            "init" => {
                flags.init_only = true;
                flags.init_filename = args.get(i + 1).cloned().unwrap_or("main.c".to_string());
                if flags.init_filename.is_empty() {
                    flags.init_filename = "main.c".to_string();
                }
                let status = init_file::init_source_file(&flags.init_filename);
                match status {
                    Ok(_) => {
                        // Successfully created or already exists
                        println!("Initialized new file: {}", flags.init_filename);
                        process::exit(0);
                    }
                    Err(_) => {
                        println!("Failed to create file: {}", flags.init_filename);
                        process::exit(1);
                    }
                }
            }

            s if s.starts_with('-') => {
                println!("Unknown flag {}", s);
                process::exit(1);
            }
            other => non_flags.push(other.to_string()),
        }
        i += if matches!(
            args[i].as_str(),
            "-c" | "-e" | "-o" | "-d" | "-r" | "-list-for" | "init"
        ) {
            2
        } else {
            1
        };
    }
    (flags, non_flags)
}

pub fn show_help() {
    println!("crun - Compile and run C/C++ files quickly");
    println!("\nUsage: crun [flags] <filename>");
    println!("\nFlags:");

    println!("  init <filename>      Initialize a new C/C++ file with a template code");
    println!(" fmt [files]        Format C/C++ source files using available formatters");

    // general releted
    println!("  -v, --verbose        Verbose mode");
    println!("  -n, --recompile      Always recompile");
    println!("  -h, --help           Show help");

    // compiler releted
    println!("  -c, --compiler <c>   Choose compiler");
    println!("  -e, --extra <flags>  Extra compiler flags");

    // output releted
    println!("  -o, --output <name>  Output binary name");
    println!("  -d, --directory <d>  Output directory");

    // run releted

    println!("  -r, --run-args <a>   Args to binary");
    println!("  -ntw, --new-terminal Run in new terminal");

    // checkup releted
    println!("  -check, --doctor     Only check for any problem in your machine");
    println!("  -list-for <c/cpp/all> List available compilers for C or C++ or all");
    println!("  -list-all            List all available compilers");
    println!("\nExample:");
    println!("  crun init my_program.c");
    println!("  crun fmt my_program.c main.cpp // format specific files");
    println!("  crun init myprogram             // creates myprogram.c by default");

    println!("  crun -v -e \"-Wall -O2\" -r \"arg1 arg2\" my_program.c");
}