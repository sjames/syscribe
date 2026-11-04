---
type: Requirement
id: REQ-TRS-AGENTGUIDE-001
name: "agent-guide generates a project-specific agent instruction block from the model"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - agents
---

`syscribe agent-guide` shall print the guidance an automated editor needs for this model, built from the loaded model and `.syscribe.toml`, and shall keep a marked region of a hand-written file in step with it (GH #441).

## Behavior

- Sections `ids` (the id prefix of each type in use plus configured `[ids.prefixes]` extras, with the `next-id` command), `types` (per lifecycle type: status vocabulary, required fields, working commands), `gates` (the `validate` commands to run, one per `[profiles.<name>]`, and the `[gates]` they enforce) and `rules` (safe mutation: `next-id`/`template`, `set`/`move`/`claim`/`release`, `--dry-run`). `--sections` selects a subset; `--format json` emits the structure.
- Output is deterministic: no timestamps and no tool version, so it changes only when the model or its configuration does.
- `--write <file>` replaces only the `<!-- syscribe:agent-guide -->` … `<!-- /syscribe:agent-guide -->` region, creating the file or appending the region when absent; text outside the region is never modified. Repeated or unbalanced markers are refused and the file is left untouched.
- `--check <file>` exits `1` when the region is missing or differs from the generated block; `0` when current.
- Every command the guide names exists.
