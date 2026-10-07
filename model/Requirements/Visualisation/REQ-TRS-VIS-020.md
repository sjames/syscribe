---
type: Requirement
id: REQ-TRS-VIS-020
name: "The Requirement generator derives requirements with their derive, refine, satisfy and verify relationships from a Package or requirement subject"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-015]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - requirement
---

For a derived `diagramKind: Requirement` whose subject is a `Package` (or `LibraryPackage`/
`Namespace`), `RequirementDef` or `Requirement`, the generator shall produce:

- one `Requirement` node per native `Requirement`, SysML `RequirementDef` or `Requirement` whose
  qualified name is the subject or lies under it (any depth), labelled by `name` (or id), with
  compartment lines for the stable `id` and `status` when present;
- a `Derive` edge from each requirement to each `derivedFrom:` target on the diagram and a
  `Refine` edge for each `refines:` target;
- a `Satisfy` edge from a `Block` context node (the satisfying architecture element, drawn
  with its real type's stereotype) to the requirement for every element in the model whose
  `satisfies:` names a requirement on the diagram, and a `Verify` edge from a `TestCase` context
  node for every `TestCase` whose `verifies:` names one;
- a `Containment` edge from a `RequirementDef` to each requirement it owns, when both are on the
  diagram.

`include:`/`exclude:` apply to requirements and to context nodes alike (by qualified name, stable
id or short name). Shape ids are `derived_shape_id(<qualified name>)`. Layout hints: layered,
top-to-bottom, with `Derive`, `Satisfy`, `Verify` and `Refine` reversed so parents sit above.

## Rationale

The requirement tree with its satisfy and verify legs is the V-model picture every review
wants, and `REQ-TRS-DIAG-*`'s hand-listed `SafetyRequirementsD` goes stale the moment a
requirement is added.

## Scope

- Suspect-link state (`W090`) and status colours are not drawn in this cut.
- Very large packages are expected to be narrowed with `include:`; no automatic depth cap.
