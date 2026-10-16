# MCProster

**Status: design phase — nothing to install yet.**

MCProster will be a read-only inventory of the MCP servers declared on a Linux
machine, across the assistant hosts that can run them (Claude Desktop, Claude
Code, Cursor, VS Code, Codex, Gemini CLI, OpenCode, Continue, Zed…). One list,
with every place each server is declared, the duplicates, the dead entries, the
secrets in plain text and the servers given access to the whole disk.

It is the inspector, not the manager: it reads the hosts' configuration files
and never writes to them. No network, no changes, secrets masked everywhere and
never written down.

The design brief, plan and journal are kept in French at the root of this
repository (`BRIEF.md`, `PLAN.md`, `JOURNAL.md`). Code, comments and
documentation are in English.

By the author of [Cleanator](https://github.com/blackberrylabs/cleanator).

## License

MIT — see [LICENSE](LICENSE).
