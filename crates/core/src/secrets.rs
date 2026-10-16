//! Secret detection and masking. A value is classified the moment it is read; the clear
//! text of a secret never leaves this module — `Masked` holds only the shown form.
//!
//! Three kinds of value:
//! - a *reference*: the host expands it itself (`${VAR}`, `${env:NAME}`, `{env:VAR}`,
//!   `${{ secrets.NAME }}`, `$VAR`, `%VAR%`): never a secret, shown as written;
//! - a *secret*: the key name says so (KEY, TOKEN, SECRET, PASSWORD, …) or the value has
//!   the shape of a credential (`sk-…`, `ghp_…`, a JWT, 32+ hex, …): masked;
//! - a *plain* value: shown as written.

use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueKind {
    Plain,
    Reference,
    Secret,
}

/// A value as it may be shown. For a secret, `shown` is the masked form only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Masked {
    pub shown: String,
    pub kind: ValueKind,
}

fn key_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(key|token|secret|passw|credential|auth|bearer|api)").unwrap()
    })
}

fn shape_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(concat!(
            r"^(",
            r"sk-[A-Za-z0-9_-]{16,}",       // OpenAI / Anthropic style
            r"|gh[pousr]_[A-Za-z0-9]{20,}", // GitHub
            r"|github_pat_[A-Za-z0-9_]{20,}",
            r"|xox[abp]-[A-Za-z0-9-]{10,}", // Slack
            r"|AKIA[0-9A-Z]{16}",           // AWS access key id
            r"|AIza[0-9A-Za-z_-]{30,}",     // Google API key
            r"|ntn_[A-Za-z0-9]{20,}",       // Notion
            r"|glpat-[A-Za-z0-9_-]{20,}",   // GitLab
            r"|eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}", // JWT
            r"|[0-9a-fA-F]{32,}",           // hex blob
            r")$"
        ))
        .unwrap()
    })
}

fn reference_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^\s*(\$\{[^}]*\}|\$\{\{[^}]*\}\}|\{(env|file):[^}]*\}|\$[A-Za-z_][A-Za-z0-9_]*|%[A-Za-z_][A-Za-z0-9_]*%)\s*$").unwrap()
    })
}

/// Does the value contain a reference the host expands (anywhere in the string)?
pub fn contains_reference(value: &str) -> bool {
    value.contains("${")
        || value.contains("{env:")
        || value.contains("{file:")
        || value.contains('%') && value.matches('%').count() >= 2
}

/// Is the whole value a reference?
pub fn is_reference(value: &str) -> bool {
    reference_re().is_match(value)
}

/// Does the key name suggest a credential?
pub fn key_looks_secret(key: &str) -> bool {
    key_re().is_match(key)
}

/// Does the value have the shape of a credential? A `Bearer <token>` header counts.
pub fn value_looks_secret(value: &str) -> bool {
    let v = value.trim();
    let v = v
        .strip_prefix("Bearer ")
        .or_else(|| v.strip_prefix("bearer "))
        .unwrap_or(v)
        .trim();
    if v.len() < 16 {
        return false;
    }
    if shape_re().is_match(v) {
        return true;
    }
    // Long opaque string without spaces or path separators, mixed classes.
    v.len() >= 40
        && !v.contains(' ')
        && !v.contains('/')
        && !v.contains('.')
        && v.chars().any(|c| c.is_ascii_digit())
        && v.chars().any(|c| c.is_ascii_alphabetic())
}

/// Mask a value: first 6 and last 4 characters when long enough, otherwise dots only.
pub fn mask(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() >= 16 {
        let head: String = chars[..6].iter().collect();
        let tail: String = chars[chars.len() - 4..].iter().collect();
        format!("{head}…{tail}")
    } else {
        "••••••••".to_string()
    }
}

/// Classify a key/value pair and produce what may be shown.
pub fn classify(key: &str, value: &str) -> Masked {
    if is_reference(value) || (contains_reference(value) && !value_looks_secret(value)) {
        return Masked {
            shown: value.to_string(),
            kind: ValueKind::Reference,
        };
    }
    if key_looks_secret(key) && !value.is_empty() || value_looks_secret(value) {
        // A URL-looking value under a "secret" key name is not a secret (APIPRICEINDEX_URL).
        if key_looks_secret(key) && !value_looks_secret(value) && looks_like_url_or_path(value) {
            return Masked {
                shown: value.to_string(),
                kind: ValueKind::Plain,
            };
        }
        return Masked {
            shown: mask(value),
            kind: ValueKind::Secret,
        };
    }
    Masked {
        shown: value.to_string(),
        kind: ValueKind::Plain,
    }
}

