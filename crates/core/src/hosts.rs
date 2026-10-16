//! Host rules (`hosts/*.toml`): data model, embedded files, loading, validation. The schema
//! is documented in `docs/hosts.md`; this module is its executable counterpart and refuses
//! a rule whose location has no source.

use std::collections::{BTreeMap, HashSet};

use serde::Deserialize;
use thiserror::Error;

/// Where a location applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    User,
    Project,
    System,
}

/// Text format of a configuration file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Json,
    Jsonc,
    Toml,
    Yaml,
}

/// Shape of the node found at `servers_key`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Entry {
    /// `name → spec`.
    Object,
    /// A list of specs, each carrying its name under `name_key`.
    Array,
    /// One spec; the name is read at the document root under `name_key`.
    Single,
}

/// How a location was established.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    Documentation,
    SourceCode,
    Observed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Active,
    OutOfScope,
}

/// A string or a list of strings.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        let v: Vec<&str> = match self {
            OneOrMany::One(s) => vec![s.as_str()],
            OneOrMany::Many(v) => v.iter().map(String::as_str).collect(),
        };
        v.into_iter()
    }
}

/// Paths per operating system. `any` applies everywhere.
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
pub struct PathSpec {
    pub any: Option<OneOrMany>,
    pub linux: Option<OneOrMany>,
    pub macos: Option<OneOrMany>,
    pub windows: Option<OneOrMany>,
}

/// Where the path, format and key of a location were read.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct Source {
    pub kind: SourceKind,
    pub url: Option<String>,
    pub read_on: toml::value::Datetime,
    pub quote: String,
    pub note: Option<String>,
}

/// One file (or glob) where a host keeps declarations.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct Location {
    pub scope: Scope,
    pub format: Format,
    pub servers_key: String,
    pub entry: Entry,
    pub name_key: Option<String>,
    pub project_key: Option<String>,
    pub inputs_key: Option<String>,
    pub path: PathSpec,
    #[serde(default)]
    pub map: BTreeMap<String, OneOrMany>,
    #[serde(default)]
    pub command_is_array: bool,
    #[serde(default)]
    pub legacy: bool,
    #[serde(default)]
    pub shared_with: Vec<String>,
    pub role: Option<String>,
    pub source: Source,
}

impl Location {
    /// Field candidates for a common-model field, in order.
    pub fn mapped(&self, field: &str) -> Vec<&str> {
        match self.map.get(field) {
            Some(m) => m.iter().collect(),
            None => Vec::new(),
        }
    }

    pub fn is_registry(&self) -> bool {
        self.role.as_deref() == Some("registry")
    }
}

/// A host left out, with the reason.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct NotRead {
    pub id: String,
    pub name: String,
    pub reason: String,
}

/// One `hosts/*.toml` file.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct HostRule {
    pub id: String,
    pub name: String,
    pub vendor: Option<String>,
    pub kind: String,
    pub status: Status,
    #[serde(default)]
    pub references: Vec<String>,
    pub remove_hint: Option<String>,
    pub list_hint: Option<String>,
    #[serde(default)]
    pub locations: Vec<Location>,
    #[serde(default)]
    pub hosts: Vec<NotRead>,
}

#[derive(Debug, Error)]
pub enum RuleError {
    #[error("{file}: {source}")]
    Parse {
        file: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("{file}: {message}")]
    Invalid { file: String, message: String },
}

/// The rule files shipped inside the binary, so the tool works without a `hosts/`
/// directory next to it. Kept in sync with the repository by `include_str!`.
pub const EMBEDDED: &[(&str, &str)] = &[
    (
        "_not-read.toml",
        include_str!("../../../hosts/_not-read.toml"),
    ),
    ("amp.toml", include_str!("../../../hosts/amp.toml")),
    (
        "claude-code.toml",
        include_str!("../../../hosts/claude-code.toml"),
    ),
    (
        "claude-desktop-extensions.toml",
        include_str!("../../../hosts/claude-desktop-extensions.toml"),
    ),
    (
        "claude-desktop.toml",
        include_str!("../../../hosts/claude-desktop.toml"),
    ),
    ("cline.toml", include_str!("../../../hosts/cline.toml")),
    ("codex.toml", include_str!("../../../hosts/codex.toml")),
    (
        "continue.toml",
        include_str!("../../../hosts/continue.toml"),
    ),
    (
        "copilot-cli.toml",
        include_str!("../../../hosts/copilot-cli.toml"),
    ),
    ("cursor.toml", include_str!("../../../hosts/cursor.toml")),
    (
        "gemini-cli.toml",
        include_str!("../../../hosts/gemini-cli.toml"),
    ),
    ("goose.toml", include_str!("../../../hosts/goose.toml")),
    ("kiro.toml", include_str!("../../../hosts/kiro.toml")),
    (
        "lm-studio.toml",
        include_str!("../../../hosts/lm-studio.toml"),
    ),
    (
        "opencode.toml",
        include_str!("../../../hosts/opencode.toml"),
    ),
    (
        "qwen-code.toml",
        include_str!("../../../hosts/qwen-code.toml"),
    ),
    (
        "roo-code.toml",
        include_str!("../../../hosts/roo-code.toml"),
    ),
    ("vscode.toml", include_str!("../../../hosts/vscode.toml")),
    (
        "windsurf.toml",
        include_str!("../../../hosts/windsurf.toml"),
    ),
    ("zed.toml", include_str!("../../../hosts/zed.toml")),
];

