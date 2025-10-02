use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};


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
        "start \"\" cmd /c \"{}{} & echo =============== Program Finished ===============  & echo Press any key to exit... & pause > nul\"",
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

    let sh_cmd = format!("{} {} ; echo =============== Program Finished ===============; echo Press Enter to exit...; read -n 1", quoted_binary, arg_line);
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
    use std::process::Command;

    let quoted_binary = format!("\"{}\"", binary_path);
    let quoted_args: Vec<String> = args.iter().map(|a| format!("\"{}\"", a)).collect();
    let arg_line = quoted_args.join(" ");
    let sh_cmd = format!(
        "{} {} ; echo =============== Program Finished ===============; echo Program Finished; echo Press Enter to exit... ; read -n 1",
        quoted_binary, arg_line
    );

    // Universal first
    let primary_terminals = ["x-terminal-emulator", "xterm"];

    // Exhaustive fallback list
    let fallback_terminals = [
        // GNOME & GTK
        "gnome-terminal", "tilix", "terminator", "mate-terminal", "lxterminal", "xfce4-terminal",
        "deepin-terminal", "pantheon-terminal", "elementary-terminal",

        // KDE & Qt
        "konsole", "yakuake", "qterminal",

        // Lightweight / minimalist
        "alacritty", "kitty", "urxvt", "rxvt", "st", "eterm", "sakura", "tilda", "guake",
        "cool-retro-term", "lxterm", "mlterm", "roxterm", "termite", "xvt",

        // Wayland-native or experimental
        "foot", "wezterm", "warp-terminal",

        // Others / niche
        "hyper", "terminal", "gnustep-terminal", "terminology", "yeahconsole", "evilvte",
        "gnome-console", "blackbox-terminal", "wterm", "x3270", "aterm", "zterm", "finalterm"
    ];

    let terminal = primary_terminals
        .iter()
        .chain(fallback_terminals.iter())
        .find(|term| crate::command_exists::command_exists(term))
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No terminal emulator found"))?;

    let args = match *terminal {
        // Terminals that require "--" before bash -c
        "gnome-terminal" | "xfce4-terminal" | "lxterminal" | "mate-terminal" |
        "terminator" | "tilix" | "deepin-terminal" | "pantheon-terminal" |
        "elementary-terminal" | "gnome-console" => vec!["--", "bash", "-c", &sh_cmd],

        // Most others use -e
        _ => vec!["-e", "bash", "-c", &sh_cmd],
    };

    Command::new(terminal).args(args).spawn()?;
    Ok(())
}
