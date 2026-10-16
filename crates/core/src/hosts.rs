//! MCProster's host rules: the common schema comes from `hostrules`; this module adds the
//! fields an MCP inventory needs in every location ([`McpLocation`]), embeds the rule files
//! shipped in `hosts/*.toml`, and validates what is specific to MCP.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub use hostrules::rules::{
    Format, NotRead, OneOrMany, PathSpec, RuleError, Scope, Source, SourceKind, Status,
};

/// Shape of the node found at `servers_key`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Entry {
    /// `name → spec`.
    Object,
    /// A list of specs, each carrying its name under `name_key`.
    Array,
    /// One spec; the name is read at the document root under `name_key`.
    Single,
}

/// What an MCP inventory needs to know about a location, on top of where the file is.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpLocation {
    /// Key path of the servers node (`mcpServers`, `mcp.servers`, `projects.*.mcpServers`).
    #[serde(default)]
    pub servers_key: String,
    #[serde(default = "default_entry")]
    pub entry: Entry,
    pub name_key: Option<String>,
    pub inputs_key: Option<String>,
    /// Field candidates per common-model field (`command = ["command", "cmd"]`).
    #[serde(default)]
    pub map: BTreeMap<String, OneOrMany>,
    #[serde(default)]
    pub command_is_array: bool,
}

fn default_entry() -> Entry {
    Entry::Object
}

impl McpLocation {
    /// Field candidates for a common-model field, in order.
    pub fn mapped(&self, field: &str) -> Vec<&str> {
        match self.map.get(field) {
            Some(m) => m.iter().collect(),
            None => Vec::new(),
        }
    }
}

impl hostrules::Extension for McpLocation {
    fn validate(loc: &Location) -> Result<(), String> {
        if loc.ext.servers_key.trim().is_empty() && loc.role.as_deref() != Some("settings") {
            return Err("empty servers_key".into());
        }
        if matches!(loc.ext.entry, Entry::Array | Entry::Single) && loc.ext.name_key.is_none() {
            return Err("array/single entries need name_key".into());
        }
        Ok(())
    }
}

pub type Location = hostrules::Location<McpLocation>;
pub type HostRule = hostrules::HostRule<McpLocation>;
pub type Rules = hostrules::Rules<McpLocation>;

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

/// The embedded rules, validated. Panics only if the repository ships an invalid rule,
/// which the test suite prevents.
pub fn embedded() -> Rules {
    Rules::load(EMBEDDED.iter().map(|(n, t)| (*n, *t))).expect("embedded host rules are valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_rules_load_and_every_location_has_a_source() {
        let rules = embedded();
        assert!(rules.hosts.len() >= 19, "{} hosts", rules.hosts.len());
        assert!(rules.not_read.len() >= 6);
        assert!(rules.locations() >= 38, "{} locations", rules.locations());
        for h in &rules.hosts {
            for l in &h.locations {
                assert!(
                    !l.source.quote.is_empty(),
                    "{}: location without quote",
                    h.id
                );
            }
        }
        assert_eq!(
            rules.host("claude-code").unwrap().hint("remove"),
            Some("claude mcp remove <name> --scope <user|local|project>")
        );
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

    #[test]
    fn an_array_entry_without_name_key_is_refused() {
        let text = r#"
id = "x"
name = "X"
kind = "cli"
status = "active"
[[locations]]
scope = "user"
format = "yaml"
servers_key = "mcpServers"
entry = "array"
path.any = "~/x.yaml"
[locations.source]
kind = "observed"
read_on = 2026-10-16
quote = "words"
"#;
        let err = Rules::load([("x.toml", text)]).unwrap_err();
        assert!(err.to_string().contains("need name_key"), "{err}");
    }
}
