use anyhow::{Result, bail};

pub fn require_wayland() -> Result<()> {
    if std::env::var("WAYLAND_DISPLAY").is_err() {
        bail!("This tool must run under Wayland (Hyprland). $WAYLAND_DISPLAY is not set.");
    }
    Ok(())
}
