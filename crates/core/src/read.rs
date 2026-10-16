//! Readers: JSON, JSONC (comments and trailing commas), TOML and YAML, all turned into one
//! `serde_json::Value` tree; and `dig`, a dotted-path lookup with `*` wildcards that
//! reports the actual key matched by each `*`.

use std::path::Path;

use serde_json::Value;
use thiserror::Error;

use crate::hosts::Format;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("cannot read: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("invalid YAML: {0}")]
    Yaml(#[from] serde_yaml_ng::Error),
    #[error("{0}")]
    Other(String),
}

/// Read a file into a JSON-like tree. Read-only: the file is opened for reading only.
pub fn read_tree(path: &Path, format: Format) -> Result<(String, Value), ReadError> {
    let text = std::fs::read_to_string(path)?;
    let value = parse(&text, format)?;
    Ok((text, value))
}

pub fn parse(text: &str, format: Format) -> Result<Value, ReadError> {
    match format {
        Format::Json => Ok(serde_json::from_str(text)?),
        Format::Jsonc => Ok(serde_json::from_str(&strip_jsonc(text))?),
        Format::Toml => {
            let v: toml::Value = toml::from_str(text)?;
            serde_json::to_value(v).map_err(ReadError::Json)
        }
        Format::Yaml => {
            let v: serde_yaml_ng::Value = serde_yaml_ng::from_str(text)?;
            serde_json::to_value(v)
                .map_err(|e| ReadError::Other(format!("YAML with non-string keys: {e}")))
        }
    }
}

/// Remove `//` and `/* */` comments outside strings, then trailing commas before `}` / `]`.
pub fn strip_jsonc(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut in_str = false;
    let mut esc = false;
    while i < b.len() {
        let c = b[i] as char;
        if in_str {
            out.push(c);
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            i += 1;
        } else if text[i..].starts_with("//") {
            i = text[i..].find('\n').map(|j| i + j).unwrap_or(b.len());
        } else if text[i..].starts_with("/*") {
            i = text[i + 2..]
                .find("*/")
                .map(|j| i + 2 + j + 2)
                .unwrap_or(b.len());
        } else {
            out.push(c);
            i += 1;
        }
    }
    // Trailing commas: a comma followed only by whitespace then } or ].
    let mut cleaned = String::with_capacity(out.len());
    let chars: Vec<char> = out.chars().collect();
    let mut i = 0;
    let mut in_str = false;
    let mut esc = false;
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            cleaned.push(c);
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            cleaned.push(c);
            i += 1;
            continue;
        }
        if c == ',' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                i += 1;
                continue;
            }
        }
        cleaned.push(c);
        i += 1;
    }
    cleaned
}

/// Match a dotted path against a tree. `*` matches every key of an object at that level.
/// Returns, for each match, the keys the wildcards took (in order) and the node.
pub fn dig<'a>(root: &'a Value, path: &str) -> Vec<(Vec<String>, &'a Value)> {
    let parts = split_path(path);
    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
    let mut found = Vec::new();
    walk(root, &parts, Vec::new(), &mut found);
    found
}

/// Split a dotted path; `\.` is a literal dot inside a key (`amp\.mcpServers`).
pub fn split_path(path: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'.') => {
                cur.push('.');
                chars.next();
            }
            '.' => {
                if !cur.is_empty() {
                    parts.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        parts.push(cur);
    }
    parts
}

fn walk<'a>(
    node: &'a Value,
    parts: &[&str],
    wild: Vec<String>,
    out: &mut Vec<(Vec<String>, &'a Value)>,
) {
    match parts.split_first() {
        None => out.push((wild, node)),
        Some((head, rest)) => {
            let Some(obj) = node.as_object() else { return };
            if *head == "*" {
                for (k, v) in obj {
                    let mut w = wild.clone();
                    w.push(k.clone());
                    walk(v, rest, w, out);
                }
            } else if let Some(v) = obj.get(*head) {
                walk(v, rest, wild, out);
            }
        }
    }
}

/// A single value at a dotted path (no wildcards).
pub fn get<'a>(node: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = node;
    for p in split_path(path) {
        cur = cur.as_object()?.get(&p)?;
    }
    Some(cur)
}

/// 1-based line of the first occurrence of `needle` in `text`, if any.
pub fn line_of(text: &str, needle: &str) -> Option<usize> {
    let pos = text.find(needle)?;
    Some(text[..pos].matches('\n').count() + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn jsonc_comments_and_trailing_commas() {
        let t = r#"{
  // a comment with "quotes" inside
  "a": "http://x/y", /* block */
  "b": ["1", "2",],
}"#;
        let v = parse(t, Format::Jsonc).unwrap();
        assert_eq!(v["a"], "http://x/y");
        assert_eq!(v["b"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn toml_tables_become_objects() {
        let v = parse(
            "[mcp_servers.a]\ncommand = \"x\"\nargs = [\"-y\"]\n",
            Format::Toml,
        )
        .unwrap();
        assert_eq!(v["mcp_servers"]["a"]["command"], "x");
    }

    #[test]
    fn yaml_list_becomes_array() {
        let v = parse("mcpServers:\n  - name: a\n    command: x\n", Format::Yaml).unwrap();
        assert_eq!(v["mcpServers"][0]["name"], "a");
    }

    #[test]
    fn escaped_dot_is_a_literal_key_character() {
        let v = json!({"amp.mcpServers": {"a": {}}});
        assert_eq!(dig(&v, r"amp\.mcpServers").len(), 1);
        assert!(get(&v, r"amp\.mcpServers.a").is_some());
    }

    #[test]
    fn dig_with_wildcard_reports_keys() {
        let v = json!({"projects": {"/p/one": {"mcpServers": {"s": {}}}, "/p/two": {}}});
        let found = dig(&v, "projects.*.mcpServers");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, vec!["/p/one".to_string()]);
    }
}
