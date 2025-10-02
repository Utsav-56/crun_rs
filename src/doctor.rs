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


pub(crate) fn run_doctor() {

    static C_COMPILERS: &[&str] = &["gcc", "clang", "zig", "cl", "icc", "tcc", "pcc"];
    static CPP_COMPILERS: &[&str] = &["g++", "clang++", "cl", "icpc"];


    let mut foundCount = 0;

    println!("\x1b[1mRunning doctor...\n \x1b[0m"); // bold text

    println!("Checking for C compilers...");

    for &compiler in C_COMPILERS {
        let compailer_path = find_command(compiler);
        if !compailer_path.is_empty() {
            log_pass(&format!("Found {} at {}", compiler, compailer_path));
            foundCount += 1;
        }
    }

    if foundCount == 0 {
        log_fail("No supported compilers for C found. Please install at least one of the following compilers: clang, gcc, zig, cl, icc, tcc, pcc.");
    } else if foundCount == 1 {
        log_pass(&format!("Found {} supported compiler for C.", foundCount));
    }else {
        println!("Needed only 1 but Found {} C compilers. (very good, )", foundCount);
    }

    println!("\nChecking for C++ compilers...");
    foundCount = 0;
    for &compiler in CPP_COMPILERS {
        let compailer_path = find_command(compiler);
        if !compailer_path.is_empty() {
            log_pass(&format!("Found {} at {}", compiler, compailer_path));
            foundCount += 1;
        }
    }
    if foundCount == 0 {
        log_fail("No supported compilers for C++ found. Please install at least one of the following compilers: g++, clang++, cl, icpc.");
    } else if foundCount == 1 {
        log_pass(&format!("Found {} supported compiler for C++.", foundCount));
    }else {
        println!("Needed only 1 but Found {} C++ compilers. (very good, )", foundCount);
    }
    println!("\nDoctor finished.");





}

pub fn list_compilers(src_type: &str) {

    let compilers : &[&str] = match src_type {
        "c" => &["gcc", "clang", "zig", "cl", "icc", "tcc", "pcc"],
        "cpp" => &["g++", "clang++", "cl", "icpc"],
        "all" => &["gcc", "clang", "zig", "cl", "icc", "tcc", "pcc", "g++", "clang++", "icpc"],
        _ => &[],
    };

    for &compiler in compilers {
        if !find_command(compiler).is_empty() {
            println!("{}", compiler);
        }
    }
}