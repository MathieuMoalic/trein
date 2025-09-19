use anyhow::{Result, bail};
use std::path::PathBuf;
use std::{env, fs};

use crate::cli::Args;

pub fn resolve_deepl_api_key(args: &Args) -> Result<String> {
    // 1) CLI flag (also populated by env if clap `env` feature is enabled)
    if let Some(k) = args.deepl_api_key.as_deref().filter(|s| !s.is_empty()) {
        return Ok(k.to_string());
    }

    // 2) Env var (explicit fallback)
    if let Ok(k) = env::var("DEEPL_API_KEY") {
        if !k.trim().is_empty() {
            return Ok(k);
        }
    }

    // 3) Config files
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        candidates.push(PathBuf::from(xdg).join("trein/config.toml"));
    }
    if let Ok(home) = env::var("HOME") {
        candidates.push(PathBuf::from(home).join(".config/trein/config.toml"));
    }

    for p in candidates {
        if let Ok(content) = fs::read_to_string(&p) {
            let line = content.trim();
            if line.is_empty() {
                continue;
            }
            // Accept either "DEEPL_API_KEY=..." or just the raw value.
            let value = if let Some(rest) = line.strip_prefix("DEEPL_API_KEY=") {
                rest.trim()
            } else {
                line
            };
            if !value.is_empty() {
                return Ok(value.to_string());
            }
        }
    }

    bail!(
        "Set your DeepL key via --deepl-api-key, $DEEPL_API_KEY, or a config file at \
         $XDG_CONFIG_HOME/trein/config.toml (or $HOME/.config/trein/config.toml) with a single line: \
         DEEPL_API_KEY=..."
    );
}

pub fn deepl_base_url() -> String {
    std::env::var("DEEPL_API_BASE").unwrap_or_else(|_| "https://api-free.deepl.com".to_string())
}
