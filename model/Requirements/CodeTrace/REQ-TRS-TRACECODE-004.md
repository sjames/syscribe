---
type: Requirement
id: REQ-TRS-TRACECODE-004
name: "trace-code checks each composed repository against its own elements"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - multi-repo
---

With `[repos]` configured, a component's code shall be checked against the component's own model (GH #451).

## Behavior

- `trace-code` also checks every `[repos]` peer that has its own `[traceCode]` table, using the peer's elements, the peer's repository root and the peer's baseline, and reports it separately (`peers` in `--json`). An element of the composing model that names peer code does not trace it.
- A peer without a `[traceCode]` table, or not on disk, is listed as not checked and does not change the exit code; a peer whose table is malformed or whose model cannot be loaded is a configuration error (exit `1`). A new untraced unit or stale entry in a peer exits `2`.
- `--write-baseline` writes only the composing model's baseline.
- `validate` (`W326`/`W327`) checks only the model being validated; peers are validated in their own repository.
