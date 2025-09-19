use anyhow::{Result, bail};

pub fn tesseract_pack_from_deepl_source(src: &str) -> Result<&'static str> {
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
