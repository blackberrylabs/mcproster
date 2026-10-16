//! The probe against a fake stdio MCP server (`tests/fixtures/servers/fake_mcp.py`).

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use mcproster_core::probe::{probe_stdio, PROTOCOL_VERSION};

fn server() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/servers/fake_mcp.py")
        .display()
        .to_string()
}

#[test]
fn initialize_and_tools_list_are_collected() {
    let r = probe_stdio(
        "python3",
        &[server()],
        &BTreeMap::new(),
        None,
        &BTreeMap::new(),
        Duration::from_secs(10),
    );
    assert!(r.ok, "{r:#?}");
    assert_eq!(r.server_name.as_deref(), Some("fake-mcp"));
    assert_eq!(r.server_version.as_deref(), Some("9.9.9"));
    assert_eq!(r.protocol_version.as_deref(), Some(PROTOCOL_VERSION));
    let names: Vec<&str> = r.tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, vec!["ping", "echo"]);
    // First line only, truncated at 120 characters.
    assert_eq!(r.tools[0].description.as_deref(), Some("Answers pong."));
    assert!(r.tools[1].description.as_ref().unwrap().ends_with('…'));
    assert!(r.duration_ms < 10_000);
}

#[test]
fn a_silent_server_hits_the_time_limit_and_is_stopped() {
    let mut env = BTreeMap::new();
    env.insert("SLOW".to_string(), "1".to_string());
    let r = probe_stdio(
        "python3",
        &[server()],
        &env,
        None,
        &BTreeMap::new(),
        Duration::from_millis(800),
    );
    assert!(!r.ok);
    assert!(r.error.as_deref().unwrap().contains("time limit"), "{r:#?}");
    assert!(
        r.duration_ms < 5_000,
        "the child was not stopped promptly: {} ms",
        r.duration_ms
    );
}

#[test]
fn a_missing_command_is_reported_not_panicked() {
    let r = probe_stdio(
        "/nonexistent/mcp-server",
        &[],
        &BTreeMap::new(),
        None,
        &BTreeMap::new(),
        Duration::from_secs(2),
    );
    assert!(!r.ok);
    assert!(r.error.as_deref().unwrap().contains("cannot start"));
}

#[test]
fn env_references_are_expanded_for_the_child_only() {
    // The fake server ignores env; this checks expansion does not break launching.
    let mut env = BTreeMap::new();
    env.insert("TOKEN".to_string(), "${MY_TOKEN}".to_string());
    let mut vars = BTreeMap::new();
    vars.insert("MY_TOKEN".to_string(), "secret-value".to_string());
    let r = probe_stdio(
        "python3",
        &[server()],
        &env,
        None,
        &vars,
        Duration::from_secs(10),
    );
    assert!(r.ok, "{r:#?}");
    assert!(!serde_json::to_string(&r).unwrap().contains("secret-value"));
}
