//! The inventory: resolve every rule location against an environment, read the files,
//! translate each declaration into the common model, group servers, derive findings.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::Serialize;
use serde_json::Value;

use bbl_hostrules::resolve::{project_roots, resolve};
use bbl_hostrules::Env;

use crate::hosts::{Entry, HostRule, Location, Rules};
use crate::model::{Declaration, FileStatus, Finding, FindingKind, Server, Transport};
use crate::read::{dig, get, line_of, read_tree};
use crate::secrets::{self, Masked, ValueKind};

#[derive(Debug, Clone, Serialize)]
pub struct HostStatus {
    pub id: String,
    pub name: String,
    pub kind: String,
    /// Application the host rule belongs to (several rules may share one).
    pub application: String,
    pub files_found: usize,
    pub declarations: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Inventory {
    pub hosts: Vec<HostStatus>,
    pub files: Vec<FileStatus>,
    pub declarations: Vec<Declaration>,
    pub servers: Vec<Server>,
    pub findings: Vec<Finding>,
    /// Project roots that were searched for per-project files.
    pub roots: Vec<PathBuf>,
}

/// Version / enabled metadata read from a registry or settings location, keyed by id.
#[derive(Default, Debug)]
struct Meta {
    version: Option<String>,
    enabled: Option<bool>,
}

/// Run the inventory. Read-only: every file is opened for reading only.
pub fn run(rules: &Rules, env: &Env) -> Inventory {
    let roots = project_roots(env);
    let mut files = Vec::new();
    let mut declarations: Vec<Declaration> = Vec::new();
    let mut findings = Vec::new();
    let mut hosts = Vec::new();
    let mut meta: BTreeMap<String, Meta> = BTreeMap::new();

    for host in &rules.hosts {
        let mut files_found = 0;
        let mut decls = 0;
        for (li, loc) in host.locations.iter().enumerate() {
            for bbl_hostrules::resolve::Resolved {
                path: file,
                project,
            } in resolve(loc, env, &roots)
            {
                if !file.is_file() {
                    continue;
                }
                files_found += 1;
                let (text, tree) = match read_tree(&file, loc.format) {
                    Ok(x) => x,
                    Err(e) => {
                        files.push(FileStatus {
                            host: host.id.clone(),
                            host_name: host.name.clone(),
                            scope: loc.scope,
                            file: file.clone(),
                            exists: true,
                            error: Some(e.to_string()),
                            declarations: 0,
                            source_kind: loc.source.kind,
                            legacy: loc.legacy,
                            role: loc.role.clone(),
                        });
                        findings.push(Finding {
                            kind: FindingKind::Unreadable,
                            message: format!("{} ({}): {e}", file.display(), host.name),
                            declarations: vec![],
                            file: Some(file.clone()),
                        });
                        continue;
                    }
                };
                let mut count = 0;
                match loc.role.as_deref() {
                    Some("registry") => collect_registry(loc, &tree, &mut meta),
                    Some("settings") => collect_settings(loc, &file, &tree, &mut meta),
                    _ => {
                        for d in extract(host, loc, li, &file, &text, &tree, project.as_deref()) {
                            count += 1;
                            merge(&mut declarations, d);
                        }
                    }
                }
                decls += count;
                files.push(FileStatus {
                    host: host.id.clone(),
                    host_name: host.name.clone(),
                    scope: loc.scope,
                    file,
                    exists: true,
                    error: None,
                    declarations: count,
                    source_kind: loc.source.kind,
                    legacy: loc.legacy,
                    role: loc.role.clone(),
                });
            }
        }
        hosts.push(HostStatus {
            id: host.id.clone(),
            name: host.name.clone(),
            kind: host.kind.clone(),
            application: host.application().to_string(),
            files_found,
            declarations: decls,
        });
    }

    // Hosts named through `shared_with` carry their id: give them their display name.
    for d in &mut declarations {
        for (i, hid) in d.hosts.clone().iter().enumerate() {
            if let Some(h) = rules.host(hid) {
                d.host_names[i] = h.name.clone();
            }
        }
    }

    // Registry / settings metadata: match on a path component equal to the id.
    for d in &mut declarations {
        for (id, m) in &meta {
            if d.file
                .components()
                .any(|c| c.as_os_str().to_string_lossy() == *id)
            {
                if d.version.is_none() {
                    d.version = m.version.clone();
                }
                if let Some(e) = m.enabled {
                    d.enabled = e;
                }
            }
        }
    }

    per_declaration_findings(&declarations, env, &mut findings);
    let servers = group(&declarations, &mut findings);
    findings.sort_by_key(|f| severity(f.kind));

    Inventory {
        hosts,
        files,
        declarations,
        servers,
        findings,
        roots,
    }
}

fn severity(k: FindingKind) -> u8 {
    match k {
        FindingKind::PlainSecret => 0,
        FindingKind::WideAccess => 1,
        FindingKind::ConfirmationsSkipped => 2,
        FindingKind::CommandNotFound => 3,
        FindingKind::VersionMismatch => 4,
        FindingKind::Duplicate => 5,
        FindingKind::Unreadable => 6,
        FindingKind::ContextCost => 7,
        FindingKind::PackageNotCached => 8,
        FindingKind::Disabled => 9,
    }
}

/// Is a runner package already in the local cache? `npx` keeps
/// `~/.npm/_npx/<hash>/node_modules/<package>`; `uv` keeps unpacked wheels under
/// `~/.cache/uv/archive-v0/<hash>/lib/python*/site-packages/<name>` (observed layouts, see
/// `docs/context-cost.md`). `None` when the runner is not npx/bunx/uvx/pipx.
pub fn package_cached(env: &Env, command: &str, package: &str) -> Option<bool> {
    let runner = Path::new(command)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    match runner.as_str() {
        "npx" | "bunx" | "pnpx" => {
            let pattern = env
                .home
                .join(".npm/_npx/*/node_modules")
                .join(package)
                .join("package.json");
            Some(
                bbl_hostrules::resolve::expand_glob(&pattern)
                    .into_iter()
                    .any(|p| p.is_file()),
            )
        }
        "uvx" | "pipx" => {
            let name = package.replace('-', "_").to_lowercase();
            let pattern = env
                .home
                .join(".cache/uv/archive-v0/*/lib/python*/site-packages")
                .join(&name);
            let pattern2 = env.home.join(".local/pipx/venvs").join(package);
            Some(
                bbl_hostrules::resolve::expand_glob(&pattern)
                    .into_iter()
                    .any(|p| p.is_dir())
                    || pattern2.is_dir(),
            )
        }
        _ => None,
    }
}

fn collect_registry(loc: &Location, tree: &Value, meta: &mut BTreeMap<String, Meta>) {
    for (_, node) in dig(tree, &loc.ext.servers_key) {
        let Some(obj) = node.as_object() else {
            continue;
        };
        for (id, spec) in obj {
            let m = meta.entry(id.clone()).or_default();
            if let Some(v) = first_str(spec, &loc.ext.mapped("version")) {
                m.version = Some(v);
            }
            if let Some(e) = first_bool(spec, &loc.ext.mapped("enabled")) {
                m.enabled = Some(e);
            }
        }
    }
}

fn collect_settings(loc: &Location, file: &Path, tree: &Value, meta: &mut BTreeMap<String, Meta>) {
    let Some(id) = file.file_stem().map(|s| s.to_string_lossy().to_string()) else {
        return;
    };
    let m = meta.entry(id).or_default();
    if let Some(e) = first_bool(tree, &loc.ext.mapped("enabled")) {
        m.enabled = Some(e);
    }
}

fn first_str(spec: &Value, candidates: &[&str]) -> Option<String> {
    candidates
        .iter()
        .find_map(|c| get(spec, c).and_then(Value::as_str).map(str::to_string))
}

fn first_bool(spec: &Value, candidates: &[&str]) -> Option<bool> {
    candidates
        .iter()
        .find_map(|c| get(spec, c).and_then(Value::as_bool))
}

fn first_node<'a>(spec: &'a Value, candidates: &[&str]) -> Option<&'a Value> {
    candidates.iter().find_map(|c| get(spec, c))
}

