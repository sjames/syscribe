# Show HN / r/rust draft

Not posted. Fill in the `<…>` placeholders once the demo is recorded (see `demo/README.md`).

## Title options

- Show HN: Guarded MCP writes so an LLM agent can't corrupt structured project state
- Show HN: Syscribe – Markdown models your AI agent can edit, dry-run and validated before commit
- (r/rust) Syscribe: a single-binary MCP server that dry-runs every LLM write against a validator

## Body

LLM agents are good at editing Markdown and bad at knowing when they've broken something that isn't in the file
they're looking at: a requirement that traces to an ID that doesn't exist, a test that verifies a deleted spec, a
rename that leaves dangling links. Syscribe is my attempt at making that class of mistake hard to commit.

Project state (requirements, architecture, tests, decisions, work items) lives as plain Markdown files with YAML
frontmatter in your git repo. `syscribe mcp` exposes it to an agent over the Model Context Protocol. Every write
tool (`create_element`, `update_element`, `move_element`, `delete_element`, `apply_changes`) defaults to a dry run
that returns the *validation delta* — every new or resolved warning, plus errors for dangling references and
violated link-type rules — and a commit that would introduce an unresolved reference is refused. (Other validator
errors show up in a full `validate`; the delta is deliberately narrower.) So the loop is: agent proposes, sees what it
would break, fixes it, commits. The result is a normal file diff you review in git. `--read-only` hides the write tools
entirely.

Demo (real output from the bundled ISO 26262 example model): <asciinema/GIF link>

The validator underneath comes from safety-critical engineering — it's a Markdown/YAML rendering of a subset of
SysMLv2 with 200+ rules (traceability, ASIL/SIL consistency, release baselines) — and the repo ships automotive and
railway demo models. But the write guard doesn't care about the domain; the same dry-run → delta → commit gate would
apply to any structured state with a validator.

Written in Rust, mostly because a single static binary is the right shape for something an agent runtime spawns over
stdio: `curl` it, `claude mcp add` it, done. No network access, no daemon.

Repo: https://github.com/sjames/syscribe
Docs: https://sjames.github.io/syscribe

Things I'd like feedback on: whether "refuse commits that add new errors" is the right default strictness (there's an
env override), and what other guard signals an agent should get back besides the validation delta.
