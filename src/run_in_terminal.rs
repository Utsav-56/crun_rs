

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;


#[cfg(windows)]
pub fn launch_in_external_terminal(binary_path: &str, args: &[&str]) -> std::io::Result<()> {
    let quoted_binary = format!("\"{}\"", binary_path);
    let quoted_args: Vec<String> = args.iter().map(|a| format!("\"{}\"", a)).collect();
    let arg_line = if !quoted_args.is_empty() {
        format!(" {}", quoted_args.join(" "))
    } else {
        "".to_string()
    };

    let command_line = format!(
        "start \"\" cmd /c \"{}{} & echo Press any key to exit... & pause > nul\"",
        quoted_binary, arg_line
    );

    let tmp_bat: PathBuf = env::temp_dir().join("launch_external_terminal.bat");
    let bat_content = format!("@echo off\n{}\n", command_line);
    fs::write(&tmp_bat, bat_content)?;

    Command::new("cmd.exe").arg("/C").arg(&tmp_bat).spawn()?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn launch_in_external_terminal(binary_path: &str, args: &[&str]) -> std::io::Result<()> {
    let quoted_binary = format!("\"{}\"", binary_path);
    let quoted_args: Vec<String> = args.iter().map(|a| format!("\"{}\"", a)).collect();
    let arg_line = if !quoted_args.is_empty() {
        quoted_args.join(" ")
    } else {
        "".to_string()
    };

    let sh_cmd = format!("{} {} ; echo; echo Press Enter to exit...; read -n 1", quoted_binary, arg_line);
    let cmd_str = format!("'{}'", sh_cmd);
    let script = format!(
        r#"tell application "Terminal"
activate
do script {}
end tell"#,
        cmd_str
    );

    Command::new("osascript").arg("-e").arg(script).spawn()?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn launch_in_external_terminal(binary_path: &str, args: &[&str]) -> std::io::Result<()> {
    let quoted_binary = format!("\"{}\"", binary_path);
    let quoted_args: Vec<String> = args.iter().map(|a| format!("\"{}\"", a)).collect();
    let arg_line = if !quoted_args.is_empty() {
        quoted_args.join(" ")
    } else {
        "".to_string()
    };

    let sh_cmd = format!("{} {} ; echo ; echo Press Enter to exit... ; read -n 1", quoted_binary, arg_line);

    let terminals = [
        "gnome-terminal", "konsole", "xterm", "lxterminal", "xfce4-terminal",
        "mate-terminal", "terminator", "tilix", "alacritty", "kitty", "urxvt"
    ];

    let mut selected_terminal = None;
    for term in &terminals {
        if crate::command_exists::command_exists(term) {
            selected_terminal = Some(*term);
            break;
        }
    }

    let term = match selected_terminal {
        Some(t) => t,
        None => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No supported terminal emulator found")),
    };

    let mut cmd: Command = match term {
        "gnome-terminal" | "xfce4-terminal" | "lxterminal" | "mate-terminal" | "terminator" | "tilix" => {
            let mut c = Command::new(term);
            c.args(&["--", "bash", "-c", &sh_cmd]);
            c
        }
        "konsole" => {
            let mut c = Command::new(term);
            c.args(&["-e", "bash", "-c", &sh_cmd]);
            c
        }
        "xterm" | "alacritty" | "kitty" | "urxvt" => {
            let mut c = Command::new(term);
            c.args(&["-e", "bash", "-c", &sh_cmd]);
            c
        }
        _ => {
            let mut c = Command::new(term);
            c.args(&["-e", "bash", "-c", &sh_cmd]);
            c
        }
    };

    cmd.spawn()?;
    Ok(())
}

