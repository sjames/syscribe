---
type: Requirement
id: REQ-TRS-PROVLOG-001
name: "Tool changes can be attributed and committed to git, and read back with log"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - git
---

A writing command shall be able to record who made a change and why in a git commit, and `syscribe log` shall read that history back (GH #443, `ADR-SYS-PROVENANCE-001`).

## Behavior

- `--by <actor>`, `--reason <text>`, `--commit` and `--expect-hash` are global: accepted by every command, before or after its name. `--by` is always optional; an actor id outside letters, digits, `.`, `_`, `-` (at most 64) is a usage error. `claim` and `log` keep their own meaning of `--by` (the claimant; a filter).
- With `--commit`, or `[provenance] commit = true`, the command commits the files it wrote and only those — never another actor's staged or unstaged work — as one commit per command. With `--by` the actor is the commit author; without it the configured git identity is used. The message is `syscribe: <command>`, the reason, and trailers `Syscribe-Command`, `Syscribe-Actor` (only when given) and `Syscribe-Reason` (only when given). A dry run never commits. A commit that fails leaves the change written, says so, and exits 1.
- `log` lists the commits that touched the model; `log <element>` follows the element's file through renames and says what each commit changed in it. `--by`, `--since`, `--limit` and `--json` filter and format; `--unattributed` lists commits without syscribe's trailers (hand edits). Uncommitted changes are reported separately. A model outside git, or an unknown element, exits 1.
- The MCP write tools `create_element`, `update_element`, `move_element`, `delete_element` and `question` take `by`, `reason`, `commit` and `expect_hash`; a committed write returns `commit`. A read tool `log` returns the same history.
- `question.answer` records the actor as `answeredBy` when `by=` is not given.
