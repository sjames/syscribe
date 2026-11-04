---
type: Requirement
id: REQ-TRS-HELPDISC-001
name: "help finds commands by element type, keyword and task"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - help
---

`syscribe help` shall answer "what can I do with this type", "which command does X" and "how do I do this job", without a model directory (GH #438).

## Behavior

- `help <Type>` (any element type, case-insensitive) prints the type's core fields with required/optional and enum domains, a skeleton, the validation-catalogue rows that name it, and every command that reads or changes it. The command list is derived from the man pages (every page that names the type, excluding its SEE ALSO block) plus the commands that act on any element, so it cannot go stale. `help PlanningItem` lists `claim` and `release`.
- `help --search <kw>` (`-k`) ranks command names, synopses, option text, type names and spec sections case-insensitively, printing the matching line. All words of a multi-word keyword must match. `help --search evidence` returns at least `set`, `ingest-results` and `results`. No keyword exits `1`.
- `help tasks` maps common intents to commands.
- A mistyped topic exits `1` and lists the three nearest commands with their one-line purpose.
- MCP `describe_type` returns the same `commands` and `codes` with the field schema.
- `template <Type>` needs no model: a missing model directory still prints the skeleton.
