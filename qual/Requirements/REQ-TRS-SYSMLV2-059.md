---
id: REQ-TRS-SYSMLV2-059
type: Requirement
name: "syscribe sysml and sysml_submodels count unresolved package-level satisfy statements and unresolved includes as unmapped"
status: verified
reqDomain: software
verificationMethod: test
---

The per-file and per-submodel unmapped counts reported by `syscribe sysml`/`sysml_submodels` shall equal the counts ingestion raises in `W543`, including package-level `satisfy R by X;` statements whose subject does not resolve and includes that do not resolve (both previously visible only in the advisory, never in the report). The report shall obtain them from the ingestion pass itself rather than re-deriving them.

**Source:** `REQ-TRS-SYSMLV2-059` (product model), `ADR-SYS-SYSMLV2-001`.