fn looks_like_url_or_path(v: &str) -> bool {
    v.starts_with("http://")
        || v.starts_with("https://")
        || v.starts_with('/')
        || v.starts_with("~/")
        || v.eq_ignore_ascii_case("true")
        || v.eq_ignore_ascii_case("false")
        || v.chars().all(|c| c.is_ascii_digit())
}

/// A free-text value (argument, URL): masked only if a credential shape appears inside it,
/// such as `?token=…` or `Bearer …`.
pub fn scrub_text(text: &str) -> Masked {
    // query parameter with a secret-looking name
    if let Some(q) = text.find('?') {
        let (base, query) = text.split_at(q);
        let mut changed = false;
        let parts: Vec<String> = query[1..]
            .split('&')
            .map(|kv| match kv.split_once('=') {
                Some((k, v)) if key_looks_secret(k) && !v.is_empty() && !is_reference(v) => {
                    changed = true;
                    format!("{k}={}", mask(v))
                }
                _ => kv.to_string(),
            })
            .collect();
        if changed {
            return Masked {
                shown: format!("{base}?{}", parts.join("&")),
                kind: ValueKind::Secret,
            };
        }
    }
    if value_looks_secret(text) && !is_reference(text) {
        return Masked {
            shown: mask(text),
            kind: ValueKind::Secret,
        };
    }
    Masked {
        shown: text.to_string(),
        kind: ValueKind::Plain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_are_never_secrets() {
        for v in [
            "${OPENAI_API_KEY}",
            "${env:TOKEN}",
            "{env:MY_API_KEY}",
            "${{ secrets.X }}",
            "$TOKEN",
            "%API_KEY%",
            "${VAR:-default}",
        ] {
            let m = classify("API_KEY", v);
            assert_eq!(m.kind, ValueKind::Reference, "{v}");
            assert_eq!(m.shown, v);
        }
        let m = classify("Authorization", "Bearer ${env:MY_SERVICE_TOKEN}");
        assert_eq!(m.kind, ValueKind::Reference);
    }

    #[test]
    fn literal_secrets_are_masked_by_name_and_by_shape() {
        let m = classify(
            "OPENAI_API_KEY",
            "sk-proj-abcdefghijklmnopqrstuvwxyz0123456789",
        );
        assert_eq!(m.kind, ValueKind::Secret);
        assert!(!m.shown.contains("abcdefghijklmnop"));
        assert_eq!(m.shown, "sk-pro…6789");
        let m = classify("SOMETHING", "ghp_abcdefghijklmnopqrstuvwxyz0123456789");
        assert_eq!(m.kind, ValueKind::Secret);
        let m = classify(
            "Authorization",
            "Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abcdefghijklmnopqrst",
        );
        assert_eq!(m.kind, ValueKind::Secret);
        let m = classify("PASSWORD", "hunter2");
        assert_eq!(m.kind, ValueKind::Secret);
        assert_eq!(m.shown, "••••••••");
    }

    #[test]
    fn urls_and_flags_under_secret_like_names_stay_plain() {
        assert_eq!(
            classify("APIPRICEINDEX_URL", "https://apipriceindex.com").kind,
            ValueKind::Plain
        );
        assert_eq!(classify("AUTH_ENABLED", "true").kind, ValueKind::Plain);
        assert_eq!(classify("NODE_ENV", "production").kind, ValueKind::Plain);
    }

    #[test]
    fn url_query_tokens_are_scrubbed() {
        let m = scrub_text("https://mcp.example.com/sse?api_key=0123456789abcdef0123456789abcdef");
        assert_eq!(m.kind, ValueKind::Secret);
        assert!(m
            .shown
            .starts_with("https://mcp.example.com/sse?api_key=012345…"));
        assert_eq!(scrub_text("/home/u/project").kind, ValueKind::Plain);
    }
}
