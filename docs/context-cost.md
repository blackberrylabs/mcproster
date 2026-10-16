# Context cost

What the MCP servers declared on a machine cost in the model's context window, as
MCProster 0.2 estimates it — method, thresholds, limits, sources.

## Why

Every host sends the definitions of every available tool (name, description, input
schema) to the model with every request. One user measured about **43 000 tokens of a
99 000-token context consumed by 127 MCP tools from 35 plugins at session start** — four
times the skills catalogue, fourteen times the system prompt — and had to set up a
logging proxy to see it ([Szablowski, 2026](https://www.adrianszablowski.com/blog/claude-code-context-bloat-is-mcp)).
MCProster already receives those definitions when it probes a server (`tools/list`); 0.2
weighs them.

## Method

- The probe (`mcproster probe <server>` or `probe --all`) keeps, for each tool returned by
  `tools/list`, the number of characters of its JSON definition (`name`, `description`,
  `inputSchema`, plus `outputSchema` / `annotations` when present).
- **Estimated tokens = characters / 4, rounded up.** Four characters per token is the usual
  rule of thumb for English text and JSON; real tokenizers differ by model, and each host
  serialises tool definitions its own way (some add a prefix per server, some reformat the
  schema), so the figure is an **estimate, ±20 %**, always shown with "~".
- Per server: number of tools, ~tokens, the three heaviest tools.
- Per host — more exactly per *application*: the Claude Desktop configuration file and its
  `.mcpb` extensions are two host rules but one application, so their costs are summed
  (rule field `application`). The sum covers the **enabled** stdio servers the application
  declares that have been probed. A server declared twice by the same host counts once. Servers not probed yet make
  the total "at least"; failed probes are counted separately; remote servers are never
  probed (no network) and are listed as such.
- **Claude Code** can load MCP tools on demand (tool search) instead of sending them all at
  every request; its figure is therefore an upper bound and the output says so.

## Thresholds (BRIEF D8)

| | Threshold | Why |
|---|---|---|
| One server | **~5 000 tokens** | A single server above this is worth a look: the measured outliers were 26-tool servers at 30 KB of schema (~8 000 tokens). |
| One host | **~20 000 tokens** | About 10 % of a 200 000-token window, before any conversation. The measured case was 43 000. |

Above a threshold, MCProster raises a `context-cost` finding with the action the user
could take: disable the servers they do not use in that host (each host has a `disabled`
/ `enabled` flag or a settings page), or, for Claude Code, rely on on-demand loading.
Below, the figures are shown as information. The thresholds are absolute on purpose: the
same numbers for everyone, no setting to tune. A relative mode (percentage of a chosen
window) will come only if users ask for it.

## What is kept on disk (BRIEF D9)

`~/.config/mcproster/probes.json` keeps, per probed server: date, command, status,
server name and version, protocol version, total schema size, estimated tokens, and for
each tool **its name and its tokens**. Tool descriptions and schemas are shown live during
a probe and **never written**: nothing from a server's output is stored as text.

## Package cache (0.2)

A server launched through `npx -y <package>` or `uvx <package>` works even when the
package is not on the machine — the runner downloads it first. MCProster 0.2 reports
`package-not-cached` (information, not a problem) when the package is not found in the
local cache:

- npx: `~/.npm/_npx/<hash>/node_modules/<package>/package.json` (observed layout).
- uv: `~/.cache/uv/archive-v0/<hash>/lib/python*/site-packages/<name>` with `-` → `_`
  (observed layout); pipx: `~/.local/pipx/venvs/<package>`.

Both layouts were observed, not read in documentation; a cache kept elsewhere
(`npm_config_cache`, `UV_CACHE_DIR`) is not seen yet.

## Limits

- No probe, no cost: the method needs `tools/list`, which only the probe asks (on demand,
  BRIEF R4).
- Estimates, not measurements: to know the real figure for one host, use that host's own
  tool (`/context` in Claude Code).
- Resources and prompts exposed by a server are not weighed, only tools.
