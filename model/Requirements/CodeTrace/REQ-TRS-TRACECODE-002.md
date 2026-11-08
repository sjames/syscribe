---
type: Requirement
id: REQ-TRS-TRACECODE-002
name: "The trace-code baseline only shrinks"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - traceability
---

Known untraced code shall be recorded in a baseline with a reason, and the baseline shall only shrink (GH #451).

## Behavior

- The baseline file (`[traceCode] baseline`, default `trace-code-baseline.toml`, relative to the model root) is a list of `[[untraced]]` tables with `path` and `reason`. A missing file is an empty baseline.
- `trace-code` exits `2` when an untraced unit is not in the baseline, or when a baseline entry is stale: now traced, no longer existing, or no longer a unit under the rules. Otherwise it exits `0`.
- `--write-baseline` writes the baseline from the current state: existing reasons are kept, new entries get `reason = "TODO"`, stale entries are dropped. The write goes through the guarded write layer, so the agent policy's `protected_paths` refuses it for an agent (exit 1).
