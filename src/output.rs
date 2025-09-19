pub fn print_result(
    ocr_lang: &str,
    ocr_text: &str,
    target: &str,
    translation: &str,
    detected_src: Option<&str>,
) {
    println!(
        "{}",
        render_result(ocr_lang, ocr_text, target, translation, detected_src)
    );
}

pub fn render_result(
    ocr_lang: &str,
    ocr_text: &str,
    target: &str,
    translation: &str,
    detected_src: Option<&str>,
) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "=== OCR (lang: {}) ===\n{}\n\n",
        ocr_lang,
        ocr_text.trim()
    ));
    match detected_src {
        Some(src) => s.push_str(&format!(
            "=== Translation → {} (detected: {}) ===\n{}\n\n",
            target, src, translation
        )),
        _ => s.push_str(&format!(
            "=== Translation → {} ===\n{}\n\n",
            target, translation
        )),
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_with_detected_source() {
        let out = render_result("EN / eng", "hello", "FR", "bonjour", Some("EN"));
        assert!(out.contains("=== OCR (lang: EN / eng) ==="));
        assert!(out.contains("hello"));
        assert!(out.contains("Translation → FR (detected: EN)"));
        assert!(out.contains("bonjour"));
    }

    #[test]
    fn render_without_detected_source() {
        let out = render_result("ZH / chi_sim", "你好", "EN-GB", "hello", None);
        assert!(out.contains("=== OCR (lang: ZH / chi_sim) ==="));
        assert!(out.contains("Translation → EN-GB ==="));
        assert!(out.contains("hello"));
    }
}
