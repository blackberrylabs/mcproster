//! What the user could do about a finding — displayed, never executed (BRIEF R1). The host's
//! own command when its rule gives one, otherwise where to edit.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::hosts::{Rules, Scope};
use crate::model::{Finding, FindingKind};
use crate::Inventory;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Action {
    /// Run this command of the host itself (copyable).
    HostCommand { host: String, command: String },
    /// Edit the entry in the host's own settings, then restart it.
    EditInHost { host: String },
    /// Open the file and remove or fix the entry.
    OpenFile,
    /// Disable the server in the host, or let the host load tools on demand.
    ReduceContext { host: String, may_defer: bool },
}

pub fn actions_for(rules: &Rules, inv: &Inventory, f: &Finding) -> Vec<Action> {
    if f.kind == FindingKind::Unreadable {
        return vec![Action::OpenFile];
    }
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for &di in &f.declarations {
        let Some(d) = inv.declarations.get(di) else {
            continue;
        };
        for (hid, hname) in d.hosts.iter().zip(d.host_names.iter()) {
            if !seen.insert(hid.clone()) {
                continue;
            }
            if f.kind == FindingKind::ContextCost {
                out.push(Action::ReduceContext {
                    host: hname.clone(),
                    may_defer: hid == "claude-code",
                });
                continue;
            }
            match rules.host(hid).and_then(|h| h.remove_hint.clone()) {
                Some(h) => {
                    let command = h.replace("<name>", &d.name).replace(
                        "<user|local|project>",
                        if d.scope == Scope::User {
                            "user"
                        } else {
                            "local"
                        },
                    );
                    out.push(Action::HostCommand {
                        host: hname.clone(),
                        command,
                    });
                }
                None if hid == "claude-desktop" => out.push(Action::EditInHost {
                    host: hname.clone(),
                }),
                None => out.push(Action::OpenFile),
            }
        }
    }
    out.dedup();
    out
}
