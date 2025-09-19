use anyhow::{bail, Result};
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

#[cfg(test)]
mod tests {
    use super::*; // brings resolve_deepl_api_key, deepl_base_url into scope
    use crate::cli::Args; // your real CLI struct
    use serial_test::serial;
    use std::{env, fs};
    use tempfile::TempDir;

    // ----------------- helpers -----------------

    fn make_args(key: Option<&str>) -> Args {
        Args {
            source_lang: "EN".to_string(),
            target_lang: "EN".to_string(),
            copy: false,
            ocr_lang: None,
            deepl_api_key: key.map(|s| s.to_string()),
        }
    }

    /// Save selected env vars, run `f`, then restore them.
    fn with_env_guard<F: FnOnce()>(keys: &[&str], f: F) {
        let saved: Vec<(String, Option<String>)> = keys
            .iter()
            .map(|k| (k.to_string(), env::var(k).ok()))
            .collect();
        f();
        for (k, v) in saved {
            match v {
                Some(val) => unsafe { env::set_var(&k, val) },
                None => unsafe { env::remove_var(&k) },
            }
        }
    }

    fn write_xdg_config(dir: &std::path::Path, contents: &str) {
        let p = dir.join("trein").join("config.toml");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, contents).unwrap();
    }

    fn write_home_config(dir: &std::path::Path, contents: &str) {
        let p = dir.join(".config").join("trein").join("config.toml");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, contents).unwrap();
    }

    // ----------------- tests -------------------

    #[test]
    #[serial]
    fn cli_wins_over_env_and_config() {
        with_env_guard(&["DEEPL_API_KEY", "XDG_CONFIG_HOME", "HOME"], || {
            unsafe { env::set_var("DEEPL_API_KEY", "ENV_VAL") };

            let xdg = TempDir::new().unwrap();
            unsafe { env::set_var("XDG_CONFIG_HOME", xdg.path()) };
            write_xdg_config(xdg.path(), "DEEPL_API_KEY=FILE_VAL");

            let home = TempDir::new().unwrap();
            unsafe { env::set_var("HOME", home.path()) };
            write_home_config(home.path(), "HOME_FILE_VAL");

            let got = resolve_deepl_api_key(&make_args(Some("CLI_VAL"))).unwrap();
            assert_eq!(got, "CLI_VAL");
        });
    }

    #[test]
    #[serial]
    fn env_used_when_cli_missing_or_empty() {
        with_env_guard(&["DEEPL_API_KEY", "XDG_CONFIG_HOME", "HOME"], || {
            unsafe {
                env::set_var("DEEPL_API_KEY", "ENV_KEY");
                env::remove_var("XDG_CONFIG_HOME");
                env::remove_var("HOME");
            }

            // Empty CLI should be ignored
            let got = resolve_deepl_api_key(&make_args(Some(""))).unwrap();
            assert_eq!(got, "ENV_KEY");

            // Or when CLI is absent
            let got2 = resolve_deepl_api_key(&make_args(None)).unwrap();
            assert_eq!(got2, "ENV_KEY");
        });
    }

    #[test]
    #[serial]
    fn whitespace_only_env_is_ignored_then_xdg_config_is_used_raw_value() {
        with_env_guard(&["DEEPL_API_KEY", "XDG_CONFIG_HOME", "HOME"], || {
            unsafe {
                env::set_var("DEEPL_API_KEY", "   "); // ignored
                env::remove_var("HOME");
            }

            let xdg = TempDir::new().unwrap();
            unsafe { env::set_var("XDG_CONFIG_HOME", xdg.path()) };
            // Raw value (no DEEPL_API_KEY= prefix)
            write_xdg_config(xdg.path(), "FILE_KEY");

            let got = resolve_deepl_api_key(&make_args(None)).unwrap();
            assert_eq!(got, "FILE_KEY");
        });
    }

    #[test]
    #[serial]
    fn home_config_with_prefix_is_read() {
        with_env_guard(&["DEEPL_API_KEY", "XDG_CONFIG_HOME", "HOME"], || {
            unsafe {
                env::remove_var("DEEPL_API_KEY");
                env::remove_var("XDG_CONFIG_HOME");
            }

            let home = TempDir::new().unwrap();
            unsafe { env::set_var("HOME", home.path()) };
            write_home_config(home.path(), "DEEPL_API_KEY=ABC123\n");

            let got = resolve_deepl_api_key(&make_args(None)).unwrap();
            assert_eq!(got, "ABC123");
        });
    }

    #[test]
    #[serial]
    fn errors_when_no_sources_available() {
        with_env_guard(&["DEEPL_API_KEY", "XDG_CONFIG_HOME", "HOME"], || {
            unsafe {
                env::remove_var("DEEPL_API_KEY");
                env::remove_var("XDG_CONFIG_HOME");
                env::remove_var("HOME");
            }

            let err = resolve_deepl_api_key(&make_args(None)).unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("Set your DeepL key via --deepl-api-key")
                    && msg.contains("$XDG_CONFIG_HOME/trein/config.toml")
                    && msg.contains("$HOME/.config/trein/config.toml"),
                "unexpected error message: {msg}"
            );
        });
    }

    #[test]
    #[serial]
    fn deepl_base_url_default_and_override() {
        with_env_guard(&["DEEPL_API_BASE"], || {
            unsafe { env::remove_var("DEEPL_API_BASE") };
            assert_eq!(deepl_base_url(), "https://api-free.deepl.com");

            unsafe { env::set_var("DEEPL_API_BASE", "https://example.invalid") };
            assert_eq!(deepl_base_url(), "https://example.invalid");
        });
    }
}
