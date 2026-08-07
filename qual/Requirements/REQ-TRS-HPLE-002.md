---
id: REQ-TRS-HPLE-002
type: Requirement
name: parameterBindings shall reach transitively into a subConfigurations-consolidated subtree via ordinary qname resolution
status: draft
reqDomain: software
verificationMethod: test
---

`Configuration.parameterBindings:` (a flat map keyed by the dotted `<FeatureDef qname>.<param>`
reference) **shall** be able to bind a parameter belonging to any `FeatureDef` reachable through
`subConfigurations:`, at any depth — not only this `Configuration`'s own local features — using
the parameter's ordinary, already-mounted qualified name. No new field and no new cross-repo
addressing syntax **shall** be introduced.

**Source:** `REQ-TRS-HPLE-002` (product model), `ADR-SYS-HPLE-001`.

**Acceptance criteria:** a `parameterBindings:` entry targeting a parameter reachable only through
`subConfigurations:` resolves and applies exactly like one targeting a purely local `FeatureDef`
parameter, at any depth, using the parameter's ordinary qname; a dotted key naming a `FeatureDef`
unreachable by any means is still reported as an unresolved reference.
