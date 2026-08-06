---
type: Requirement
id: REQ-TRS-SYSMLV2-006
name: "A SysMLv2 ingestion failure degrades gracefully, under its own error/warning code range, and never aborts validation"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - validation
---

A malformed `sysmlSubmodel: true` declaration, a `.sysml`/`.kerml` parse failure, or an unmapped
construct shall never abort the rest of `validate`. Each failure downgrades only the affected
file's (or subtree's) contribution — fewer or no elements from that file, plus a `Finding` in the
normal validate report — while every other native and SysMLv2-originated element in the model
validates normally. This subsystem uses its own dedicated error/warning code range, distinct from
the WASM-plugin family (`E530`–`E532`/`W530`–`W534`), since it is not a plugin.

## Rationale

Matches the graceful-degradation posture already established for multi-repository composition
(`RefState::Unknown`) and for WASM plugins (`ADR-SYS-PLUGIN-001`) — a broken subtree is loudly
flagged, and gateable via `--deny`, without taking down validation of everything else in the model.
A dedicated code range keeps diagnostics honestly labeled: these are native-parser/mapping
failures, not plugin-execution failures, and conflating the two ranges would misattribute the
failure mode to anyone grepping a validation report.

## Scope

- All SysMLv2-related findings are ordinary `Finding`s in the normal `validate` report — no
  separate report surface, no special-casing in CLI/MCP/web-server callers.
- Exact code numbers are an implementation detail assigned against the live `E`/`W` registry at
  build time, not fixed by this requirement.
- A stale/cached last-good result is never silently substituted on failure — a failed parse means
  fewer elements from that run, not a hidden fallback to previous output.
