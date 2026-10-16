# Hosts

How MCProster knows where each assistant keeps its MCP server declarations. One rule per
host in `hosts/*.toml`; the inventory reads the rules, then the files they point at, and
nothing else. **It never writes to any of them.**

Every location carries its source: the vendor's documentation, the host's source code, or
an observation on a real machine — with the URL, the date it was read and the words it
was read from. A location whose source is `observed` is reported as such in the output.
No rule may rest on memory.

## Rule schema

```toml
id = "cursor"                 # stable identifier, file name
name = "Cursor"               # as shown
vendor = "Anysphere"
kind = "editor"               # desktop | cli | editor | extension
status = "active"             # active | out-of-scope
references = ["${env:NAME}"]  # variable syntaxes the host expands itself: never masked, never a secret
remove_hint = "…"             # optional: the host's own command to remove a server (displayed only)
list_hint = "…"

[[locations]]
scope = "user"                # user | project | system
format = "json"               # json | jsonc | toml | yaml
servers_key = "mcpServers"    # dotted path to the servers; `*` matches any key; `\.` a literal dot
entry = "object"              # object: name → spec · array: list of specs with name_key · single: one spec
name_key = "name"             # for array / single
project_key = "projects.*"    # when the project path is a key inside the file (Claude Code local scope)
path.any = "~/.cursor/mcp.json"        # or path.linux / path.macos / path.windows; a list means "first that exists"
map = { command = "command", args = "args", env = "env", url = "url", headers = "headers", type = "type" }
command_is_array = false      # OpenCode: the program is args[0]
legacy = false                # kept for old installs, reported as legacy
shared_with = ["claude-code"] # the same file is read by another host: one declaration, several hosts
role = "registry"             # optional: not servers but metadata about them (versions, enabled)

[locations.source]
kind = "documentation"        # documentation | source-code | observed
url = "https://…"
read_on = 2026-10-16
quote = "the words the path and key were read from"
note = "what the source does not say, and what was derived or observed instead"
```

`map` translates the host's field names to the common model (`command`, `args`, `env`, `cwd`,
`url`, `headers`, `type`, `enabled`/`disabled`, `env_file`, `env_keys`, `bearer_env`,
`env_headers`, `trust`). A value may be a list of candidates (`url = ["httpUrl", "url"]`).

Per-project locations are relative to a project root. Where to look for project roots is
an inventory setting; the default list is the keys of `projects` in `~/.claude.json`, which
are the absolute paths of every project Claude Code has been run in, plus the roots the
user adds.

## Locations read (38 across 19 hosts)

Paths shown for Linux; macOS and Windows are in the rule files.

