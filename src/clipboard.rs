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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn no_op_when_copy_false() {
        maybe_copy_to_clipboard(false, "anything");
    }

    #[test]
    #[serial]
    fn best_effort_when_wl_copy_missing() {
        // Ensure wl-copy isn't found
        unsafe {
            std::env::set_var("PATH", "");
        }
        // Should not panic; will eprintln! a tip.
        maybe_copy_to_clipboard(true, "hello");
        // no assertions; just verifying it doesn't crash
    }
}
