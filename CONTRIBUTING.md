# Contributing

Thank you for looking. MCProster is small and deliberately narrow — it reads, shows and
explains; it never changes a configuration. Contributions that keep it that way are
welcome.

## Adding or fixing a host

A host is one file in `hosts/`, read by every binary. Copy the closest existing rule and
fill in each location **with its source**: the vendor's documentation page or the source
file that gives the path, format and key, the date you read it, and the words you read it
from. If you only know a path because you saw it on your machine, say so with
`kind = "observed"` — the output will tell users as much. A rule without a source is
refused at load time, on purpose. The schema is in [`docs/hosts.md`](docs/hosts.md).

Then add the host to the fake home in `crates/core/tests/fixtures/home/` (use `__HOME__`
for the home directory; it is replaced when the tests copy the fixture) and adjust the
expected counts in `crates/core/tests/inventory.rs`. `cargo test` must stay green, and
`docs/hosts.md` is regenerated from the rules.

## Adding a kind of finding

A new `FindingKind` needs its label in the CLI and the app, and its place on the four
common severities of the Blackberrylabs tower (`severity_of` in
`crates/core/src/report.rs`) — act now, review, good to know, information — written in the
CHANGELOG. The tower never reinterprets.

## What will not be merged

- Anything that writes to, moves or deletes a host's configuration file, even with a
  confirmation.
- Anything that opens a network connection: no update check, no telemetry, no remote
  probe.
- A path, key or format "from memory": each location needs its source.
- Showing a secret in clear anywhere, including in debug output.

## Running

```bash
cargo build --release
cargo test --workspace
./target/release/mcproster --home crates/core/tests/fixtures/home list   # fixture paths are placeholders; the tests copy it with real ones
packaging/build.sh deb                                                   # needs cargo-deb
```

`cargo fmt --all` and `cargo clippy --workspace --all-targets` before a pull request.
Code, comments, commit messages and documentation are in English; the project's own
brief, plan and journal (`BRIEF.md`, `PLAN.md`, `JOURNAL.md`) are in French.

## Reporting

Open an issue with the output of `mcproster files` and `mcproster --json findings` —
secrets are masked by construction, but read it once before pasting.
