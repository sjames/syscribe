---
type: Requirement
id: REQ-TRS-TCELEM-001
name: "A TestCase shall be able to verify architecture elements directly through verifiesElement, and an untested active architecture element shall be reported"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - traceability
  - testcase
---

A `TestCase` **shall** accept an optional `verifiesElement:` field (string or list) naming
`PartDef`, `InterfaceDef`, `ActionDef` or `StateDef` elements by qualified name or stable id,
resolved like every other reference. An unresolved entry **shall** raise `E102`; an entry that
resolves to any other element kind **shall** raise `E129`.

An active (not `draft`/`deprecated`/`retired`/`obsolete`/`rejected`/`superseded`), non-abstract
`PartDef`, `InterfaceDef`, `ActionDef` or `StateDef` that is named by no active `TestCase`'s
`verifiesElement:`, and that `satisfies:` no requirement with an active verifying `TestCase`,
**shall** raise warning `W104`. An element carrying a non-empty `testExempt:` rationale **shall**
not raise `W104`.

`verifiesElement:` entries **shall** take part in `appliesWhen` escape analysis exactly like
`verifies:`: an active `TestCase` naming an element inactive in the projected variant raises `W019`.

**Acceptance criteria:** a resolving link clears `W104`; a satisfied, verified requirement clears
`W104`; `testExempt:` suppresses it; a draft element never raises it; a bad target raises
`E129`/`E102`; the `W019` escape is raised in a variant lens; a model using none of this gains
no finding other than `W104` on genuinely untested active elements.
