---
type: Requirement
id: REQ-TRS-PLUGIN-005
name: "A plugin execution failure degrades gracefully and never aborts the rest of validation"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLUGIN-000]
breakdownAdr: Decisions::WasmPluginsADR
tags:
  - plugins
  - validation
---

A configuration mistake (`E530` missing wasm path, `E532` unresolved `foreignFormat:` alias) or a
runtime failure (`W530` load error/trap/panic/timeout, `W532` malformed envelope JSON or
plugin-reported parse diagnostics) shall never abort the rest of `validate`. A failure downgrades
only the affected package's contribution to zero elements plus the relevant finding; every other
native and plugin-originated element in the model validates normally.

## Rationale

Matches the graceful-degradation posture multi-repo composition already established:
`RefState::Unknown` never false-flags drift when it can't be determined, rather than failing the
run. The same posture here means a broken plugin is loudly flagged (and `--deny W530` etc. gates
CI on it immediately) without taking down validation of everything else in the model.

## Scope

- All plugin-related findings are `Finding`s in the normal `validate` report — no separate report
  surface, no special-casing in CLI/MCP/web-server callers.
- A stale/cached last-good result is never silently substituted on failure — a failed run means
  fewer elements that run, not a hidden fallback to previous output.
