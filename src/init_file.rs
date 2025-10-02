use std::fs;
use std::path::PathBuf;

pub fn init_source_file(name: &str) -> std::io::Result<()> {
    let file_path = {
        let path = PathBuf::from(name);
        if path.extension().is_none() {
            path.with_extension("c") // default to .c
        } else {
            path
        }
    };

    if file_path.exists() {
        println!(
            "File '{}' already exists. Skipping creation.",
            file_path.display()
        );
        return Ok(());
    }

    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("c")
        .to_lowercase();

    let template = match ext.as_str() {
        "cpp" | "cxx" | "cc" => {
            r#"#include <iostream>

int main() {
    std::cout << "Hello from C++!" << std::endl;
    return 0;
}
"#
        }
        _ => {
            r#"#include <stdio.h>

int main() {
    printf("Hello from C!\n");
    return 0;
}
"#
        }
    };

    fs::write(&file_path, template)?;
    println!("Created '{}'", file_path.display());
    Ok(())
}
