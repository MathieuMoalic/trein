use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

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

pub async fn translate_deepl(
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

pub fn deepl_source(code: &str) -> Result<String> {
    use anyhow::anyhow;
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

pub fn deepl_target(code: &str) -> Result<String> {
    use anyhow::anyhow;
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
