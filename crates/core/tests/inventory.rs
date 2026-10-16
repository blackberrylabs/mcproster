//! The inventory on a fake home (`tests/fixtures/home`) that represents every host read in
//! v0, copied to a temporary directory with `__HOME__` replaced by its real path.

use std::path::{Path, PathBuf};

use mcproster_core::env::Os;
use mcproster_core::model::FindingKind;
use mcproster_core::{hosts, inventory, Env};

fn copy_fixture(home: &Path) {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/home");
    let home_str = home.display().to_string();
    for entry in walkdir(&src) {
        let rel = entry.strip_prefix(&src).unwrap();
        let dest = home.join(rel);
        std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
        let text = std::fs::read_to_string(&entry).unwrap();
        std::fs::write(&dest, text.replace("__HOME__", &home_str)).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&entry).unwrap().permissions().mode();
            std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(mode)).unwrap();
        }
    }
}

fn walkdir(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(walkdir(&p));
        } else {
            out.push(p);
        }
    }
    out
}

fn run() -> (tempfile::TempDir, inventory::Inventory) {
    let tmp = tempfile::tempdir().unwrap();
    copy_fixture(tmp.path());
    let env = Env::fixture(tmp.path(), Os::Linux);
    let inv = inventory::run(&hosts::embedded(), &env);
    (tmp, inv)
}

#[test]
fn every_declaration_of_the_fake_home_is_found() {
    let (_tmp, inv) = run();
    let names: Vec<String> = inv
        .declarations
        .iter()
        .map(|d| format!("{}:{}", d.hosts.join("+"), d.name))
        .collect();
    assert_eq!(inv.declarations.len(), 26, "{names:#?}");
    let with_files = inv.hosts.iter().filter(|h| h.files_found > 0).count();
    assert!(with_files >= 18, "{with_files} hosts with files");
    // Per-project roots came from ~/.claude.json `projects`.
    assert_eq!(inv.roots.len(), 2, "{:?}", inv.roots);
}

#[test]
fn a_shared_mcp_json_is_one_declaration_for_two_hosts() {
    let (_tmp, inv) = run();
    let shared = inv
        .declarations
        .iter()
        .find(|d| d.name == "shared-db")
        .unwrap();
    assert_eq!(
        shared.hosts,
        vec!["claude-code".to_string(), "vscode".to_string()]
    );
    assert_eq!(shared.host_names[1], "Visual Studio Code (Copilot)");
    assert!(shared.file.ends_with("projects/alpha/.mcp.json"));
    assert!(shared.project.as_ref().unwrap().ends_with("projects/alpha"));
    // Not counted as a duplicate.
    assert!(!inv
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Duplicate && f.message.contains("server.js")));
}

#[test]
fn duplicates_and_version_mismatches() {
    let (_tmp, inv) = run();
    let dups: Vec<&str> = inv
        .findings
        .iter()
        .filter(|f| f.kind == FindingKind::Duplicate)
        .map(|f| f.message.as_str())
        .collect();
    assert_eq!(dups.len(), 6, "{dups:#?}");
    assert!(dups
        .iter()
        .any(|m| m.contains("apipriceindex-mcp") && m.contains("2 times")));
    assert!(dups
        .iter()
        .any(|m| m.contains("@playwright/mcp") && m.contains("3 times")));
    assert!(dups.iter().any(|m| m.contains("api.githubcopilot.com/mcp")));
    let mism: Vec<&str> = inv
        .findings
        .iter()
        .filter(|f| f.kind == FindingKind::VersionMismatch)
        .map(|f| f.message.as_str())
        .collect();
    assert_eq!(mism.len(), 2, "{mism:#?}");
    assert!(mism.iter().any(|m| m.contains("1.2.0 / 1.3.0")));
    assert!(mism.iter().any(|m| m.contains("0.0.29 / 0.0.30")));
}

#[test]
fn secrets_are_found_masked_and_never_shown() {
    let (_tmp, inv) = run();
    assert_eq!(
        inv.count(FindingKind::PlainSecret),
        3,
        "{:#?}",
        inv.findings
    );
    let text = inv.all_text();
    for clear in [
        "sk-proj-abcdefghij",
        "ghp_abcdefghij",
        "0123456789abcdef0123456789abcdef",
    ] {
        assert!(!text.contains(clear), "clear secret in output: {clear}");
    }
    // Masked forms are there.
    assert!(text.contains("sk-pro…6789"));
    // References are shown as written and are not findings.
    assert!(text.contains("${GITHUB_TOKEN}"));
    assert!(text.contains("${{ secrets.SUPABASE_TOKEN }}"));
    assert!(text.contains("{env:MY_API_KEY}"));
    assert!(text.contains("${input:pw-token}"));
    assert!(text.contains("${DB_URL:-postgres://localhost/db}"));
}

#[test]
fn other_findings() {
    let (_tmp, inv) = run();
    let msgs = |k| -> Vec<String> {
        inv.findings
            .iter()
            .filter(|f| f.kind == k)
            .map(|f| f.message.clone())
            .collect()
    };
    let nf = msgs(FindingKind::CommandNotFound);
    assert_eq!(nf.len(), 2, "{nf:#?}");
    assert!(nf.iter().any(|m| m.contains("ghost-mcp")));
    assert!(nf.iter().any(|m| m.contains("old-mcp")));
    let wide = msgs(FindingKind::WideAccess);
    assert_eq!(wide.len(), 1, "{wide:#?}");
    assert!(wide[0].contains("/home"));
    assert_eq!(msgs(FindingKind::ConfirmationsSkipped).len(), 2);
    assert_eq!(msgs(FindingKind::Disabled).len(), 2);
    let unreadable = msgs(FindingKind::Unreadable);
    assert_eq!(unreadable.len(), 1, "{unreadable:#?}");
    assert!(unreadable[0].contains("windsurf"));
}

#[test]
fn extension_metadata_is_attached() {
    let (_tmp, inv) = run();
    let eliot = inv.declarations.iter().find(|d| d.name == "eliot").unwrap();
    assert_eq!(eliot.version.as_deref(), Some("0.1.0"));
    assert!(eliot.enabled);
    assert_eq!(eliot.hosts, vec!["claude-desktop-extensions".to_string()]);
    // The ${__dirname} command resolved to an existing file: not reported missing.
    assert!(!inv
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::CommandNotFound && f.message.contains("eliot")));
}

#[test]
fn legacy_locations_are_marked() {
    let (_tmp, inv) = run();
    let old = inv.declarations.iter().find(|d| d.name == "old").unwrap();
    assert!(old.legacy);
    assert!(old.file.ends_with("Code/User/settings.json"));
}

#[test]
fn nothing_is_written() {
    // Every file of the fake home has the same content after the inventory.
    let tmp = tempfile::tempdir().unwrap();
    copy_fixture(tmp.path());
    let before: Vec<(PathBuf, Vec<u8>)> = walkdir(tmp.path())
        .into_iter()
        .map(|p| (p.clone(), std::fs::read(&p).unwrap()))
        .collect();
    let env = Env::fixture(tmp.path(), Os::Linux);
    let _ = inventory::run(&hosts::embedded(), &env);
    let after = walkdir(tmp.path());
    assert_eq!(after.len(), before.len());
    for (p, bytes) in before {
        assert_eq!(std::fs::read(&p).unwrap(), bytes, "{} changed", p.display());
    }
}
