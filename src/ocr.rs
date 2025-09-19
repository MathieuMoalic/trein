use anyhow::{Context, Result, anyhow, bail};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{TempDir, tempdir};

pub fn select_region() -> Result<String> {
    let out = Command::new("slurp")
        .args(["-f", "%x,%y %wx%h"])
        .output()
        .context("Failed to run `slurp` (is it installed?)")?;

    if !out.status.success() {
        bail!("Selection cancelled or `slurp` failed.");
    }
    let geometry = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if geometry.is_empty() {
        bail!("No selection geometry received from `slurp`.");
    }
    Ok(geometry)
}

pub fn capture_region(geometry: &str) -> Result<(TempDir, PathBuf)> {
    let tmpdir = tempdir().context("Could not create temp dir")?;
    let png_path = tmpdir.path().join("capture.png");
    let png_path_str = png_path.to_string_lossy().to_string();

    let status = Command::new("grim")
        .args(["-g", geometry, &png_path_str])
        .status()
        .context("Failed to run `grim` (is it installed?)")?;
    if !status.success() {
        bail!("`grim` failed to capture the region.");
    }
    Ok((tmpdir, png_path))
}

pub fn ocr_image(png_path: &Path, ocr_lang: &str) -> Result<String> {
    let png = png_path
        .to_str()
        .ok_or_else(|| anyhow!("Screenshot path not valid UTF-8"))?
        .to_string();

    let out = Command::new("tesseract")
        .args([&png, "stdout", "-l", ocr_lang])
        .output()
        .context("Failed to run `tesseract` (is it installed, with language data?)")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!("Tesseract failed: {stderr}");
    }

    let raw = String::from_utf8_lossy(&out.stdout).to_string();
    Ok(tidy_ocr(&raw))
}

fn tidy_ocr(s: &str) -> String {
    let s = s.replace('\u{00AD}', ""); // soft hyphens
    let s = s.replace("-\n", ""); // hyphenated line break
    let s = s.replace('\r', "").replace('\n', " "); // newlines → spaces
    let collapsed = s.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.trim().to_string()
}
