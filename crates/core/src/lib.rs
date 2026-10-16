//! MCProster core: a read-only inventory of the MCP servers declared on a machine.
//!
//! The library reads host rules (`hosts/*.toml`, embedded), resolves the files they point
//! at for a given environment, reads those files, and produces an [`inventory::Inventory`]:
//! every declaration with its host, file and scope; the servers grouped across hosts; and
//! the findings (duplicates, plain-text secrets, missing commands, wide filesystem access,
//! skipped confirmations, unreadable files).
//!
//! Three promises, enforced here and tested:
//! - nothing is ever written: no `File::create`, no `remove`, no `rename` in this crate;
//! - nothing leaves the machine: no network code in this crate;
//! - a secret is masked the moment it is read and never stored in clear.
#![forbid(unsafe_code)]

pub mod env;
pub mod hosts;
pub mod inventory;
pub mod model;
pub mod read;
pub mod secrets;

pub use env::Env;
pub use hosts::{HostRule, Rules};
pub use inventory::Inventory;
