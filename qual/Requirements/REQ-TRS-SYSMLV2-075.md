---
id: REQ-TRS-SYSMLV2-075
type: Requirement
name: "Use case include accepts qualified targets and the declaring form"
status: verified
reqDomain: software
verificationMethod: test
---

In a use case body, `include <qualified::Name>;` and the declaring form `include use case v : <qualified::Name>;` shall add the resolved qualified name of the referenced use case to `includes:` exactly as the simple-name reference form does (innermost scope first, an unresolved target counted in `W543` as `include`).

**Source:** `REQ-TRS-SYSMLV2-075` (product model), `ADR-SYS-SYSMLV2-001`.
