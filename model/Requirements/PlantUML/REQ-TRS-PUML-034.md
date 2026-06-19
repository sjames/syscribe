---
type: Requirement
id: REQ-TRS-PUML-034
name: "Existing Diagram validation rules continue to apply to pumlMode companion elements"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PUML-001]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
  - validation
---

A `Diagram` element that sets `pumlMode: companion` shall remain subject to all existing
diagram validation rules without exemption or relaxation.

## Rules that continue to apply unchanged

- **W401** (unresolved `subject:`) — applies whenever the `subject:` qualified name cannot
  be resolved in the loaded element set, regardless of `pumlMode`.
- **W402** (unresolved shape `ref:`) — applies to every shape in the `shapes:` map whose
  `ref:` qualified name cannot be resolved.
- **W411** (unresolved shape `link:`) — applies when one or more shapes carry a `link:`
  value that cannot be resolved.
- **W412** (unresolved `href` in SVG body) — applies if the element's markdown body
  contains inline SVG with `href` attributes that cannot be resolved.

## Effect of pumlMode on the validation pass

The `pumlMode` field has a single, additive effect on the validation pass: it enables the
new rules W413, W414, E403, and E404 (REQ-TRS-PUML-030 through REQ-TRS-PUML-033).
It does not suppress, relax, or modify any pre-existing diagram validation rule.

## Rationale

Keeping the existing rules active ensures that a `pumlMode: companion` element is still
checked for broken cross-references, dangling links, and inline-SVG hygiene. Suppressing
those checks for PlantUML companion elements would create a validation blind spot for
authors who mix `pumlMode` with shapes or SVG bodies.
