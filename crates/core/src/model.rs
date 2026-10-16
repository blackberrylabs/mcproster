//! The common model every host is translated into.

use std::path::PathBuf;

use serde::Serialize;

use crate::hosts::{Scope, SourceKind};
use crate::secrets::Masked;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    Stdio,
    Http,
    Sse,
    Ws,
    Unknown,
}

impl Transport {
    pub fn from_type(t: &str, has_url: bool) -> Transport {
        match t.to_ascii_lowercase().as_str() {
            "stdio" | "local" => Transport::Stdio,
            "http" | "streamable-http" | "streamablehttp" | "streamable_http" | "remote" => {
                Transport::Http
            }
            "sse" => Transport::Sse,
            "ws" | "websocket" => Transport::Ws,
            "" => {
                if has_url {
                    Transport::Http
                } else {
                    Transport::Stdio
                }
            }
            _ => Transport::Unknown,
        }
    }
}

/// One server declared in one file for one (or several, when the file is shared) host(s).
#[derive(Debug, Clone, Serialize)]
pub struct Declaration {
    pub hosts: Vec<String>,
    pub host_names: Vec<String>,
    pub scope: Scope,
    pub file: PathBuf,
    pub line: Option<usize>,
    /// Project root, for per-project declarations.
    pub project: Option<PathBuf>,
    pub name: String,
    pub transport: Transport,
    pub command: Option<String>,
    pub args: Vec<Masked>,
    pub url: Option<Masked>,
    pub env: Vec<(String, Masked)>,
    pub headers: Vec<(String, Masked)>,
    /// Environment variable names the host reads for the server (Codex, Goose).
    pub env_keys: Vec<String>,
    /// `false` when the host marks the server disabled.
    pub enabled: bool,
    /// Confirmations skipped: `trust`, `autoApprove`, `alwaysAllow`.
    pub trusted: bool,
    pub legacy: bool,
    /// Package behind `npx` / `uvx` / `pipx run` / `bunx`, without version.
    pub package: Option<String>,
    pub version: Option<String>,
    /// How the location was established.
    pub source_kind: SourceKind,
    /// Local filesystem paths given as arguments.
    pub path_args: Vec<PathBuf>,
    /// Rule and location this came from (`hosts[0]`, index in its `locations`), so the
    /// probe can re-read the raw spec. Not shown.
    #[serde(skip)]
    pub origin: (String, usize),
}

impl Declaration {
    /// The key servers are grouped by: URL, package, or command file name.
    pub fn identity(&self) -> String {
        if let Some(u) = &self.url {
            return u.shown.trim_end_matches('/').to_string();
        }
        if let Some(p) = &self.package {
            return p.clone();
        }
        match &self.command {
            Some(c) => {
                let name = std::path::Path::new(c)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| c.clone());
                // A generic interpreter is not an identity: add the first argument.
                if matches!(
                    name.as_str(),
                    "node" | "python" | "python3" | "bun" | "deno" | "sh" | "bash"
                ) {
                    match self.args.first() {
                        Some(a) => format!("{name} {}", a.shown),
                        None => name,
                    }
                } else {
                    name
                }
            }
            None => self.name.clone(),
        }
    }
}

/// Declarations of the same server, across hosts and files.
#[derive(Debug, Clone, Serialize)]
pub struct Server {
    pub identity: String,
    pub declarations: Vec<usize>,
    pub versions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingKind {
    Duplicate,
    VersionMismatch,
    PlainSecret,
    CommandNotFound,
    WideAccess,
    ConfirmationsSkipped,
    Unreadable,
    Disabled,
    /// A probed server, or a host's probed servers together, weigh more than the thresholds
    /// in `docs/context-cost.md`.
    ContextCost,
    /// `npx` / `uvx` package not found in the local cache: the command will work but will
    /// download the package first.
    PackageNotCached,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub kind: FindingKind,
    pub message: String,
    pub declarations: Vec<usize>,
    pub file: Option<PathBuf>,
}

/// One file the inventory looked at.
#[derive(Debug, Clone, Serialize)]
pub struct FileStatus {
    pub host: String,
    pub host_name: String,
    pub scope: Scope,
    pub file: PathBuf,
    pub exists: bool,
    pub error: Option<String>,
    pub declarations: usize,
    pub source_kind: SourceKind,
    pub legacy: bool,
    /// `registry` / `settings` for metadata files, `None` for declaration files.
    pub role: Option<String>,
}
