# Changelog

All notable changes to MCProster. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [Semantic Versioning](https://semver.org/).

## [0.2.1] — 2026-10-17

MCProster becomes a module of the Blackberrylabs control tower. Nothing changes on screen.

### Added

- `mcproster report`: the inventory as a [`bbl-report/1`](https://github.com/blackberrylabs/bbl-report)
  document for the Blackberrylabs control tower (always JSON). Nothing in it is new: the four
  figures of the app's cards, the findings (disabled servers included, as information), the
  declaring files and the same actions the CLI and the app already show, secrets masked as
  everywhere else. MCProster's findings map onto the four common severities as follows — the
  tower never reinterprets:

  | MCProster | Common severity |
  |---|---|
  | secret in clear | 1 · act now |
  | wide access, confirmations skipped | 2 · review |
  | declared twice, versions differ, command missing, context cost | 3 · good to know |
  | unreadable, disabled, not in cache | 4 · information |

- `mcproster-app --select <server>` opens the app on that server (its identity as `list`
  shows it), the way the tower does.

### Changed

- The interface building blocks, palette, preferences and translated strings now come from
  the shared crate [`bbl-ui`](https://github.com/blackberrylabs/bbl-ui), like the other
  Blackberrylabs tools. Nothing visible changes.
- The common ground of the host rules (schema, loading, validation, path resolution,
  readers, secret masking) moved to the shared crate
  [`bbl-hostrules`](https://github.com/blackberrylabs/bbl-hostrules) (named `hostrules`
  until its 0.3.0), so the next Blackberrylabs
  tool reads the same files the same way. No change to what MCProster shows; the only
  visible difference is in `mcproster --json hosts`, where `remove_hint` / `list_hint`
  became a `hints` table (`hints.remove`, `hints.list`).

## [0.2.0] — 2026-10-16

What your MCP servers cost in the model's context — the question the community was
answering with logging proxies.

### Added

- **Context cost.** The probe now keeps the size of every tool definition returned by
  `tools/list` and estimates its tokens (characters / 4, shown with "~"): per tool, per
  server (with the three heaviest tools) and per host (sum of its enabled, probed servers;
  "at least" when some are not probed; Claude Code marked as an upper bound since it can
  load tools on demand). A `context-cost` finding above ~5 000 tokens for one server or
  ~20 000 for one host, with the action to take. New `mcproster context` command; `~tokens`
  column and "Context cost per host" block in the app. Method, thresholds and limits in
  `docs/context-cost.md`.
- **Test all.** `mcproster probe --all` (and a *Test all* button) probes every enabled stdio
  server, one launch per distinct command, after **one confirmation that lists the
  commands** (`--yes` to skip); results are then summarised per host.
- **Package cache.** `package-not-cached` (information) when an `npx` / `uvx` server's
  package is not in the local cache: it will work, but will download first.
- **Actions in the CLI.** `mcproster findings` now prints, under each finding, what you
  could do — the host's own command (`claude mcp remove …`, `codex mcp remove …`), its
  settings page, or the file to open — as the app already did.

### Changed

- Probe results no longer keep tool descriptions on disk: `probes.json` holds names,
  versions, sizes and tokens only. Descriptions are shown live during a probe.
- Failed probes are counted apart from servers not probed yet.

### Known limitations

- Token figures are estimates (±20 %); each host serialises tool definitions its own way.
- Only tools are weighed, not resources or prompts.
- Kilo Code is still not read (configuration location changed in 2026, not yet stable).

## [0.1.0] — 2026-10-16

First release. A read-only inventory of the MCP servers declared on a Linux machine, across
the assistant hosts that can run them. **It never writes to a configuration file.**

### Added

- **19 host rules** (`hosts/*.toml`, 39 locations), each location established from the
  vendor's documentation or source code — URL, date, quote — or marked as observed on a
  real machine: Claude Desktop (configuration and `.mcpb` extensions), Claude Code (user,
  local and project scopes), Cursor, Windsurf, Visual Studio Code (Copilot, including the
  shared `.mcp.json`), GitHub Copilot CLI, Cline, Roo Code, Codex CLI, Gemini CLI, Qwen Code,
  OpenCode, Continue, Zed, Goose, Amp, LM Studio, Kiro. Six hosts listed as not read, with
  the reason.
- **Readers** for JSON, JSONC (comments, trailing commas), TOML and YAML into one common
  model; per-OS paths; `~`, `$VAR`, `${VAR}` and `%VAR%` in rule paths; per-project
  locations searched in the projects Claude Code knows plus any `--root`.
- **Servers grouped by identity** — URL, package behind `npx` / `uvx` / `pipx` / `bunx`
  (version set aside), or binary — so the same server declared by several hosts is one row;
  a file read by two hosts (`.mcp.json`) is one declaration for both, not a duplicate.
- **Findings**: secret in clear (by key name and by value shape, `Bearer` headers and URL
  query tokens included), wide filesystem access (`/`, `/home`, `~`), confirmations skipped
  (`trust`, `autoApprove`, `alwaysAllow`), command not found, versions differ, declared
  twice or more, unreadable file, disabled. Each with the action the user could take —
  displayed, never executed.
- **Secrets masked** the moment they are read (`6…4`), in every output; references the
  host expands itself (`${VAR}`, `${env:NAME}`, `{env:VAR}`, `${{ secrets.NAME }}`,
  `bearer_token_env_var`, `env_keys`) are never masked and never flagged.
- **Probe on demand** (`mcproster probe <server>`): the server is started as its host
  would, asked `initialize` and `tools/list`, and stopped; 10 s at most; no tool called;
  remote servers never contacted. Results (names, versions, tool names) are dated and kept
  in `~/.config/mcproster/probes.json`, the only file MCProster writes besides its settings.
- **Command line** (`mcproster`): `hosts`, `list`, `findings`, `files`, `probe`, `--json`,
  `--home`, `--root`.
- **Desktop app** (`mcproster-app`, Rust + iced): servers × hosts table (one column per
  host up to six, a single "declared in" column beyond), text filter, host picker,
  "findings only" and "disabled & legacy" toggles; detail with *Open file*, *Copy path*,
  *Copy command* and *Test* (confirmation before the first probe, remembered); findings
  with actions; English by default, French on demand; light and dark themes.
- **Packages**: `.deb`, `.rpm` and AppImage built by `packaging/build.sh`; one package ships
  both binaries, the desktop entry and the icons.
- **Tests**: a fake home representing every host (26 declarations, 19 expected findings),
  "nothing is written" and "no clear secret anywhere in the output", a fake MCP server for
  the probe (answer, time limit, missing command, reference expansion).

### Known limitations

- Linux only; macOS and Windows paths are in the rules but untested.
- Remote servers are not probed (no network, by design).
- The `.deb` was installed on Ubuntu 24.04; the `.rpm` is built but not yet installed on a
  Fedora machine.
- Package presence in the `npx` / `uv` caches is not checked yet: a missing command is
  reported, a package that would have to be downloaded is not.
- Kilo Code, JetBrains, Warp, 5ire, Trae are not read yet.

[0.2.0]: https://github.com/blackberrylabs/mcproster/releases/tag/v0.2.0
[0.1.0]: https://github.com/blackberrylabs/mcproster/releases/tag/v0.1.0