fn str_list(v: &Value) -> Vec<String> {
    match v {
        Value::Array(a) => a.iter().map(scalar).collect(),
        Value::String(s) => vec![s.clone()],
        _ => Vec::new(),
    }
}

fn scalar(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// The (name, project, spec) entries a location designates in a tree.
pub fn entries<'a>(
    loc: &Location,
    file: &Path,
    tree: &'a Value,
    project: Option<&Path>,
) -> Vec<(String, Option<PathBuf>, &'a Value)> {
    let mut out = Vec::new();
    for (wild, node) in dig(tree, &loc.ext.servers_key) {
        // The project path may be a key inside the file (Claude Code local scope).
        let project_in_file = loc
            .project_key
            .as_ref()
            .and_then(|_| wild.first())
            .map(PathBuf::from);
        let project = project_in_file.or_else(|| project.map(Path::to_path_buf));
        let pairs: Vec<(String, &Value)> = match loc.ext.entry {
            Entry::Object => match node.as_object() {
                Some(o) => o.iter().map(|(k, v)| (k.clone(), v)).collect(),
                None => Vec::new(),
            },
            Entry::Array => match node.as_array() {
                Some(a) => a
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let name = loc
                            .ext
                            .name_key
                            .as_ref()
                            .and_then(|k| get(v, k))
                            .and_then(Value::as_str)
                            .map(str::to_string)
                            .unwrap_or_else(|| format!("#{}", i + 1));
                        (name, v)
                    })
                    .collect(),
                None => Vec::new(),
            },
            Entry::Single => {
                let name = loc
                    .ext
                    .name_key
                    .as_ref()
                    .and_then(|k| get(tree, k))
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| {
                        file.parent()
                            .and_then(Path::file_name)
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default()
                    });
                vec![(name, node)]
            }
        };
        for (name, spec) in pairs {
            if !spec.is_object() {
                continue;
            }
            // Goose: built-in extensions are not declared MCP servers.
            if first_str(spec, &loc.ext.mapped("type")).as_deref() == Some("builtin") {
                continue;
            }
            out.push((name, project.clone(), spec));
        }
    }
    out
}

