use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use serde::Deserialize;
use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::{TempDir, tempdir};

#[derive(Parser, Debug)]
#[command(name = "trein", version, about = "Select area → OCR → DeepL translate")]
struct Args {
    /// Source language code: "AR" | "BG" | "CS" | "DA" | "DE" | "EL" | "EN" | "ES" | "ET" | "FI" | "FR" | "HE"
    /// | "HU" | "ID" | "IT" | "JA" | "KO" | "LT" | "LV" | "NB" | "NL" | "PL" | "PT" | "RO"
    /// | "RU" | "SK" | "SL" | "SV" | "TH" | "TR" | "UK" | "VI" | "ZH"
    #[arg(short = 's', long = "source-lang", default_value = "EN")]
    source_lang: String,

    /// Target language code: "AR" | "BG" | "CS" | "DA" | "DE" | "EL" | "EN" | "EN-GB" | "EN-US" | "ES" | "ES-419"
    /// | "ET" | "FI" | "FR" | "HE" | "HU" | "ID" | "IT" | "JA" | "KO" | "LT" | "LV" | "NB"
    /// | "NL" | "PL" | "PT" | "PT-BR" | "PT-PT" | "RO" | "RU" | "SK" | "SL" | "SV" | "TH"
    /// | "TR" | "UK" | "VI" | "ZH" | "ZH-HANS" | "ZH-HANT"
    #[arg(short = 't', long = "target-lang", default_value = "EN")]
    target_lang: String,

    /// Also copy the translation to the Wayland clipboard using wl-copy (if available).
    #[arg(long = "copy")]
    copy: bool,

    /// Optional override for the Tesseract pack if you need to force it (e.g., chi_tra).
    /// Normally derived from --source-lang.
    #[arg(long = "ocr-pack")]
    ocr_pack_override: Option<String>,
}

#[derive(Deserialize)]
struct DeeplTranslation {
    text: String,
    #[serde(default)]
    detected_source_language: Option<String>,
}

#[derive(Deserialize)]
struct DeeplResponse {
    translations: Vec<DeeplTranslation>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI and ensure we’re on Wayland
    let args = Args::parse();
    require_wayland()?;

    // Validate DeepL codes (strict) and decide the Tesseract pack
    let src = deepl_source(&args.source_lang)?; // e.g., "EN", "ZH"
    let tgt = deepl_target(&args.target_lang)?; // e.g., "EN-GB", "PT-BR"
    let ocr_pack = if let Some(p) = &args.ocr_pack_override {
        p.clone()
    } else {
        tesseract_pack_from_deepl_source(&src)?.to_string() // e.g., EN→eng, ZH→chi_sim
    };

    // 1) Region select → geometry
    let geometry = select_region()?;

    // 2) Screenshot to temp file
    let (tmpdir, png_path) = capture_region(&geometry)?;

    // 3) OCR with the decided Tesseract pack
    let ocr_text = ocr_image(&png_path, &ocr_pack)?;
    if ocr_text.trim().is_empty() {
        bail!("OCR returned no text. Try a larger or clearer selection, or adjust --ocr-pack.");
    }

