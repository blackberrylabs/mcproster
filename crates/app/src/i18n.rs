//! UI strings. Nothing is hard-coded in the views: every label comes from
//! `locales/<lang>.json`, embedded in the binary, English as the fallback.

use std::collections::HashMap;

const EN: &str = include_str!("../../../locales/en.json");
const FR: &str = include_str!("../../../locales/fr.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Fr,
}

impl Lang {
    /// `"en"` / `"fr"` (case-insensitive; a longer tag such as `fr_FR.UTF-8` works).
    pub fn from_code(code: &str) -> Option<Lang> {
        let c = code.trim().to_ascii_lowercase();
        if c.starts_with("fr") {
            Some(Lang::Fr)
        } else if c.starts_with("en") {
            Some(Lang::En)
        } else {
            None
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Fr => "fr",
        }
    }
}

pub struct Strings {
    en: HashMap<String, String>,
    current: HashMap<String, String>,
}

impl Strings {
    pub fn load(lang: Lang) -> Strings {
        let en = parse(EN);
        let current = match lang {
            Lang::En => en.clone(),
            Lang::Fr => parse(FR),
        };
        Strings { en, current }
    }

    /// The string for `key`, with `{name}` placeholders replaced.
    pub fn t(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut s = self
            .current
            .get(key)
            .or_else(|| self.en.get(key))
            .cloned()
            .unwrap_or_else(|| format!("⟨{key}⟩"));
        for (k, v) in args {
            s = s.replace(&format!("{{{k}}}"), v);
        }
        s
    }
}

fn parse(text: &str) -> HashMap<String, String> {
    serde_json::from_str::<HashMap<String, String>>(text).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_locales_have_the_same_keys() {
        let en = parse(EN);
        let fr = parse(FR);
        let mut missing: Vec<&String> = en.keys().filter(|k| !fr.contains_key(*k)).collect();
        missing.sort();
        assert!(missing.is_empty(), "missing in fr.json: {missing:?}");
        let mut extra: Vec<&String> = fr.keys().filter(|k| !en.contains_key(*k)).collect();
        extra.sort();
        assert!(extra.is_empty(), "extra in fr.json: {extra:?}");
    }

    #[test]
    fn substitutes_and_falls_back() {
        let s = Strings::load(Lang::Fr);
        assert_eq!(s.t("header.servers", &[("n", "3")]), "3 serveur(s)");
        assert_eq!(s.t("no.such.key", &[]), "⟨no.such.key⟩");
    }
}
