---
type: Requirement
id: REQ-TRS-SCHEMA-002
name: "reqClass is a recognized Requirement classification field"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - schema
  - requirements
---

The `reqClass` field on a `Requirement` shall be a **recognised, first-class**
frontmatter field. It carries the requirement's position in the stakeholder/system
decomposition and takes one of `stakeholder`, `system`, or `derived`.

As a recognised field, `reqClass` shall be parsed into the element model, preserved on
round-trip (frontmatter-only writes), and shall **never** raise the unrecognised-field
warning of REQ-TRS-SCHEMA-001.

## Rationale

`reqClass` is already an established part of the authoring convention — the requirement
authoring workflow uses it to distinguish stakeholder-level intent from system-level and
derived requirements, and the MCP write path already offers it as a Requirement field
with a fixed value set. Despite this, the parser did not recognise it: every `reqClass:`
declaration was silently absorbed by the unknown-field catch-all. Once unrecognised
fields warn (REQ-TRS-SCHEMA-001), a field this widely and deliberately used must be
promoted to the recognised schema rather than flagged — both to keep valid models clean
and to make the classification available to the model, spec, and tooling as a
first-class datum.
