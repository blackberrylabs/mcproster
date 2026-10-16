//! `mcproster` — read-only inventory of the MCP servers declared on this machine.
#![forbid(unsafe_code)]

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};
use mcproster_core::actions::{self, Action};
use mcproster_core::cost;
use mcproster_core::env::Os;
use mcproster_core::hosts::SourceKind;
use mcproster_core::model::{FindingKind, Transport};
use mcproster_core::probe;
use mcproster_core::{inventory, Env, Inventory, Rules};
use std::collections::BTreeMap;
use std::time::Duration;

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
    about = "Read-only inventory of the MCP servers declared on this machine, across assistant hosts. Reads configuration files, never writes to them, sends nothing."
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
    /// Start one declared server as its host would, ask it `initialize` and `tools/list`, stop it.
    /// On demand only; nothing else is called; remote servers are not contacted.
    Probe {
        /// Server name as declared, or its identity as shown by `list`.
        #[arg(required_unless_present = "all")]
        server: Option<String>,
        /// Probe every enabled stdio server, one launch each, after one confirmation.
        #[arg(long)]
        all: bool,
        /// Skip the confirmation (scripts).
        #[arg(long, short = 'y')]
        yes: bool,
        /// Time limit in seconds for each exchange.
        #[arg(long, default_value_t = 10)]
        timeout: u64,
    },
    /// What the probed servers' tool definitions weigh in the model's context, per server and per host.
    Context,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let rules = Rules::embedded();
    if let Command::Hosts = cli.command {
        return hosts(&rules, cli.json);
    }
    let env = match &cli.home {
        Some(h) => {
            // Another home, this machine's PATH: commands are looked up where they would run.
            let mut e = Env::fixture(h, Os::Linux);
            e.path.extend(Env::current().path);
            e
        }
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
        Command::Probe {
            server,
            all,
            yes,
            timeout,
        } => return probe_cmd(&rules, &inv, &env, server.as_deref(), all, yes, timeout),
        Command::Context => context(&inv, &env),
        Command::List => list(&inv, &env),
        Command::Findings => findings(&rules, &inv, &env),
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
    let probes = probe::load_results(&env.home);
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
        let probed = match probes.get(&s.identity) {
            Some(r) if r.ok => format!(
                "  — probed {}: {} tool(s), ~{} tokens{}",
                &r.at[..10],
                r.tools.len(),
                cost::thousands(r.estimated_tokens),
                r.server_version
                    .as_ref()
                    .map(|v| format!(", server {v}"))
                    .unwrap_or_default()
            ),
            Some(r) => format!(
                "  — probed {}: failed ({})",
                &r.at[..10],
                r.error.clone().unwrap_or_default()
            ),
            None => String::new(),
        };
        out!("{}{}{}", s.identity, mark, probed);
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

fn findings(rules: &Rules, inv: &Inventory, env: &Env) {
    let probes = probe::load_results(&env.home);
    let report = cost::report(inv, &probes);
    let all: Vec<&mcproster_core::model::Finding> =
        inv.findings.iter().chain(report.findings.iter()).collect();
    if all.is_empty() {
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
        FindingKind::ContextCost => "CONTEXT COST     ",
        FindingKind::PackageNotCached => "NOT IN CACHE     ",
        FindingKind::Disabled => "DISABLED         ",
    };
    let home = env.home.display().to_string();
    for f in all.iter() {
        out!("{} {}", label(f.kind), f.message.replace(&home, "~"));
        for a in actions::actions_for(rules, inv, f) {
            let line = match a {
                Action::HostCommand { host, command } => format!("{host}: run `{command}`"),
                Action::EditInHost { host } => {
                    format!("{host}: edit the entry in its own settings, then restart it")
                }
                Action::OpenFile => "open the file and remove or fix the entry".to_string(),
                Action::ReduceContext { host, may_defer } => format!(
                    "{host}: disable the servers you do not use in this host{}",
                    if may_defer {
                        "; or rely on its on-demand tool loading"
                    } else {
                        ""
                    }
                ),
            };
            out!("                  → {line}");
        }
    }
    out!();
    out!(
        "{} finding(s). MCProster changes nothing: use each host's own settings or command to act.",
        all.len()
    );
}

fn context(inv: &Inventory, env: &Env) {
    let probes = probe::load_results(&env.home);
    let report = cost::report(inv, &probes);
    if report.servers.is_empty() {
        out!("No server probed yet: run `mcproster probe --all` (or `probe <server>`), then `mcproster context`.");
        out!("Estimates: characters of each tool definition / 4; hosts serialise their own way (±20 %).");
        return;
    }
    out!("Per server (probed):");
    let mut servers = report.servers.clone();
    servers.sort_by_key(|s| std::cmp::Reverse(s.tokens));
    for s in &servers {
        let mark = if s.tokens > cost::SERVER_THRESHOLD {
            "  ▲"
        } else {
            ""
        };
        out!(
            "  {:<44} {:>3} tool(s)  ~{:>7} tokens{}",
            s.identity.replace(&env.home.display().to_string(), "~"),
            s.tools,
            cost::thousands(s.tokens),
            mark
        );
        for (name, t) in &s.heaviest {
            out!("      {:<40} ~{:>7}", name, cost::thousands(*t));
        }
    }
    out!();
    out!("Per host (enabled servers it declares):");
    for h in &report.hosts {
        let mut notes = Vec::new();
        if h.unprobed > 0 {
            notes.push(format!("at least — {} not probed", h.unprobed));
        }
        if h.failed > 0 {
            notes.push(format!("{} probe(s) failed", h.failed));
        }
        if h.remote > 0 {
            notes.push(format!("{} remote, never probed", h.remote));
        }
        if h.may_defer {
            notes.push("can load tools on demand: upper bound".to_string());
        }
        let mark = if h.tokens > cost::HOST_THRESHOLD {
            "  ▲"
        } else {
            ""
        };
        out!(
            "  {:<34} ~{:>7} tokens over {} probed server(s){}{}",
            h.host_name,
            cost::thousands(h.tokens),
            h.probed,
            if notes.is_empty() {
                String::new()
            } else {
                format!("  ({})", notes.join("; "))
            },
            mark
        );
    }
    out!();
    out!(
        "▲ above the thresholds (~{} per server, ~{} per host). Estimates: characters / 4, ±20 %. See docs/context-cost.md.",
        cost::thousands(cost::SERVER_THRESHOLD),
        cost::thousands(cost::HOST_THRESHOLD)
    );
}

fn files(inv: &Inventory, env: &Env) {
    for f in &inv.files {
        let status = match (&f.error, f.role.as_deref()) {
            (Some(e), _) => format!("unreadable: {e}"),
            (None, Some("registry")) => "versions and install dates".to_string(),
            (None, Some("settings")) => "enabled flag".to_string(),
            (None, _) => format!("{} declaration(s)", f.declarations),
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

#[allow(clippy::too_many_arguments)]
fn probe_cmd(
    rules: &Rules,
    inv: &Inventory,
    env: &Env,
    server: Option<&str>,
    all: bool,
    yes: bool,
    timeout: u64,
) -> Result<()> {
    // Declarations to probe: one server by name/identity, or every enabled stdio one.
    let selected: Vec<usize> = inv
        .declarations
        .iter()
        .enumerate()
        .filter(|(_, d)| match server {
            Some(s) => d.name == s || d.identity() == s,
            None => all && d.enabled,
        })
        .map(|(i, _)| i)
        .collect();
    if selected.is_empty() {
        match server {
            Some(s) => anyhow::bail!(
                "no declared server named `{s}` (names and identities come from `mcproster list`)"
            ),
            None => {
                out!("No enabled server to probe.");
                return Ok(());
            }
        }
    }
    let home = env.home.display().to_string();
    // One launch per distinct command + args.
    struct Launch {
        decl: usize,
        command: String,
        args: Vec<String>,
        env: BTreeMap<String, String>,
        cwd: Option<std::path::PathBuf>,
    }
    let mut launches: Vec<Launch> = Vec::new();
    let mut seen: BTreeMap<String, ()> = BTreeMap::new();
    for i in selected {
        let d = &inv.declarations[i];
        if d.transport != Transport::Stdio {
            if server.is_some() {
                out!(
                    "{} ({}): remote server, not probed — MCProster never opens a network connection.",
                    d.name,
                    d.host_names.join(" + ")
                );
            }
            continue;
        }
        let Some((command, args, child_env, cwd)) = inventory::raw_launch(rules, d) else {
            out!(
                "{} ({}): cannot read its launch command any more.",
                d.name,
                d.host_names.join(" + ")
            );
            continue;
        };
        let dir = d
            .file
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let command = command.replace("${__dirname}", &dir);
        let args: Vec<String> = args
            .iter()
            .map(|a| a.replace("${__dirname}", &dir))
            .collect();
        let key = format!("{command} {}", args.join(" "));
        if seen.insert(key, ()).is_some() {
            continue;
        }
        launches.push(Launch {
            decl: i,
            command,
            args,
            env: child_env,
            cwd,
        });
    }
    if launches.is_empty() {
        return Ok(());
    }
    // One confirmation listing every command (BRIEF D7), unless --yes.
    if all && !yes {
        out!(
            "About to start {} server(s) exactly as their hosts would, each for at most {timeout} s, sending only `initialize` and `tools/list`:",
            launches.len()
        );
        for l in &launches {
            let d = &inv.declarations[l.decl];
            out!(
                "  {:<28} {}",
                d.host_names.join(" + "),
                format!("{} {}", l.command, l.args.join(" "))
                    .trim_end()
                    .replace(&home, "~")
            );
        }
        out!("Continue? [y/N] ");
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        if !matches!(answer.trim(), "y" | "Y" | "yes" | "o" | "oui") {
            out!("Nothing started.");
            return Ok(());
        }
    }
    let mut results = probe::load_results(&env.home);
    let mut changed = false;
    for l in &launches {
        let d = &inv.declarations[l.decl];
        out!(
            "Starting `{}` as {} would, for at most {timeout} s. Only `initialize` and `tools/list` are sent.",
            format!("{} {}", l.command, l.args.join(" ")).trim_end().replace(&home, "~"),
            d.host_names.join(" + ")
        );
        let r = probe::probe_stdio(
            &l.command,
            &l.args,
            &l.env,
            l.cwd.as_deref(),
            &env.vars,
            Duration::from_secs(timeout),
        );
        if r.ok {
            out!(
                "  OK in {} ms — {}{}, protocol {} — {} tool(s), ~{} tokens of definitions",
                r.duration_ms,
                r.server_name
                    .clone()
                    .unwrap_or_else(|| "unnamed server".into()),
                r.server_version
                    .as_ref()
                    .map(|v| format!(" {v}"))
                    .unwrap_or_default(),
                r.protocol_version.clone().unwrap_or_else(|| "?".into()),
                r.tools.len(),
                cost::thousands(r.estimated_tokens)
            );
            if !all {
                for t in &r.tools {
                    out!(
                        "    {:<28} ~{:>6}  {}",
                        t.name,
                        cost::thousands(t.tokens),
                        t.description.clone().unwrap_or_default()
                    );
                }
            }
        } else {
            out!(
                "  FAILED after {} ms: {}",
                r.duration_ms,
                r.error.clone().unwrap_or_default()
            );
            for line in r.stderr.iter().take(5) {
                out!("    stderr: {line}");
            }
        }
        results.insert(d.identity(), r);
        changed = true;
    }
    if !changed {
        return Ok(());
    }
    match probe::save_results(&env.home, &results) {
        Ok(()) => out!(
            "Results kept in {} (names, versions, tool names and sizes; no configuration value, no tool description).",
            short(&probe::results_file(&env.home), &env.home)
        ),
        Err(e) => out!("Results not saved ({e})."),
    }
    if all {
        out!();
        context(inv, env);
    }
    Ok(())
}
