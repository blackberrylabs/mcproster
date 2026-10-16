//! Interface preferences remembered between launches: language, theme, and whether to
//! ask before a probe. Stored in `$XDG_CONFIG_HOME/mcproster/settings.json` (default
//! `~/.config/mcproster/`), next to the probe results — the only directory MCProster
//! writes to (BRIEF R8).
//!
//! The language starts in **English** whatever the system language; French is a choice
//! the user makes once. `MCPROSTER_LANG` / `MCPROSTER_THEME` override for a launch.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::i18n::Lang;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// `false` once the user ticked "don't ask again" before a probe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ask_before_probe: Option<bool>,
}

impl Settings {
    pub fn path() -> Option<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        Some(base.join("mcproster").join("settings.json"))
    }

    pub fn load() -> Settings {
        Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    /// Best effort: a failure to save never bothers the user.
    pub fn save(&self) {
        let Some(p) = Self::path() else { return };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(p, json + "\n");
        }
    }

    /// Environment, then saved preference, then English.
    pub fn initial_lang(&self) -> Lang {
        if let Ok(v) = std::env::var("MCPROSTER_LANG") {
            if let Some(l) = Lang::from_code(&v) {
                return l;
            }
        }
        self.lang
            .as_deref()
            .and_then(Lang::from_code)
            .unwrap_or(Lang::En)
    }

    /// Environment, then saved preference, then light.
    pub fn initial_dark(&self) -> bool {
        match std::env::var("MCPROSTER_THEME").as_deref() {
            Ok("dark") => return true,
            Ok("light") => return false,
            _ => {}
        }
        self.theme.as_deref() == Some("dark")
    }

    pub fn asks_before_probe(&self) -> bool {
        self.ask_before_probe.unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_is_the_default_whatever_the_system_says() {
        std::env::remove_var("MCPROSTER_LANG");
        assert_eq!(Settings::default().initial_lang(), Lang::En);
        let s = Settings {
            lang: Some("fr".into()),
            ..Default::default()
        };
        assert_eq!(s.initial_lang(), Lang::Fr);
    }
}
