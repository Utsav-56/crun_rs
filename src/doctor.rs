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
    let SUPPORTED_COMPILERS: &[&str] = &["clang", "gcc", "zig", "cl"];

    let mut foundCount = 0;

    println!("\x1b[1mRunning doctor...\n \x1b[0m"); // bold text

    println!("Checking for supported compilers...");

    for &compiler in SUPPORTED_COMPILERS {
        let compailer_path = find_command(compiler);
        if !compailer_path.is_empty() {
            log_pass(&format!("Found {} at {}", compiler, compailer_path));
            foundCount += 1;
        }
    }

    if foundCount == 0 {
        log_fail("No supported compilers found. Please install at least one of the following compilers: clang, gcc, zig, cl.");
    } else if foundCount == 1 {
        log_pass(&format!("Found {} supported compiler(s).", foundCount));
    }else {
        println!("Needed only 1 but Found {} supported compilers. (very good, )", foundCount);
    }
}

pub fn list_compilers(){
    let SUPPORTED_COMPILERS: &[&str] = &["clang", "gcc", "zig", "cl"];
    for &compiler in SUPPORTED_COMPILERS {
        let compailer_path = find_command(compiler);
        if !compailer_path.is_empty() {
            println!("{}", compiler)
        }
    }
}