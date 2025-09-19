use anyhow::{Result, bail};
use clap::Parser;
use tempfile::TempDir;

mod cli;
mod clipboard;
mod config;
mod ocr;
mod output;
mod tesseract;
mod translate;
mod wayland;

use crate::cli::Args;
use crate::clipboard::maybe_copy_to_clipboard;
use crate::config::{deepl_base_url, resolve_deepl_api_key};
use crate::ocr::{capture_region, ocr_image, select_region};
use crate::output::print_result;
use crate::tesseract::tesseract_pack_from_deepl_source;
use crate::translate::{deepl_source, deepl_target, translate_deepl};
use crate::wayland::require_wayland;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI and ensure we’re on Wayland
    let args = Args::parse();
    require_wayland()?;

    // Validate DeepL codes (strict) and decide the Tesseract pack
    let src = deepl_source(&args.source_lang)?; // e.g., "EN", "ZH"
    let tgt = deepl_target(&args.target_lang)?; // e.g., "EN-GB", "PT-BR"
    let ocr_pack = if let Some(p) = &args.ocr_lang {
        p.clone()
    } else {
        tesseract_pack_from_deepl_source(&src)?.to_string() // e.g., EN→eng, ZH→chi_sim
    };

    // 1) Region select → geometry
    let geometry = select_region()?;

    // 2) Screenshot to temp file
    let (tmpdir, png_path): (TempDir, std::path::PathBuf) = capture_region(&geometry)?;

    // 3) OCR with the decided Tesseract pack
    let ocr_text = ocr_image(&png_path, &ocr_pack)?;
    if ocr_text.trim().is_empty() {
        bail!("OCR returned no text. Try a larger or clearer selection, or adjust --ocr-pack.");
    }

    // 4) Translate with DeepL (use explicit source & target)
    let api_key = resolve_deepl_api_key(&args)?;
    let base = deepl_base_url();
    let client = reqwest::Client::new();

    let (translation, detected_src) =
        translate_deepl(&client, &api_key, &base, &ocr_text, &tgt, Some(&src)).await?;

    // 5) Output + optional copy
    let ocr_label = format!("{} / {}", src, ocr_pack); // show DeepL src + Tesseract pack
    print_result(
        &ocr_label,
        &ocr_text,
        &tgt,
        &translation,
        detected_src.as_deref(),
    );
    maybe_copy_to_clipboard(args.copy, &translation);

    // keep tempdir alive until here
    drop(tmpdir);
    Ok(())
}
