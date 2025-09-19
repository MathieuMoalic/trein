pub fn print_result(
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
