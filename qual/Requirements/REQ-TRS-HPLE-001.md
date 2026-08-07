---
id: REQ-TRS-HPLE-001
type: Requirement
name: A Configuration shall accept subConfigurations naming other Configurations, each of which shall resolve and be internally valid
status: draft
reqDomain: software
verificationMethod: test
---

A `Configuration` **shall** accept an optional `subConfigurations:` field naming one or more other
`Configuration` elements — reachable locally or via a `[repos]`-mounted peer repo. Each named
`Configuration` **shall** resolve to a real `Configuration` element (else a dangling-reference
error), that element **shall** itself be a `Configuration` (else a wrong-type error), and it
**shall** itself be internally valid — SAT-satisfiable and free of validation errors (else a
not-internally-valid error). For a peer entry, validity **shall** be established by genuinely
loading and validating that repo's model, not merely by confirming the name exists.

**Source:** `REQ-TRS-HPLE-001` (product model), `ADR-SYS-HPLE-001`.

**Acceptance criteria:** a `Configuration` naming a real, internally-valid `Configuration` (local or
peer) via `subConfigurations:` validates cleanly; a dangling name, a name resolving to a
non-`Configuration` element, and a name resolving to a `Configuration` that is itself SAT-invalid
are each independently reported as errors.