    // 4) Translate with DeepL (use explicit source & target)
    let api_key = deepl_api_key()?;
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

fn require_wayland() -> Result<()> {
    if env::var("WAYLAND_DISPLAY").is_err() {
        bail!("This tool must run under Wayland (Hyprland). $WAYLAND_DISPLAY is not set.");
    }
    Ok(())
}

fn select_region() -> Result<String> {
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

fn capture_region(geometry: &str) -> Result<(TempDir, PathBuf)> {
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

fn ocr_image(png_path: &Path, ocr_lang: &str) -> Result<String> {
    let png = png_path
        .to_str()
        .ok_or_else(|| anyhow!("Screenshot path not valid UTF-8"))?
        .to_string();

    let out = Command::new("tesseract")
        .args([
            &png,
            &String::from("stdout"),
            &String::from("-l"),
            &String::from(ocr_lang),
        ])
        .output()
        .context("Failed to run `tesseract` (is it installed, with language data?)")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!("Tesseract failed: {stderr}");
    }

    let raw = String::from_utf8_lossy(&out.stdout).to_string();
    Ok(tidy_ocr(&raw))
}

async fn translate_deepl(
    client: &reqwest::Client,
    api_key: &str,
    base_url: &str,
    text: &str,
    target: &str,
    source_opt: Option<&str>,
) -> Result<(String, Option<String>)> {
    let url = format!("{}/v2/translate", base_url);

    // form fields
    let mut form: Vec<(String, String)> = vec![
        ("auth_key".into(), api_key.to_string()),
        ("text".into(), text.to_string()),
        ("target_lang".into(), target.to_string()),
    ];
    if let Some(src) = source_opt {
        form.push(("source_lang".into(), src.to_string()));
    }

    let resp = client
        .post(url)
        .form(&form)
        .send()
        .await
        .context("Failed to contact DeepL")?
        .error_for_status()
        .context("DeepL returned an error status")?;

    let parsed: DeeplResponse = resp.json().await.context("Invalid JSON from DeepL")?;
    let first = parsed
        .translations
        .get(0)
        .ok_or_else(|| anyhow!("No translation in response"))?;

    Ok((
        first.text.trim().to_string(),
        first.detected_source_language.clone(),
    ))
}

fn deepl_api_key() -> Result<String> {
    env::var("DEEPL_API_KEY").context("Set your DeepL key in DEEPL_API_KEY (Free or Pro).")
}

fn deepl_base_url() -> String {
    env::var("DEEPL_API_BASE").unwrap_or_else(|_| "https://api-free.deepl.com".to_string())
}

fn print_result(
    ocr_lang: &str,
    ocr_text: &str,
    target: &str,
    translation: &str,
    detected_src: Option<&str>,
) {
    println!("=== OCR (lang: {}) ===\n{}\n", ocr_lang, ocr_text.trim());
    if let Some(src) = detected_src {
        println!(
            "=== Translation → {} (detected: {}) ===\n{}\n",
            target, src, translation
        );
    } else {
        println!("=== Translation → {} ===\n{}\n", target, translation);
    }
}

fn maybe_copy_to_clipboard(copy: bool, text: &str) {
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


/// Light cleanup to make OCR text nicer for translation
fn tidy_ocr(s: &str) -> String {
    let s = s.replace('\u{00AD}', ""); // soft hyphens
    let s = s.replace("-\n", ""); // hyphenated line break
    let s = s.replace('\r', "").replace('\n', " "); // newlines → spaces
    let collapsed = s.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.trim().to_string()
}
fn deepl_source(code: &str) -> Result<String> {
    use std::fmt::Write;
    let mut up = String::with_capacity(code.len());
    for ch in code.chars() {
        up.write_char(if ch == '_' {
            '-'
        } else {
            ch.to_ascii_uppercase()
        })?;
    }
    let s = up.as_str();
    match s {
        // DeepL source list (exact)
        "AR" | "BG" | "CS" | "DA" | "DE" | "EL" | "EN" | "ES" | "ET" | "FI" | "FR" | "HE"
        | "HU" | "ID" | "IT" | "JA" | "KO" | "LT" | "LV" | "NB" | "NL" | "PL" | "PT" | "RO"
        | "RU" | "SK" | "SL" | "SV" | "TH" | "TR" | "UK" | "VI" | "ZH" => Ok(s.to_string()),
        // Helpful guidance if a target-only code is mistakenly supplied
        "EN-GB" | "EN-US" | "PT-BR" | "PT-PT" | "ZH-HANS" | "ZH-HANT" | "ES-419" => Err(anyhow!(
            "‘{}’ is a target-only DeepL code. Use the source variant (e.g., EN / PT / ZH) for --source-lang.",
            s
        )),
        _ => Err(anyhow!("Unsupported DeepL source code: {}", s)),
    }
}

fn deepl_target(code: &str) -> Result<String> {
    use std::fmt::Write;
    let mut up = String::with_capacity(code.len());
    for ch in code.chars() {
        up.write_char(if ch == '_' {
            '-'
        } else {
            ch.to_ascii_uppercase()
        })?;
    }
    let s = up.as_str();
    match s {
        "AR" | "BG" | "CS" | "DA" | "DE" | "EL" | "EN" | "EN-GB" | "EN-US" | "ES" | "ES-419"
        | "ET" | "FI" | "FR" | "HE" | "HU" | "ID" | "IT" | "JA" | "KO" | "LT" | "LV" | "NB"
        | "NL" | "PL" | "PT" | "PT-BR" | "PT-PT" | "RO" | "RU" | "SK" | "SL" | "SV" | "TH"
        | "TR" | "UK" | "VI" | "ZH" | "ZH-HANS" | "ZH-HANT" => Ok(s.to_string()),
        _ => Err(anyhow!("Unsupported DeepL target code: {}", s)),
    }
}

fn tesseract_pack_from_deepl_source(src: &str) -> Result<&'static str> {
    let pack = match src {
        "AR" => "ara",
        "BG" => "bul",
        "CS" => "ces",
        "DA" => "dan",
        "DE" => "deu",
        "EL" => "ell",
        "EN" => "eng",
        "ES" => "spa",
        "ET" => "est",
        "FI" => "fin",
        "FR" => "fra",
        "HE" => "heb",
        "HU" => "hun",
        "ID" => "ind",
        "IT" => "ita",
        "JA" => "jpn",
        "KO" => "kor",
        "LT" => "lit",
        "LV" => "lav",
        "NB" => "nor",
        "NL" => "nld",
        "PL" => "pol",
        "PT" => "por",
        "RO" => "ron",
        "RU" => "rus",
        "SK" => "slk",
        "SL" => "slv",
        "SV" => "swe",
        "TH" => "tha",
        "TR" => "tur",
        "UK" => "ukr",
        "VI" => "vie",
        "ZH" => "chi_sim",
        other => {
            bail!("No Tesseract pack mapping for DeepL source {}", other);
        }
    };
    Ok(pack)
}