/// Translate every declaration of one file.
fn extract(
    host: &HostRule,
    loc: &Location,
    li: usize,
    file: &Path,
    text: &str,
    tree: &Value,
    project: Option<&Path>,
) -> Vec<Declaration> {
    entries(loc, file, tree, project)
        .into_iter()
        .map(|(name, project, spec)| declaration(host, loc, li, file, text, &name, spec, project))
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn declaration(
    host: &HostRule,
    loc: &Location,
    li: usize,
    file: &Path,
    text: &str,
    name: &str,
    spec: &Value,
    project: Option<PathBuf>,
) -> Declaration {
    // command + args
    let mut command = first_str(spec, &loc.ext.mapped("command"));
    let mut args: Vec<String> = first_node(spec, &loc.ext.mapped("args"))
        .map(str_list)
        .unwrap_or_default();
    if loc.ext.command_is_array {
        if let Some(arr) = first_node(spec, &loc.ext.mapped("command")).and_then(Value::as_array) {
            let mut items: Vec<String> = arr.iter().map(scalar).collect();
            if !items.is_empty() {
                command = Some(items.remove(0));
                args = items;
            }
        }
    }
    let url_raw = first_str(spec, &loc.ext.mapped("url"));
    let type_raw = first_str(spec, &loc.ext.mapped("type")).unwrap_or_default();
    let transport = Transport::from_type(&type_raw, url_raw.is_some());

    let env_pairs = kv_pairs(first_node(spec, &loc.ext.mapped("env")));
    let header_pairs = kv_pairs(first_node(spec, &loc.ext.mapped("headers")));

    let mut env_keys: Vec<String> = first_node(spec, &loc.ext.mapped("env_keys"))
        .map(str_list)
        .unwrap_or_default();
    if let Some(b) = first_str(spec, &loc.ext.mapped("bearer_env")) {
        env_keys.push(b);
    }
    if let Some(Value::Object(o)) = first_node(spec, &loc.ext.mapped("env_headers")) {
        env_keys.extend(o.values().map(scalar));
    }

    let enabled = match (
        first_bool(spec, &loc.ext.mapped("enabled")),
        first_bool(spec, &loc.ext.mapped("disabled")),
    ) {
        (Some(e), _) => e,
        (None, Some(d)) => !d,
        (None, None) => true,
    };
    let trusted = first_bool(spec, &loc.ext.mapped("trust")).unwrap_or(false)
        || ["autoApprove", "alwaysAllow"].iter().any(|k| {
            spec.get(*k)
                .and_then(Value::as_array)
                .map(|a| !a.is_empty())
                .unwrap_or(false)
        });

    let (package, version) = package_of(command.as_deref(), &args);
    let home_like = ["~", "~/"];
    let path_args: Vec<PathBuf> = args
        .iter()
        .filter(|a| {
            a.starts_with('/')
                || home_like.contains(&a.as_str())
                || a.starts_with("~/")
                || (a.len() >= 3
                    && a.as_bytes()[1] == b':'
                    && (a.as_bytes()[2] == b'\\' || a.as_bytes()[2] == b'/'))
        })
        .map(PathBuf::from)
        .collect();

    let line = line_of(text, &format!("\"{name}\""))
        .or_else(|| line_of(text, &format!("[mcp_servers.{name}]")))
        .or_else(|| line_of(text, &format!("{name}:")))
        .or_else(|| line_of(text, name));

    let mut hosts = vec![host.id.clone()];
    let mut host_names = vec![host.name.clone()];
    for s in &loc.shared_with {
        hosts.push(s.clone());
        host_names.push(s.clone());
    }

    Declaration {
        hosts,
        host_names,
        scope: loc.scope,
        file: file.to_path_buf(),
        line,
        project,
        name: name.to_string(),
        transport,
        command,
        args: args.iter().map(|a| secrets::scrub_text(a)).collect(),
        url: url_raw.as_deref().map(secrets::scrub_text),
        env: env_pairs,
        headers: header_pairs,
        env_keys,
        enabled,
        trusted,
        legacy: loc.legacy,
        package,
        version,
        source_kind: loc.source.kind,
        path_args,
        origin: (host.id.clone(), li),
    }
}

/// What is needed to start a server: command, args, environment (clear), working dir.
pub type Launch = (
    String,
    Vec<String>,
    BTreeMap<String, String>,
    Option<PathBuf>,
);

/// Re-read the raw spec of a declaration (command, args, env in clear) for the probe. The
/// values are returned to the caller and never stored.
pub fn raw_launch(rules: &Rules, d: &Declaration) -> Option<Launch> {
    let host = rules.host(&d.origin.0)?;
    let loc = host.locations.get(d.origin.1)?;
    let (_, tree) = read_tree(&d.file, loc.format).ok()?;
    let project = d.project.as_deref();
    let (_, _, spec) = entries(loc, &d.file, &tree, project)
        .into_iter()
        .find(|(name, p, _)| {
            *name == d.name && (loc.project_key.is_none() || p.as_deref() == project)
        })?;
    let mut command = first_str(spec, &loc.ext.mapped("command"));
    let mut args: Vec<String> = first_node(spec, &loc.ext.mapped("args"))
        .map(str_list)
        .unwrap_or_default();
    if loc.ext.command_is_array {
        if let Some(arr) = first_node(spec, &loc.ext.mapped("command")).and_then(Value::as_array) {
            let mut items: Vec<String> = arr.iter().map(scalar).collect();
            if !items.is_empty() {
                command = Some(items.remove(0));
                args = items;
            }
        }
    }
    let env: BTreeMap<String, String> = match first_node(spec, &loc.ext.mapped("env")) {
        Some(Value::Object(o)) => o.iter().map(|(k, v)| (k.clone(), scalar(v))).collect(),
        _ => BTreeMap::new(),
    };
    let cwd = first_str(spec, &loc.ext.mapped("cwd"))
        .map(PathBuf::from)
        .or_else(|| d.project.clone());
    Some((command?, args, env, cwd))
}

fn kv_pairs(node: Option<&Value>) -> Vec<(String, Masked)> {
    match node {
        Some(Value::Object(o)) => o
            .iter()
            .map(|(k, v)| (k.clone(), secrets::classify(k, &scalar(v))))
            .collect(),
        _ => Vec::new(),
    }
}

/// Package and version behind a runner command (`npx -y pkg@1.2`, `uvx pkg`, `pipx run pkg`,
/// `bunx pkg`, `pnpm dlx pkg`, `uvx --from pkg cmd`).
fn package_of(command: Option<&str>, args: &[String]) -> (Option<String>, Option<String>) {
    let Some(cmd) = command else {
        return (None, None);
    };
    let base = Path::new(cmd)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let runner = matches!(
        base.as_str(),
        "npx" | "bunx" | "uvx" | "pipx" | "pnpm" | "yarn" | "pnpx"
    );
    if !runner {
        return (None, None);
    }
    let mut candidate: Option<&str> = None;
    let mut iter = args.iter().map(String::as_str).peekable();
    while let Some(a) = iter.next() {
        match a {
            "-y" | "--yes" | "-q" | "--quiet" | "run" | "dlx" | "exec" => continue,
            "--from" => {
                candidate = iter.next();
                break;
            }
            "-p" | "--package" => {
                candidate = iter.next();
                break;
            }
            s if s.starts_with('-') => continue,
            s => {
                candidate = Some(s);
                break;
            }
        }
    }
    let Some(c) = candidate else {
        return (None, None);
    };
    let re = Regex::new(r"^(@?[^@]+)(?:@(.+))?$").unwrap();
    match re.captures(c) {
        Some(caps) => (
            Some(caps.get(1).unwrap().as_str().to_string()),
            caps.get(2).map(|m| m.as_str().to_string()),
        ),
        None => (Some(c.to_string()), None),
    }
}

/// The same server declared in the same file for another host (a shared `.mcp.json`) is
/// one declaration with several hosts, not two declarations.
fn merge(declarations: &mut Vec<Declaration>, d: Declaration) {
    if let Some(existing) = declarations
        .iter_mut()
        .find(|e| e.file == d.file && e.name == d.name)
    {
        for (h, n) in d.hosts.iter().zip(d.host_names.iter()) {
            if !existing.hosts.contains(h) {
                existing.hosts.push(h.clone());
                existing.host_names.push(n.clone());
            }
        }
        return;
    }
    declarations.push(d);
}

fn per_declaration_findings(declarations: &[Declaration], env: &Env, findings: &mut Vec<Finding>) {
    for (i, d) in declarations.iter().enumerate() {
        let host = d.host_names.join(" + ");
        let where_ = format!("{} ({}, {})", d.name, host, d.file.display());
        // Secrets in clear.
        let mut secret_fields: Vec<String> = Vec::new();
        for (k, v) in d.env.iter().chain(d.headers.iter()) {
            if v.kind == ValueKind::Secret {
                secret_fields.push(k.clone());
            }
        }
        if d.args.iter().any(|a| a.kind == ValueKind::Secret) {
            secret_fields.push("args".into());
        }
        if d.url
            .as_ref()
            .map(|u| u.kind == ValueKind::Secret)
            .unwrap_or(false)
        {
            secret_fields.push("url".into());
        }
        if !secret_fields.is_empty() {
            findings.push(Finding {
                kind: FindingKind::PlainSecret,
                message: format!("{where_}: {} written in clear", secret_fields.join(", ")),
                declarations: vec![i],
                file: Some(d.file.clone()),
            });
        }
        // Wide filesystem access.
        let home = env.home_str();
        let wide: Vec<String> = d
            .path_args
            .iter()
            .filter(|p| {
                let s = p.to_string_lossy();
                s == "/"
                    || s == "/home"
                    || s == "~"
                    || s == "~/"
                    || s == home
                    || s == format!("{home}/")
                    || s == "/Users"
                    || s.len() == 3 && s.ends_with(":\\")
            })
            .map(|p| p.display().to_string())
            .collect();
        if !wide.is_empty() {
            findings.push(Finding {
                kind: FindingKind::WideAccess,
                message: format!("{where_}: access to {}", wide.join(", ")),
                declarations: vec![i],
                file: Some(d.file.clone()),
            });
        }
        if d.trusted {
            findings.push(Finding {
                kind: FindingKind::ConfirmationsSkipped,
                message: format!(
                    "{where_}: tool confirmations skipped (trust / autoApprove / alwaysAllow)"
                ),
                declarations: vec![i],
                file: Some(d.file.clone()),
            });
        }
        if !d.enabled {
            findings.push(Finding {
                kind: FindingKind::Disabled,
                message: format!("{where_}: declared but disabled"),
                declarations: vec![i],
                file: Some(d.file.clone()),
            });
        }
        // Runner package in the local cache? Informational: the command works but downloads.
        if d.transport == Transport::Stdio && d.enabled {
            if let (Some(cmd), Some(pkg)) = (&d.command, &d.package) {
                if package_cached(env, cmd, pkg) == Some(false) {
                    findings.push(Finding {
                        kind: FindingKind::PackageNotCached,
                        message: format!("{where_}: `{pkg}` is not in the local cache — it will be downloaded at first start"),
                        declarations: vec![i],
                        file: Some(d.file.clone()),
                    });
                }
            }
        }
        // Command present on this machine? Only for stdio, only when the command is literal.
        if d.transport == Transport::Stdio && d.enabled {
            if let Some(cmd) = &d.command {
                let dir = d
                    .file
                    .parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                let literal = cmd.replace("${__dirname}", &dir);
                if !secrets::contains_reference(&literal) && env.which(&literal).is_none() {
                    findings.push(Finding {
                        kind: FindingKind::CommandNotFound,
                        message: format!("{where_}: command `{cmd}` not found"),
                        declarations: vec![i],
                        file: Some(d.file.clone()),
                    });
                }
            }
        }
    }
}

/// Group declarations into servers; a server declared in more than one file is a finding.
fn group(declarations: &[Declaration], findings: &mut Vec<Finding>) -> Vec<Server> {
    let mut by_id: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, d) in declarations.iter().enumerate() {
        by_id.entry(d.identity()).or_default().push(i);
    }
    let mut servers = Vec::new();
    for (identity, idx) in by_id {
        let mut versions: Vec<String> = idx
            .iter()
            .filter_map(|&i| declarations[i].version.clone())
            .collect();
        versions.sort();
        versions.dedup();
        let files: BTreeSet<&Path> = idx
            .iter()
            .map(|&i| declarations[i].file.as_path())
            .collect();
        if files.len() > 1 {
            let places: Vec<String> = idx
                .iter()
                .map(|&i| {
                    let d = &declarations[i];
                    match &d.version {
                        Some(v) => format!("{} ({v})", d.host_names.join(" + ")),
                        None => d.host_names.join(" + "),
                    }
                })
                .collect();
            findings.push(Finding {
                kind: FindingKind::Duplicate,
                message: format!(
                    "`{identity}` declared {} times: {}",
                    idx.len(),
                    places.join(", ")
                ),
                declarations: idx.clone(),
                file: None,
            });
            if versions.len() > 1 {
                findings.push(Finding {
                    kind: FindingKind::VersionMismatch,
                    message: format!("`{identity}`: versions differ ({})", versions.join(" / ")),
                    declarations: idx.clone(),
                    file: None,
                });
            }
        }
        servers.push(Server {
            identity,
            declarations: idx,
            versions,
        });
    }
    servers
}

impl Inventory {
    /// Every string that can be shown, concatenated: the test "no clear secret anywhere"
    /// runs on this.
    pub fn all_text(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn count(&self, kind: FindingKind) -> usize {
        self.findings.iter().filter(|f| f.kind == kind).count()
    }
}
