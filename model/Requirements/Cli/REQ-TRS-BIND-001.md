---
type: Requirement
id: REQ-TRS-BIND-001
name: "bind writes one checked binding or selection into a Configuration, previewed first"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

`syscribe bind <Configuration> <feature | feature.parameter> <value>` and the MCP tool `bind` shall write one `parameterBindings:` value or `features:` selection into a Configuration, checked against the parameter's range, enumeration, unit, stage window and narrowing before anything is written (GH #418).

- The change is previewed on a candidate copy by the guarded-write engine; the checks are the validator's own, so the result is what `validate` would say. The report carries the value replaced and its source, the diff, and the errors and warnings added or resolved, each finding once.
- A change that adds an error is refused and nothing is written (CLI exit 1; the MCP tool reports `written: false`). A clean change is written by editing only the affected lines, reusing an existing entry's spelling; `--dry-run` (the MCP default) previews only.
- An unknown target is refused with a suggestion. The CLI report has the published `bind` schema; the MCP tool returns the standard guarded-write result with the same `target`, `previous` and `stage`.
