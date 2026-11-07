---
type: Requirement
id: REQ-TRS-TRACECODE-003
name: "validate reports untraced code and stale baseline entries when switched on"
status: draft
reqDomain: software
reqClass: system
tags:
  - validation
  - traceability
---

`validate` shall report the trace-code result when the project asks for it (GH #451).

## Behavior

- With `[traceCode] validate = true`, `validate` raises `W326` for each untraced unit missing from the baseline and `W327` for each stale baseline entry; both are warnings, gateable with `--deny`.
- A malformed `[traceCode]` entry (unknown key, unknown unit kind, wrong value type, missing roots) or an unreadable baseline is `W328`; the entry is ignored. `W328` is reported whenever the table is present.
- Without a `[traceCode]` table nothing runs and no finding is raised.
