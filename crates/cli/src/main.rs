//! `mcproster` — read-only inventory of the MCP servers declared on this machine.
#![forbid(unsafe_code)]

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};
use mcproster_core::env::Os;
use mcproster_core::hosts::SourceKind;
use mcproster_core::model::{FindingKind, Transport};
use mcproster_core::{inventory, Env, Inventory, Rules};

/// Print a line; a closed pipe (`| head`) ends the program quietly instead of panicking.
macro_rules! out {
    ($($arg:tt)*) => {
        if writeln!(std::io::stdout().lock(), $($arg)*).is_err() {
            std::process::exit(0);
        }
    };
}

/// A path with the home directory shortened to `~`.
fn short(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

#[derive(Parser)]
#[command(
    name = "mcproster",
    version,
    about = "Read-only inventory of the MCP servers declared on this machine, across assistant hosts. Reads configuration files, writes nothing, sends nothing."
)]
struct Cli {
    /// Inventory another home directory (a test fixture, a mounted backup) instead of yours.
    #[arg(long, global = true)]
    home: Option<PathBuf>,
    /// Extra project root to search for per-project files (repeatable). Projects known to
    /// Claude Code are always searched.
    #[arg(long, global = true)]
    root: Vec<PathBuf>,
    /// Machine-readable output.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// The hosts MCProster knows, where each keeps its declarations, and how that was established.
    Hosts,
    /// Every server declared on this machine, with the hosts and files that declare it.
    List,
    /// Duplicates, plain-text secrets, missing commands, wide access, skipped confirmations, unreadable files.
    Findings,
    /// The files that were looked at.
    Files,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let rules = Rules::embedded();
    if let Command::Hosts = cli.command {
        return hosts(&rules, cli.json);
    }
    let env = match &cli.home {
        Some(h) => Env::fixture(h, Os::Linux),
        None => Env::current(),
    }
    .with_roots(cli.root.clone());
    let inv = inventory::run(&rules, &env);
    if cli.json {
        out!("{}", serde_json::to_string_pretty(&inv)?);
        return Ok(());
    }
    match cli.command {
        Command::Hosts => unreachable!(),
        Command::List => list(&inv, &env),
        Command::Findings => findings(&inv, &env),
        Command::Files => files(&inv, &env),
    }
    Ok(())
}

fn hosts(rules: &Rules, json: bool) -> Result<()> {
    if json {
        out!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({ "hosts": rules.hosts, "not_read": rules.not_read })
            )?
        );
        return Ok(());
    }
    for h in &rules.hosts {
        out!("{} ({})", h.name, h.kind);
        for l in &h.locations {
            let path = l
                .path
                .any
                .as_ref()
                .or(l.path.linux.as_ref())
                .map(|p| p.iter().collect::<Vec<_>>().join(" | "))
                .unwrap_or_default();
            let src = match l.source.kind {
                SourceKind::Documentation => "documentation",
                SourceKind::SourceCode => "source code",
                SourceKind::Observed => "observed",
            };
            let mut tags = Vec::new();
            if l.legacy {
                tags.push("legacy");
            }
            if let Some(r) = &l.role {
                tags.push(r);
            }
            let tags = if tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", tags.join(", "))
            };
            out!(
                "  {:<8} {}  ← {}{}",
                format!("{:?}", l.scope).to_lowercase(),
                path,
                src,
                tags
            );
            if let Some(u) = &l.source.url {
                out!("           {u} ({})", l.source.read_on);
            }
        }
    }
    out!();
    out!("Not read in this version:");
    for n in &rules.not_read {
        out!("  {}: {}", n.name, n.reason);
    }
    Ok(())
}

