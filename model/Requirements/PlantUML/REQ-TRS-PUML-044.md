---
id: REQ-TRS-PUML-044
name: "Every shape with a ref emits a clickable URL annotation"
type: Requirement
status: draft
reqClass: derived
reqDomain: software
derivedFrom: REQ-TRS-PUML-043
breakdownAdr: Decisions::PlantUMLADR
tags: [diagram, plantuml, config]
---

In every generated `.puml` diagram, each shape declaration that carries a `ref` field
must include a PlantUML URL annotation immediately after the identifier (and stereotype
if present) using the syntax `[[URL]]`:

- Class diagram (BDD / Requirement): `class "Name" as id <<stereo>> [[URL]]`
- Component diagram (IBD blocks and boundaries): `component "Name" as id [[URL]]`
  and `rectangle "Name" as id [[URL]] {`
- State diagram: `state "Name" as id [[URL]]`
- Sequence diagram participants/actors: `participant "Name" as id [[URL]]`

The `URL` is `<base_url>/ui/detail/<ref>` where `<ref>` is the value of the shape's
`ref` frontmatter field (the qualified name of the referenced model element).

Shapes that carry no `ref` (e.g. initial pseudo-states in StateMachine) emit no URL annotation.
