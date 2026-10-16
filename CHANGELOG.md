# Changelog

All notable changes to MCProster. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [Semantic Versioning](https://semver.org/).

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
