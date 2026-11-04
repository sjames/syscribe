---
type: ADR
id: ADR-SYS-PROVENANCE-001
name: "Provenance of tool changes lives in git commits, not in a separate log file"
status: accepted
tags:
  - cli
  - git
  - agents
---

## Context

GH #443 asked for "who changed this, and why" to be recorded for every writing command: `--by` and
`--reason` on each, and an append-only `.syscribe/changes.jsonl` with one line per change. GH #444 asked
for a global `--dry-run`, a read-only MCP mode, stable exit codes and an `--expect-hash` guard against
stale edits.

The project's agents are meant to work on the mainline together, not on branches that must later be
reconciled. That makes the question concrete: several actors change one model, each change should be
attributable, and none should silently overwrite another's.

## Decision

**Git is the only record.** No new log file is introduced.

1. **`--by <actor>` and `--reason <text>` are global flags**, accepted by every command and read before
   dispatch. `--by` is optional everywhere; it is the actor's id (letters, digits, `.`, `_`, `-`).
2. **`--commit` (or `[provenance] commit = true` in `.syscribe.toml`) makes a writing command commit
   what it changed**, and only that: the files the command wrote, named explicitly, so another actor's
   staged or unstaged work in the same working tree is never swept in. With `--by` the actor is the commit
   *author* (the committer stays the configured git user); without it the configured identity is used and
   no actor trailer is written. The message carries `Syscribe-Command`, `Syscribe-Actor` and
   `Syscribe-Reason` trailers. Committing is opt-in because most uses of `set` are a person at a terminal
   who should not get a commit they did not ask for.
3. **`syscribe log [<element>]` reads the history back** from git: for the model, the commits that touched
   it; for one element, the commits that touched its file (through renames) and, per commit, what changed
   in it (status transition, links, evidence) using the same element-level comparison as `changes`.
   `--unattributed` lists commits that touched the model without the trailers — the hand edits — which
   replaces the issue's `validate --changes`.
4. **A global `--dry-run` is implemented once, in a write layer** (`syscribe_model::fsio`) that every
   user-facing file write goes through: under `--dry-run` a write is recorded as "would write" and nothing
   touches the disk. Each of the writing commands is tested to change the tree for real and to leave it
   byte-identical under `--dry-run`.
5. **`--expect-hash <hash>` hashes the whole file** (BLAKE3 of its bytes). It is exactly what a reader saw,
   so any concurrent change — body or frontmatter — makes the edit fail. `show` and the MCP `get_element`
   detail expose `contentHash` to pass back.
6. **Exit codes stay `0` success, `1` usage error or a refused write, `2` a tripped gate.** There is no
   exit `3` for "refused by policy or conflict": a refusal is a refusal. `connectivity`, `follow` and
   `impact` already exit `2` on a usage error and are documented as exceptions.

## Rejected alternatives

- **A single `.syscribe/changes.jsonl`** (the issue's proposal). Every actor appends to one file, so two
  branches or two working copies that both append collide in git despite the line-per-record format, parallel
  writers need a cross-process lock, and it only ever grows. It also duplicates what git already holds and
  can drift from it.
- **A sharded log** — per month, per actor, or per actor and month. Sharding fixes the contention but leaves
  the real objection: a second record of something git already records. It was rejected once the decision
  was made that agents work on the mainline and everything is stored in git.
- **Provenance in commit messages the agent writes by hand.** Easy to forget and impossible to query; a
  trailer written by the tool is uniform and `log` can rely on it.
- **A commit per change by default.** Surprising for a person; the opt-in flag and config switch give agent
  projects the behaviour without imposing it.
- **Hashing only the frontmatter for `--expect-hash`.** It would let a body edit made since the read slip
  through unnoticed.

## Consequences

- Provenance exists only for changes made with `--commit`. An edit made by hand, or by a tool run without
  it, appears in `log --unattributed` — which is the point, not a gap.
- Committing is at file granularity: if two actors edit the same file before either commits, the first
  commit takes both edits.
- `--expect-hash` is a check followed by a write, not a lock. It catches an edit made since the file was read,
  which is the case it exists for; it cannot stop another process writing in the instant between the check and
  the write. Within one MCP server writes are serialised by the server, so there the window does not exist.
  It is accepted only by the commands that check it (`set`, `claim`, `release`, `move`, `applies-when`,
  `scaffold-gherkin`; over MCP `update_element`, `move_element`, `delete_element`, `question`) and is a usage
  error anywhere else, so it can never be silently ignored.
- `claim --by` stays an opaque claimant id (free-form, as before); an actor id is only held to the safe
  character set when it is also used as a commit author.
- Nothing here is tamper-proof; anyone can rewrite history or forge a trailer. An audit trail that cannot be
  edited would need signed commits, which is out of scope.
- `apply_changes` (the MCP batch tool) does not take the provenance arguments yet.
- The `--record` family (`simulate`, `mutate`, `inject`) writes through the same layer and so honours
  `--dry-run`, but is not covered by the dry-run test: the bundled example models have no scenarios for it
  to run.
