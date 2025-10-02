use std::fmt::Arguments;
use std::io;
use std::io::Write;

pub struct Ulog {
    count: usize,
}

impl Ulog {
    pub(crate) fn new() -> Self {
        Ulog { count: 0 }
    }

    pub(crate) fn println(&mut self, format: &Arguments, args: Option<&[&dyn std::fmt::Display]>) {
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

    pub(crate) fn clear(&mut self) {
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