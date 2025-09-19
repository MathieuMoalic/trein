use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "trein", version, about = "Select area → OCR → DeepL translate")]
pub struct Args {
    /// Source language code: "AR" | "BG" | "CS" | "DA" | "DE" | "EL" | "EN" | "ES" | "ET" | "FI" | "FR" | "HE"
    /// | "HU" | "ID" | "IT" | "JA" | "KO" | "LT" | "LV" | "NB" | "NL" | "PL" | "PT" | "RO"
    /// | "RU" | "SK" | "SL" | "SV" | "TH" | "TR" | "UK" | "VI" | "ZH"
    #[arg(short = 's', long = "source-lang", default_value = "EN")]
    pub source_lang: String,

    /// Target language code: "AR" | "BG" | "CS" | "DA" | "DE" | "EL" | "EN" | "EN-GB" | "EN-US" | "ES" | "ES-419"
    /// | "ET" | "FI" | "FR" | "HE" | "HU" | "ID" | "IT" | "JA" | "KO" | "LT" | "LV" | "NB"
    /// | "NL" | "PL" | "PT" | "PT-BR" | "PT-PT" | "RO" | "RU" | "SK" | "SL" | "SV" | "TH"
    /// | "TR" | "UK" | "VI" | "ZH" | "ZH-HANS" | "ZH-HANT"
    #[arg(short = 't', long = "target-lang", default_value = "EN")]
    pub target_lang: String,

    /// Also copy the translation to the Wayland clipboard using wl-copy (if available).
    #[arg(short = 'c', long = "copy")]
    pub copy: bool,

    /// Optional override for the Tesseract language if you need to force it (e.g., chi_tra).
    /// Normally derived from --source-lang.
    #[arg(long = "ocr-lang")]
    pub ocr_lang: Option<String>,

    /// DeepL API key. If omitted, falls back to $DEEPL_API_KEY, then config files.
    /// NOTE: requires clap feature `env`. If you don't enable it, remove `env = ...` here.
    #[arg(long = "deepl-api-key", env = "DEEPL_API_KEY")]
    pub deepl_api_key: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_defaults() {
        let args = Args::parse_from(["trein"]);
        assert_eq!(args.source_lang, "EN");
        assert_eq!(args.target_lang, "EN");
        assert!(!args.copy);
        assert!(args.ocr_lang.is_none());
        // deepl_api_key is None unless provided
        assert!(args.deepl_api_key.is_none());
    }

    #[test]
    fn parses_all_flags() {
        let args = Args::parse_from([
            "trein",
            "-s",
            "JA",
            "-t",
            "EN-GB",
            "--copy",
            "--ocr-lang",
            "chi_tra",
            "--deepl-api-key",
            "k123",
        ]);
        assert_eq!(args.source_lang, "JA");
        assert_eq!(args.target_lang, "EN-GB");
        assert!(args.copy);
        assert_eq!(args.ocr_lang.as_deref(), Some("chi_tra"));
        assert_eq!(args.deepl_api_key.as_deref(), Some("k123"));
    }
}
