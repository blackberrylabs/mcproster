//! Context cost: what the probed servers' tool definitions weigh, per server and per host,
//! and the `context-cost` finding when it is too much. Thresholds and method are documented
//! in `docs/context-cost.md` (BRIEF §11, D8–D9).

use std::collections::BTreeMap;

use serde::Serialize;

use crate::model::{Finding, FindingKind, Transport};
use crate::probe::ProbeResult;
use crate::Inventory;

/// A single server above this is a finding (BRIEF D8).
pub const SERVER_THRESHOLD: u64 = 5_000;
/// A host's probed servers together above this is a finding — about 10 % of a 200k window.
pub const HOST_THRESHOLD: u64 = 20_000;

#[derive(Debug, Clone, Serialize)]
pub struct ServerCost {
    pub identity: String,
    pub tools: usize,
    pub tokens: u64,
    /// The heaviest tools, `(name, tokens)`, at most three.
    pub heaviest: Vec<(String, u64)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HostCost {
    pub host: String,
    pub host_name: String,
    /// Sum over the enabled stdio servers this host declares that have been probed.
    pub tokens: u64,
    pub probed: usize,
    /// Enabled stdio servers declared by this host and not probed yet: the total is "at least".
    pub unprobed: usize,
    /// Probed but the probe failed (no answer, missing command): unknown weight.
    pub failed: usize,
    /// Remote servers (never probed) declared by this host.
    pub remote: usize,
    /// The host may load MCP tools on demand, so the figure is an upper bound.
    pub may_defer: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CostReport {
    pub servers: Vec<ServerCost>,
    pub hosts: Vec<HostCost>,
    pub findings: Vec<Finding>,
}

/// Hosts documented as able to defer MCP tool loading (tool search): Claude Code.
fn may_defer(host_id: &str) -> bool {
    host_id == "claude-code"
}

/// Compute the report from an inventory and the saved probe results (keyed by identity).
pub fn report(inv: &Inventory, probes: &BTreeMap<String, ProbeResult>) -> CostReport {
    let mut out = CostReport::default();
    // Per server.
    let mut server_tokens: BTreeMap<&str, u64> = BTreeMap::new();
    for s in &inv.servers {
        let Some(r) = probes.get(&s.identity).filter(|r| r.ok) else {
            continue;
        };
        let mut heaviest: Vec<(String, u64)> =
            r.tools.iter().map(|t| (t.name.clone(), t.tokens)).collect();
        heaviest.sort_by_key(|(_, t)| std::cmp::Reverse(*t));
        heaviest.truncate(3);
        server_tokens.insert(s.identity.as_str(), r.estimated_tokens);
        if r.estimated_tokens > SERVER_THRESHOLD {
            out.findings.push(Finding {
                kind: FindingKind::ContextCost,
                message: format!(
                    "`{}`: ~{} tokens of tool definitions ({} tool(s)) sent to the model at every request — above {}",
                    s.identity,
                    thousands(r.estimated_tokens),
                    r.tools.len(),
                    thousands(SERVER_THRESHOLD)
                ),
                declarations: s.declarations.clone(),
                file: None,
            });
        }
        out.servers.push(ServerCost {
            identity: s.identity.clone(),
            tools: r.tools.len(),
            tokens: r.estimated_tokens,
            heaviest,
        });
    }
    // Per host: every enabled declaration of the host, grouped by its server identity so a
    // server declared twice by the same host counts once.
    for h in inv
        .hosts
        .iter()
        .filter(|h| h.declarations > 0 || h.files_found > 0)
    {
        let mut tokens = 0u64;
        let mut probed = 0usize;
        let mut unprobed = 0usize;
        let mut failed = 0usize;
        let mut remote = 0usize;
        let mut seen: Vec<&str> = Vec::new();
        for s in &inv.servers {
            let declared_here = s
                .declarations
                .iter()
                .map(|&i| &inv.declarations[i])
                .any(|d| d.enabled && d.hosts.contains(&h.id));
            if !declared_here || seen.contains(&s.identity.as_str()) {
                continue;
            }
            seen.push(s.identity.as_str());
            let stdio = s
                .declarations
                .iter()
                .any(|&i| inv.declarations[i].transport == Transport::Stdio);
            if !stdio {
                remote += 1;
                continue;
            }
            match server_tokens.get(s.identity.as_str()) {
                Some(t) => {
                    tokens += t;
                    probed += 1;
                }
                None if probes.contains_key(&s.identity) => failed += 1,
                None => unprobed += 1,
            }
        }
        if probed + unprobed + failed + remote == 0 {
            continue;
        }
        if tokens > HOST_THRESHOLD {
            let decls: Vec<usize> = inv
                .declarations
                .iter()
                .enumerate()
                .filter(|(_, d)| d.enabled && d.hosts.contains(&h.id))
                .map(|(i, _)| i)
                .collect();
            out.findings.push(Finding {
                kind: FindingKind::ContextCost,
                message: format!(
                    "{}: ~{} tokens of MCP tool definitions across {} probed server(s){} — above {}{}",
                    h.name,
                    thousands(tokens),
                    probed,
                    if unprobed > 0 { format!(" (at least: {unprobed} not probed)") } else { String::new() },
                    thousands(HOST_THRESHOLD),
                    if may_defer(&h.id) { "; this host can load tools on demand, so this is an upper bound" } else { "" }
                ),
                declarations: decls,
                file: None,
            });
        }
        out.hosts.push(HostCost {
            host: h.id.clone(),
            host_name: h.name.clone(),
            tokens,
            probed,
            unprobed,
            failed,
            remote,
            may_defer: may_defer(&h.id),
        });
    }
    out
}

/// `12 400` style grouping.
pub fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('\u{202f}');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouping() {
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(12400), "12\u{202f}400");
        assert_eq!(thousands(1_234_567), "1\u{202f}234\u{202f}567");
    }
}
