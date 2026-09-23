---
id: REQ-TRS-LINKTYPE-008
type: Requirement
name: "A link-types command shall list the declared link-type vocabulary and its rules"
status: draft
reqDomain: software
verificationMethod: test
---

The CLI **shall** provide `syscribe -m <root> link-types [--json]`, listing every valid
declared link type with its description, inverse, extends base, relaxed codes, coverage,
source/target types, cardinality, acyclic and suspect settings, and the number of link
instances in the model. With no link types declared it **shall** say so, show how to declare
one, and exit zero.

**Acceptance criteria:** every declared type and its rules appear in text and JSON; an invalid
(`W630`) entry is not listed as usable; an unconfigured model prints the "none declared" hint.

**Source:** `REQ-TRS-LINKTYPE-008` (product model), `ADR-SYS-LINKTYPE-001`.
