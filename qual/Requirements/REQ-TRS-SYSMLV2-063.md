---
id: REQ-TRS-SYSMLV2-063
type: Requirement
name: "Round-trip tests verify cross-entry consistency of exported behaviour bodies"
status: verified
reqDomain: software
verificationMethod: test
---

The export parse-back tests shall check, beyond per-entry equality, that every emitted \`first … then …;\` names only steps that are exported in the same body, and that re-ingesting a whole exported body yields successions and control nodes equal to the exported subset, so entry-level verification cannot hide a dangling reference between entries.

**Source:** `REQ-TRS-SYSMLV2-063` (product model), `ADR-SYS-SYSMLV2-001`.
