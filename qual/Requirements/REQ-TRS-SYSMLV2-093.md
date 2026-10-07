---
id: REQ-TRS-SYSMLV2-093
type: Requirement
name: "An anonymous dependency is ingested under a synthesized name"
status: verified
reqDomain: software
verificationMethod: test
---

An anonymous `dependency from a to b;` shall be ingested as a `Dependency` element named `dependency_N` — N counting from 1 in source order within the owning scope, skipping a name already taken — with `clients:`/`suppliers:` resolved exactly as for a named one, so it no longer counts in `W543`; `export-sysml` writes the named form.

**Source:** `REQ-TRS-SYSMLV2-093` (product model), `ADR-SYS-SYSMLV2-001`.
