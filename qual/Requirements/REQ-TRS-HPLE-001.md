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

A peer entry **shall** resolve by the peer's native qualified name, by global stable id, **or**
through a `repoImports:` mount path (`<package>::<as>::X` → the peer's `<qname>::X`, §14.4) exactly
as the other cross-repo reference fields do; a mount path naming nothing in its peer is a
dangling-reference error (GH #146).

A consolidated `Configuration` **shall** be judged — for validity, for the parameters it already
closes, and for what it selects — on its **effective** selection and bindings, including those it
inherits from a base through `derivedFrom:` (spec §9.8, `REQ-TRS-VAR-007`).

**Source:** `REQ-TRS-HPLE-001` (product model), `ADR-SYS-HPLE-001`.

**Acceptance criteria:** a `Configuration` naming a real, internally-valid `Configuration` (local or
peer) via `subConfigurations:` validates cleanly; a dangling name, a name resolving to a
non-`Configuration` element, and a name resolving to a `Configuration` that is itself SAT-invalid
are each independently reported as errors; a `subConfigurations:` entry written through a
`repoImports:` mount path resolves and is validity-checked in that peer (no `E516`), while a mount
path naming no element of the peer raises `E516`.
