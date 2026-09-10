# CLAUDE.md

@AGENTS.md

`AGENTS.md` is the single source of truth for agent guidance in this repository, and it
applies here in full. This file exists only because Claude Code reads `CLAUDE.md` and does
not read `AGENTS.md`, so the line above imports it.

Put project rules in `AGENTS.md`, not here. Only add something below if it is specific to
Claude Code and would make no sense to a contributor using a different tool.

## Claude Code specifics

- `.claude/settings.local.json` is personal and git-ignored. Do not commit it, and do not
  assume it exists. Another contributor's approved commands are not yours.
- Cargo takes a lock on `target/`, so two cargo commands cannot run at once. Wait for one
  to finish before starting the next.