fn list(inv: &Inventory, env: &Env) {
    let hosts_with_files = inv.hosts.iter().filter(|h| h.files_found > 0).count();
    out!(
        "{} host(s) with configuration, {} declaration(s), {} server(s), {} finding(s)",
        hosts_with_files,
        inv.declarations.len(),
        inv.servers.len(),
        inv.findings.len()
    );
    out!();
    for s in &inv.servers {
        let n = s.declarations.len();
        let mark = if n > 1 {
            format!("  ×{n}")
        } else {
            String::new()
        };
        out!("{}{}", s.identity, mark);
        for &i in &s.declarations {
            let d = &inv.declarations[i];
            let transport = match d.transport {
                Transport::Stdio => "stdio",
                Transport::Http => "http",
                Transport::Sse => "sse",
                Transport::Ws => "ws",
                Transport::Unknown => "?",
            };
            let mut flags = Vec::new();
            if !d.enabled {
                flags.push("disabled".to_string());
            }
            if d.trusted {
                flags.push("confirmations skipped".to_string());
            }
            if d.legacy {
                flags.push("legacy".to_string());
            }
            if let Some(v) = &d.version {
                flags.push(format!("@{v}"));
            }
            if d.source_kind == SourceKind::Observed {
                flags.push("location observed, not documented".to_string());
            }
            let flags = if flags.is_empty() {
                String::new()
            } else {
                format!("  [{}]", flags.join(", "))
            };
            let line = d.line.map(|l| format!(":{l}")).unwrap_or_default();
            out!(
                "  {:<34} {:<6} {}{}{}",
                d.host_names.join(" + "),
                transport,
                short(&d.file, &env.home),
                line,
                flags
            );
            let home = env.home.display().to_string();
            let what = match (&d.command, &d.url) {
                (Some(c), _) => {
                    let args: Vec<&str> = d.args.iter().map(|a| a.shown.as_str()).collect();
                    format!("{c} {}", args.join(" "))
                        .trim_end()
                        .replace(&home, "~")
                }
                (None, Some(u)) => u.shown.clone(),
                (None, None) => String::from("(no command, no url)"),
            };
            out!("  {:<34} {}", format!("  as `{}`", d.name), what);
            for (k, v) in d.env.iter().chain(d.headers.iter()) {
                out!("  {:<34}   {k} = {}", "", v.shown);
            }
        }
    }
}

fn findings(inv: &Inventory, env: &Env) {
    if inv.findings.is_empty() {
        out!("No findings.");
        return;
    }
    let label = |k: FindingKind| match k {
        FindingKind::PlainSecret => "SECRET IN CLEAR  ",
        FindingKind::WideAccess => "WIDE ACCESS      ",
        FindingKind::ConfirmationsSkipped => "NO CONFIRMATION  ",
        FindingKind::CommandNotFound => "COMMAND MISSING  ",
        FindingKind::VersionMismatch => "VERSIONS DIFFER  ",
        FindingKind::Duplicate => "DECLARED TWICE+  ",
        FindingKind::Unreadable => "UNREADABLE       ",
        FindingKind::Disabled => "DISABLED         ",
    };
    let home = env.home.display().to_string();
    for f in &inv.findings {
        out!("{} {}", label(f.kind), f.message.replace(&home, "~"));
    }
    out!();
    out!(
        "{} finding(s). MCProster changes nothing: use each host's own settings or command to act.",
        inv.findings.len()
    );
}

fn files(inv: &Inventory, env: &Env) {
    for f in &inv.files {
        let status = match &f.error {
            Some(e) => format!("unreadable: {e}"),
            None => format!("{} declaration(s)", f.declarations),
        };
        let legacy = if f.legacy { " [legacy]" } else { "" };
        out!(
            "{:<34} {:<8} {}  — {}{}",
            f.host_name,
            format!("{:?}", f.scope).to_lowercase(),
            short(&f.file, &env.home),
            status,
            legacy
        );
    }
    if !inv.roots.is_empty() {
        out!();
        out!(
            "Project roots searched: {}",
            inv.roots
                .iter()
                .map(|r| short(r, &env.home))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}