/// A validated set of host rules.
#[derive(Debug, Clone)]
pub struct Rules {
    pub hosts: Vec<HostRule>,
    pub not_read: Vec<NotRead>,
}

impl Rules {
    /// The embedded rules, validated. Panics only if the repository ships an invalid rule,
    /// which the test suite prevents.
    pub fn embedded() -> Self {
        Self::load(EMBEDDED.iter().map(|(n, t)| (*n, *t))).expect("embedded host rules are valid")
    }

    /// Parse and validate rule texts.
    pub fn load<'a>(
        files: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self, RuleError> {
        let mut hosts = Vec::new();
        let mut not_read = Vec::new();
        let mut ids = HashSet::new();
        for (file, text) in files {
            let rule: HostRule = toml::from_str(text).map_err(|source| RuleError::Parse {
                file: file.to_string(),
                source,
            })?;
            validate(file, &rule)?;
            if !ids.insert(rule.id.clone()) {
                return Err(RuleError::Invalid {
                    file: file.to_string(),
                    message: format!("duplicate host id `{}`", rule.id),
                });
            }
            if rule.status == Status::OutOfScope {
                not_read.extend(rule.hosts.iter().cloned());
            } else {
                hosts.push(rule);
            }
        }
        hosts.sort_by_key(|h| h.name.to_lowercase());
        Ok(Rules { hosts, not_read })
    }

    pub fn host(&self, id: &str) -> Option<&HostRule> {
        self.hosts.iter().find(|h| h.id == id)
    }
}

fn validate(file: &str, rule: &HostRule) -> Result<(), RuleError> {
    let invalid = |message: String| RuleError::Invalid {
        file: file.to_string(),
        message,
    };
    if rule.id.is_empty() || rule.name.is_empty() {
        return Err(invalid("id and name are required".into()));
    }
    match rule.status {
        Status::OutOfScope => {
            if rule.hosts.is_empty() {
                return Err(invalid(
                    "an out-of-scope list needs at least one host".into(),
                ));
            }
            for h in &rule.hosts {
                if h.reason.trim().is_empty() {
                    return Err(invalid(format!(
                        "host `{}` left out without a reason",
                        h.id
                    )));
                }
            }
        }
        Status::Active => {
            if rule.locations.is_empty() {
                return Err(invalid("an active host needs at least one location".into()));
            }
            for (i, loc) in rule.locations.iter().enumerate() {
                let p = &loc.path;
                if p.any.is_none() && p.linux.is_none() && p.macos.is_none() && p.windows.is_none()
                {
                    return Err(invalid(format!("location {i}: no path")));
                }
                if loc.servers_key.trim().is_empty() && loc.role.as_deref() != Some("settings") {
                    return Err(invalid(format!("location {i}: empty servers_key")));
                }
                if loc.source.quote.trim().is_empty() {
                    return Err(invalid(format!(
                        "location {i}: a source needs the words it was read from"
                    )));
                }
                if loc.source.kind != SourceKind::Observed && loc.source.url.is_none() {
                    return Err(invalid(format!(
                        "location {i}: a documentation or source-code source needs a URL"
                    )));
                }
                if matches!(loc.entry, Entry::Array | Entry::Single) && loc.name_key.is_none() {
                    return Err(invalid(format!(
                        "location {i}: array/single entries need name_key"
                    )));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_rules_load_and_every_location_has_a_source() {
        let rules = Rules::embedded();
        assert!(rules.hosts.len() >= 19, "{} hosts", rules.hosts.len());
        assert!(rules.not_read.len() >= 6);
        let locations: usize = rules.hosts.iter().map(|h| h.locations.len()).sum();
        assert!(locations >= 38, "{locations} locations");
        for h in &rules.hosts {
            for l in &h.locations {
                assert!(
                    !l.source.quote.is_empty(),
                    "{}: location without quote",
                    h.id
                );
            }
        }
    }

    #[test]
    fn a_location_without_source_is_refused() {
        let text = r#"
id = "x"
name = "X"
kind = "cli"
status = "active"
[[locations]]
scope = "user"
format = "json"
servers_key = "mcpServers"
entry = "object"
path.any = "~/x.json"
[locations.source]
kind = "documentation"
read_on = 2026-10-16
quote = "words"
"#;
        let err = Rules::load([("x.toml", text)]).unwrap_err();
        assert!(err.to_string().contains("needs a URL"), "{err}");
    }
}