| Host | Scope | Path | Format | Servers key | Source | Notes |
|---|---|---|---|---|---|---|
| Amp | user | `~/.config/amp/settings.json · ~/.config/amp/settings.jsonc` | jsonc | `amp.mcpServers` | [doc](https://ampcode.com/docs/cli/settings) |  |
| Amp | project | `.amp/settings.json · .amp/settings.jsonc` | jsonc | `amp.mcpServers` | [doc](https://ampcode.com/docs/cli/settings) |  |
| Claude Code | user | `~/.claude.json` | json | `mcpServers` | [doc](https://code.claude.com/docs/en/mcp) |  |
| Claude Code | project | `~/.claude.json` | json | `projects.*.mcpServers` | [doc](https://code.claude.com/docs/en/mcp) |  |
| Claude Code | project | `.mcp.json` | json | `mcpServers` | [doc](https://code.claude.com/docs/en/mcp) | shared with vscode |
| Claude Desktop (extensions, .mcpb) | user | `~/.config/Claude/Claude Extensions/*/manifest.json` | json | `server.mcp_config` | [doc](https://github.com/anthropics/mcpb/blob/main/MANIFEST.md) |  |
| Claude Desktop (extensions, .mcpb) | user | `~/.config/Claude/extensions-installations.json` | json | `extensions` | observed | registry |
| Claude Desktop | user | `~/.config/Claude/claude_desktop_config.json` | json | `mcpServers` | [doc](https://modelcontextprotocol.io/quickstart/user) |  |
| Cline | user | `$CLINE_MCP_SETTINGS_PATH · ~/.cline/data/settings/cline_mcp_settings.json` | json | `mcpServers` | [doc](https://docs.cline.bot/mcp/configuring-mcp-servers) |  |
| Cline | user | `~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json` | json | `mcpServers` | [code](https://github.com/cline/cline/blob/main/apps/vscode/src/core/storage/disk.ts) |  |
| Codex CLI | user | `~/.codex/config.toml` | toml | `mcp_servers` | [doc](https://learn.chatgpt.com/docs/extend/mcp?surface=cli) |  |
| Codex CLI | project | `.codex/config.toml` | toml | `mcp_servers` | [doc](https://learn.chatgpt.com/docs/extend/mcp?surface=cli) |  |
| Continue | user | `~/.continue/config.yaml` | yaml | `mcpServers` | [doc](https://docs.continue.dev/customize/deep-dives/configuration) |  |
| Continue | project | `.continue/mcpServers/*.yaml` | yaml | `mcpServers` | [doc](https://docs.continue.dev/customize/deep-dives/mcp) |  |
| Continue | user | `~/.continue/config.json` | json | `experimental.modelContextProtocolServers` | observed | legacy |
| GitHub Copilot CLI | user | `$COPILOT_HOME/mcp-config.json · ~/.copilot/mcp-config.json` | json | `mcpServers` | [doc](https://code.visualstudio.com/docs/copilot/reference/mcp-configuration) |  |
| Cursor | user | `~/.cursor/mcp.json` | json | `mcpServers` | [doc](https://cursor.com/docs/context/mcp) |  |
| Cursor | project | `.cursor/mcp.json` | json | `mcpServers` | [doc](https://cursor.com/docs/context/mcp) |  |
| Gemini CLI | user | `~/.gemini/settings.json` | json | `mcpServers` | [doc](https://geminicli.com/docs/tools/mcp-server/) |  |
| Gemini CLI | project | `.gemini/settings.json` | json | `mcpServers` | [doc](https://geminicli.com/docs/tools/mcp-server/) |  |
| Goose | user | `~/.config/goose/config.yaml` | yaml | `extensions` | [doc](https://block.github.io/goose/docs/guides/config-file/) |  |
| Kiro | user | `~/.kiro/settings/mcp.json` | json | `mcpServers` | [doc](https://kiro.dev/docs/mcp/configuration/) |  |
| Kiro | project | `.kiro/settings/mcp.json` | json | `mcpServers` | [doc](https://kiro.dev/docs/mcp/configuration/) |  |
| LM Studio | user | `~/.lmstudio/mcp.json` | json | `mcpServers` | [doc](https://lmstudio.ai/blog/mcp) |  |
| OpenCode | user | `~/.config/opencode/opencode.json · ~/.config/opencode/opencode.jsonc · $OPENCODE_CONFIG` | jsonc | `mcp` | [doc](https://opencode.ai/docs/config/) |  |
| OpenCode | project | `opencode.json · opencode.jsonc` | jsonc | `mcp` | [doc](https://opencode.ai/docs/config/) |  |
| OpenCode | system | `/etc/opencode/*.json` | jsonc | `mcp` | [doc](https://opencode.ai/docs/config/) |  |
| Qwen Code | user | `~/.qwen/settings.json` | json | `mcpServers` | [doc](https://qwenlm.github.io/qwen-code-docs/en/users/features/mcp/) |  |
| Qwen Code | project | `.qwen/settings.json` | json | `mcpServers` | [doc](https://qwenlm.github.io/qwen-code-docs/en/users/features/mcp/) |  |
| Roo Code | user | `~/.config/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/mcp_settings.json` | json | `mcpServers` | [code](https://github.com/RooCodeInc/Roo-Code/blob/main/src/shared/globalFileNames.ts) |  |
| Roo Code | project | `.roo/mcp.json` | json | `mcpServers` | [doc](https://roocodeinc.github.io/Roo-Code/features/mcp/using-mcp-in-roo/) |  |
| Visual Studio Code (Copilot) | user | `~/.config/Code/User/mcp.json · ~/.config/Code/User/profiles/*/mcp.json` | jsonc | `servers` | [doc](https://code.visualstudio.com/docs/copilot/reference/mcp-configuration) |  |
| Visual Studio Code (Copilot) | project | `.vscode/mcp.json` | jsonc | `servers` | [doc](https://code.visualstudio.com/docs/copilot/reference/mcp-configuration) |  |
| Visual Studio Code (Copilot) | project | `.mcp.json` | json | `mcpServers` | [doc](https://code.visualstudio.com/docs/copilot/reference/mcp-configuration) | shared with claude-code |
| Visual Studio Code (Copilot) | user | `~/.config/Code/User/settings.json` | jsonc | `mcp.servers` | observed | legacy |
| Windsurf (Cascade) | user | `~/.codeium/windsurf/mcp_config.json` | json | `mcpServers` | [doc](https://docs.windsurf.com/windsurf/cascade/mcp) |  |
| Zed | user | `$XDG_CONFIG_HOME/zed/settings.json · ~/.config/zed/settings.json · $FLATPAK_XDG_CONFIG_HOME/zed/settings.json` | jsonc | `context_servers` | [code](https://github.com/zed-industries/zed/blob/main/crates/paths/src/paths.rs) |  |
| Zed | project | `.zed/settings.json` | jsonc | `context_servers` | [code](https://github.com/zed-industries/zed/blob/main/crates/paths/src/paths.rs) |  |

## Hosts not read in v0

| Host | Why |
|---|---|
| Kilo Code | Rebuilt in 2026 (packages/kilo-vscode, publisher kilocode, name kilo-code — package.json read 2026-10-16); the previous Roo-like mcp_settings.json path no longer applies and the new location is not documented. To establish. |
| JetBrains AI Assistant / Junie | MCP servers are kept in the IDE's options XML, per IDE and version. Not established. |
| Warp | MCP servers are stored inside the application (no user-editable file documented). |
| 5ire | Not established. |
| Aider | No native MCP support. |
| Trae | Not established. |

## Things learnt while writing the rules

- `.mcp.json` at a project root is read by Claude Code **and** by VS Code (Copilot): one
  file, two hosts. The inventory shows it once, attributed to both, not as a duplicate.
- Several hosts name the file but not its path (Zed, Roo Code, Cline's extension, Continue's
  MCP page): the path comes from their source code, cited file by file.
- Every host has its own way to say "take this from the environment": `${VAR}` (Claude
  Code), `${env:NAME}` (Cursor, VS Code, Roo), `{env:VAR}` (OpenCode), `${{ secrets.NAME }}`
  (Continue), `bearer_token_env_var` (Codex), `env_keys` (Goose). These are references, not
  secrets, and must never be flagged or masked. Only a literal value is a finding.
- A disabled server is still a declaration: `disabled: true` (Cline, Roo, Kiro),
  `enabled: false` (Codex, OpenCode, Goose, Zed), `amp.mcpPermissions` reject rules (Amp),
  `isEnabled` (Claude Desktop extensions settings). Shown as declared and disabled.
- `trust: true` (Gemini CLI, Qwen Code) and `autoApprove` / `alwaysAllow` lists (Cline,
  Roo, Kiro) skip the user's confirmation for a server's tools: worth a finding.
