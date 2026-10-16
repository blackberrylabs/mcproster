//! The `bbl-report/1` view of an inventory, for the Blackberrylabs control tower.
//!
//! Nothing here is new: every figure, finding, path and action is already shown by the CLI
//! or the app, secrets masked upstream. The mapping of MCProster's findings onto the four
//! common severities is in [`severity_of`] and in the CHANGELOG.

use std::collections::BTreeSet;
use std::path::Path;

use bbl_report::{Action, ActionKind, Evidence, Finding, Module, Open, Report, Severity, Subject, SubjectKind, Summary, Tone};

use crate::actions::{self, Action as Act};
use crate::cost;
use crate::hosts::Rules;
use crate::model::FindingKind;
use crate::probe::ProbeResult;
use crate::{Env, Inventory};

/// Binary name of the app, as packaged.
pub const APP: &str = "mcproster-app";
/// Binary name of the command line, as packaged.
pub const CLI: &str = "mcproster";

/// Who we are, for the report header.
pub fn module() -> Module {
    Module {
        id: "mcproster".into(),
        name: "MCProster".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        verb: "inventory".into(),
        tagline: "Every MCP server declared on your machine, in one list".into(),
        app: Some(APP.into()),
        cli: Some(CLI.into()),
        home: "https://github.com/blackberrylabs/mcproster".into(),
    }
}

/// MCProster's findings on the common scale.
pub fn severity_of(kind: FindingKind) -> Severity {
    match kind {
        // A credential anyone who reads the file can use.
        FindingKind::PlainSecret => Severity::ActNow,
        // The server reaches wide, or acts without a confirmation.
        FindingKind::WideAccess | FindingKind::ConfirmationsSkipped => Severity::Review,
        // Worth tidying: declared twice, versions that differ, a command that is not there,
        // tool definitions that weigh on every request.
        FindingKind::Duplicate | FindingKind::VersionMismatch | FindingKind::CommandNotFound | FindingKind::ContextCost => {
            Severity::GoodToKnow
        }
        // Nothing to do, good to know it is there.
        FindingKind::Unreadable | FindingKind::Disabled | FindingKind::PackageNotCached => Severity::Info,
    }
}

/// The slug of a kind, as it appears in the report and in finding ids (the same words as
/// the `--json` output).
pub fn kind_slug(kind: FindingKind) -> &'static str {
    match kind {
        FindingKind::Duplicate => "duplicate",
        FindingKind::VersionMismatch => "version-mismatch",
        FindingKind::PlainSecret => "plain-secret",
        FindingKind::CommandNotFound => "command-not-found",
        FindingKind::WideAccess => "wide-access",
        FindingKind::ConfirmationsSkipped => "confirmations-skipped",
        FindingKind::Unreadable => "unreadable",
        FindingKind::Disabled => "disabled",
        FindingKind::ContextCost => "context-cost",
        FindingKind::PackageNotCached => "package-not-cached",
    }
}

