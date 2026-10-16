//! Context cost report and the `context-cost` finding, from an inventory of the fake home
//! and synthetic probe results.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mcproster_core::cost::{self, HOST_THRESHOLD, SERVER_THRESHOLD};
use mcproster_core::env::Os;
use mcproster_core::model::FindingKind;
use mcproster_core::probe::{ProbeResult, ToolInfo};
use mcproster_core::{inventory, Env, Rules};

fn copy_fixture(home: &Path) {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/home");
    let home_str = home.display().to_string();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    walk(&src, &mut files);
    for entry in files {
        let rel = entry.strip_prefix(&src).unwrap();
        let dest = home.join(rel);
        std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
        let text = std::fs::read_to_string(&entry).unwrap();
        std::fs::write(&dest, text.replace("__HOME__", &home_str)).unwrap();
    }
}

fn result(tokens_per_tool: &[u64]) -> ProbeResult {
    ProbeResult {
        at: "2026-10-16T12:00:00Z".into(),
        command: String::new(),
        ok: true,
        error: None,
        server_name: Some("x".into()),
        server_version: None,
        protocol_version: Some("2025-06-18".into()),
        tools: tokens_per_tool
            .iter()
            .enumerate()
            .map(|(i, &t)| ToolInfo {
                name: format!("tool_{i}"),
                description: None,
                tokens: t,
            })
            .collect(),
        schema_bytes: tokens_per_tool.iter().sum::<u64>() * 4,
        estimated_tokens: tokens_per_tool.iter().sum(),
        duration_ms: 5,
        stderr: vec![],
    }
}

#[test]
fn per_server_and_per_host_with_thresholds() {
    let tmp = tempfile::tempdir().unwrap();
    copy_fixture(tmp.path());
    let env = Env::fixture(tmp.path(), Os::Linux);
    let inv = inventory::run(&Rules::embedded(), &env);

    let mut probes = BTreeMap::new();
    // playwright: declared by Amp, Gemini CLI and VS Code — heavy.
    probes.insert(
        "@playwright/mcp".to_string(),
        result(&[3_000, 2_500, 1_000]),
    );
    // filesystem: light, declared by Claude Desktop and Cursor.
    probes.insert(
        "@modelcontextprotocol/server-filesystem".to_string(),
        result(&[400, 300]),
    );
    // apipriceindex: Claude Code + Claude Desktop, heavy enough to push Claude Desktop over.
    probes.insert(
        "apipriceindex-mcp".to_string(),
        result(&[9_000, 8_000, 4_000]),
    );
    // a failed probe
    let mut failed = result(&[]);
    failed.ok = false;
    failed.error = Some("no answer".into());
    probes.insert("ghost-mcp".to_string(), failed);

    let report = cost::report(&inv, &probes);

    let pw = report
        .servers
        .iter()
        .find(|s| s.identity == "@playwright/mcp")
        .unwrap();
    assert_eq!(pw.tokens, 6_500);
    assert_eq!(pw.heaviest[0], ("tool_0".to_string(), 3_000));
    assert!(pw.tokens > SERVER_THRESHOLD);

    let server_findings: Vec<&str> = report
        .findings
        .iter()
        .filter(|f| f.kind == FindingKind::ContextCost && f.message.starts_with('`'))
        .map(|f| f.message.as_str())
        .collect();
    assert_eq!(server_findings.len(), 2, "{server_findings:#?}"); // playwright, apipriceindex

    // Claude Desktop declares apipriceindex (21 000) + filesystem (700) + openai (not probed).
    let cd = report
        .hosts
        .iter()
        .find(|h| h.host == "claude-desktop")
        .unwrap();
    assert_eq!(cd.tokens, 21_700);
    assert_eq!(cd.probed, 2);
    assert_eq!(cd.unprobed, 1);
    assert!(cd.tokens > HOST_THRESHOLD);
    assert!(report
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::ContextCost && f.message.starts_with("Claude Desktop")));

    // Claude Code: apipriceindex probed, github remote, shared-db not probed; may defer.
    let cc = report
        .hosts
        .iter()
        .find(|h| h.host == "claude-code")
        .unwrap();
    assert_eq!(cc.tokens, 21_000);
    assert_eq!(cc.remote, 1);
    assert!(cc.may_defer);
    let cc_finding = report
        .findings
        .iter()
        .find(|f| f.message.starts_with("Claude Code"))
        .unwrap();
    assert!(cc_finding.message.contains("upper bound"));

    // Zed: ghost probed but failed.
    let zed = report.hosts.iter().find(|h| h.host == "zed").unwrap();
    assert_eq!(zed.failed, 1);
    assert_eq!(zed.tokens, 0);
}

#[test]
fn package_cache_findings() {
    let tmp = tempfile::tempdir().unwrap();
    copy_fixture(tmp.path());
    let env = Env::fixture(tmp.path(), Os::Linux);
    let inv = inventory::run(&Rules::embedded(), &env);
    let msgs: Vec<&str> = inv
        .findings
        .iter()
        .filter(|f| f.kind == FindingKind::PackageNotCached)
        .map(|f| f.message.as_str())
        .collect();
    // Cached in the fixture: @playwright/mcp, server-filesystem (npx), mcp-server-fetch (uv).
    assert!(
        msgs.iter().all(|m| !m.contains("@playwright/mcp")),
        "{msgs:#?}"
    );
    assert!(
        msgs.iter().all(|m| !m.contains("server-filesystem")),
        "{msgs:#?}"
    );
    assert!(
        msgs.iter().all(|m| !m.contains("mcp-server-fetch")),
        "{msgs:#?}"
    );
    // Not cached: context7 (Codex + OpenCode), server-memory (Continue), openai-mcp (uvx).
    assert!(msgs.iter().any(|m| m.contains("@upstash/context7-mcp")));
    assert!(msgs.iter().any(|m| m.contains("server-memory")));
    assert!(msgs.iter().any(|m| m.contains("openai-mcp")));
    // Disabled servers are not checked (aws-docs is disabled).
    assert!(msgs.iter().all(|m| !m.contains("awslabs")), "{msgs:#?}");
}
