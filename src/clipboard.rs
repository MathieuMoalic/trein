use std::io::Write;
use std::process::{Command, Stdio};

pub fn maybe_copy_to_clipboard(copy: bool, text: &str) {
    if !copy {
        return;
    }
    if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    } else {
        eprintln!("(Tip) wl-copy not found, skipping clipboard copy.");
    }
}
