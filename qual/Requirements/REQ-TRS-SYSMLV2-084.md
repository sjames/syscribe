---
id: REQ-TRS-SYSMLV2-084
type: Requirement
name: "A named dependency is ingested and exported as a Dependency element"
status: verified
reqDomain: software
verificationMethod: test
---

`dependency <name> from a, b to c;` shall be ingested as a native `Dependency` element with `clients:` and `suppliers:` (resolved from the scope of the declaring package), and `export-sysml` shall write a native `Dependency` as that statement. An anonymous `dependency from a to b;` has no identity to synthesize an element against and stays unmapped and counted in `W543`.

**Source:** `REQ-TRS-SYSMLV2-084` (product model), `ADR-SYS-SYSMLV2-001`.