/// Build the report of an inventory. `probes` are the saved probe results, read by the
/// caller as the CLI and the app do; `duration_ms` is the time the inventory took.
pub fn build(
    inv: &Inventory,
    rules: &Rules,
    env: &Env,
    probes: &std::collections::BTreeMap<String, ProbeResult>,
    duration_ms: u64,
) -> Report {
    let mut report = Report::new(module());
    report.duration_ms = duration_ms;

    let cost_report = cost::report(inv, probes);
    let with_config = inv.hosts.iter().filter(|h| h.files_found > 0).count();
    // The app's "Findings" card leaves disabled servers out, like its list by default.
    let n_findings = inv.findings.iter().filter(|f| f.kind != FindingKind::Disabled).count() + cost_report.findings.len();
    let secrets = inv.count(FindingKind::PlainSecret);
    let zero_is_good = |n: usize| if n == 0 { Tone::Good } else { Tone::Alert };
    report.summary = vec![
        Summary { key: "servers".into(), label: "Servers".into(), value: inv.servers.len().to_string(), tone: Tone::Neutral },
        Summary { key: "hosts".into(), label: "Hosts with a configuration".into(), value: with_config.to_string(), tone: Tone::Neutral },
        Summary { key: "findings".into(), label: "Findings".into(), value: n_findings.to_string(), tone: zero_is_good(n_findings) },
        Summary { key: "secrets".into(), label: "Secrets in clear".into(), value: secrets.to_string(), tone: zero_is_good(secrets) },
    ];

    let mut ids = BTreeSet::new();
    for f in inv.findings.iter().chain(cost_report.findings.iter()) {
        let decls: Vec<&crate::model::Declaration> = f.declarations.iter().filter_map(|&i| inv.declarations.get(i)).collect();
        let identity = decls.first().map(|d| d.identity());
        let (subject, third) = match (&identity, &f.file) {
            (Some(id), _) => (Subject { kind: SubjectKind::Server, label: short_text(id, env) }, slug(&short_text(id, env))),
            (None, Some(p)) => (Subject { kind: SubjectKind::File, label: short(p, env) }, slug(&short(p, env))),
            (None, None) => (Subject { kind: SubjectKind::Other, label: f.message.clone() }, slug(&f.message)),
        };
        let id = unique(&mut ids, format!("{}/{third}", kind_slug(f.kind)));

        // The message names the file in full; the title does without it, the evidence has it.
        let mut title = f.message.clone();
        if let Some(p) = &f.file {
            title = title.replace(&format!(", {}", p.display()), "");
        }
        let title = short_text(&title, env);

        let mut evidence = Vec::new();
        let mut files = BTreeSet::new();
        for d in &decls {
            let hosts = d.host_names.join(", ");
            let at = match d.line {
                Some(l) => format!("{}:{l}", short(&d.file, env)),
                None => short(&d.file, env),
            };
            evidence.push(Evidence {
                label: format!("{at} — `{}` ({hosts})", d.name),
                path: Some(d.file.display().to_string()),
                line: d.line.map(|l| l as u32),
            });
            files.insert(d.file.clone());
        }
        if let (Some(p), true) = (&f.file, decls.is_empty()) {
            evidence.push(Evidence { label: short(p, env), path: Some(p.display().to_string()), line: None });
            files.insert(p.clone());
        }

        let mut acts = Vec::new();
        for a in actions::actions_for(rules, inv, f) {
            match a {
                Act::HostCommand { host, command } => {
                    acts.push(Action { kind: ActionKind::Command, label: format!("{host}: run"), target: Some(command) })
                }
                Act::EditInHost { host } => acts.push(Action {
                    kind: ActionKind::Hint,
                    label: format!("{host}: edit the entry in its own settings, then restart it"),
                    target: None,
                }),
                Act::OpenFile => {
                    for p in &files {
                        acts.push(Action { kind: ActionKind::OpenFile, label: format!("Open {}", short(p, env)), target: Some(p.display().to_string()) });
                    }
                }
                Act::ReduceContext { host, may_defer } => acts.push(Action {
                    kind: ActionKind::Hint,
                    label: format!(
                        "{host}: disable the servers you do not use in this host{}",
                        if may_defer { "; or rely on its on-demand tool loading" } else { "" }
                    ),
                    target: None,
                }),
            }
        }
        acts.dedup();

        report.findings.push(Finding {
            id,
            severity: severity_of(f.kind),
            kind: kind_slug(f.kind).into(),
            title,
            subject,
            detail: String::new(),
            evidence,
            actions: acts,
            open: identity.map(|id| Open { app: APP.into(), args: vec!["--select".into(), id] }),
        });
    }

    report.notes.push(format!("{} declaration(s) in {} file(s)", inv.declarations.len(), inv.files.iter().filter(|f| f.exists).count()));
    if !inv.roots.is_empty() {
        report.notes.push(format!("{} project root(s) looked at", inv.roots.len()));
    }
    if !probes.is_empty() {
        report.notes.push(format!("{} saved probe result(s)", probes.len()));
    }
    for f in inv.files.iter().filter(|f| f.error.is_some()) {
        report.errors.push(format!("{}: {}", short(&f.file, env), f.error.as_deref().unwrap_or_default()));
    }
    report
}

/// `~` for the home directory, as the CLI and the app show paths.
pub fn short(p: &Path, env: &Env) -> String {
    short_text(&p.display().to_string(), env)
}

fn short_text(s: &str, env: &Env) -> String {
    let home = env.home.display().to_string();
    if home.is_empty() {
        return s.to_string();
    }
    s.replace(&home, "~")
}

/// Lowercase ASCII letters, digits and `-`, one dash between words.
fn slug(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

fn unique(ids: &mut BTreeSet<String>, id: String) -> String {
    let mut candidate = id.clone();
    let mut n = 2;
    while !ids.insert(candidate.clone()) {
        candidate = format!("{id}#{n}");
        n += 1;
    }
    candidate
}
