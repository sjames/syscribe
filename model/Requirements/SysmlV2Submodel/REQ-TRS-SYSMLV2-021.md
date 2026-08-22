---
type: Requirement
id: REQ-TRS-SYSMLV2-021
name: "A SysMLv2 viewpoint def maps to the native ViewpointDef schema — stakeholders, concerns; a viewpoint usage maps onto View"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - views
---

A `viewpoint def` shall be synthesized into a native `ViewpointDef` element carrying the same
`stakeholders:`/`concerns:` shape a hand-authored one uses
(`model/Viewpoints/SystemsEngineerViewpoint.md`'s existing convention). A `viewpoint` usage shall be
synthesized into a native `View` element — the native schema has no dedicated `Viewpoint` usage
`ElementType`, and `View` is already documented as "usage of a ViewDef or ViewpointDef."

## Rationale

Same traceability-symmetry rationale as `REQ-TRS-SYSMLV2-020` for View. Without this requirement, a
SysMLv2-authored viewpoint — and any `view`/`viewpoint` usage that types against it — is invisible
to the graph, so `W500`'s viewpoint-resolution check can never actually pass for SysMLv2-authored
content.

## Scope

- `ViewpointDef.body`/`ViewpointUsage.body` are both literally `RequirementDefBody` — confirmed
  against the parser's own AST, not a coincidental structural match — the same body type a plain
  `requirement def` uses. `stakeholders:`/`concerns:` come from the same `Stakeholder`/`Purpose`
  variants a `requirement def` already exposes: `StakeholderMember.name` (only — `type_name`/
  `is_redefinition` have no native slot) and `PurposeMember.target` respectively. `Frame`
  (`FrameMember`) and every other `RequirementDefBodyElement` variant stay unmapped — no native
  "framed concern" field exists.
- Doc lifting reuses the shared `collect_doc` core `REQ-TRS-SYSMLV2-009`/`-018`/`-019` already
  established, but needs its own new wrapper (`viewpoint_body_doc`) — `REQ-TRS-SYSMLV2-009`
  deliberately did not extend doc-lifting to `Requirement`/`RequirementDef`/`RequirementUsage`, so
  there was no existing `RequirementDefBody` doc-collector to reuse unchanged.
- `methods:`/`satisfiedBy:` (native schema §8.14.1 fields on `ViewpointDef`) are **not** populated —
  deliberately, not an oversight. No AST source exists at all: the relationship only exists in the
  other direction, as a `view`'s own `satisfy <viewpoint>;` clause (`REQ-TRS-SYSMLV2-020`). Computing
  it here by inverting every view's `satisfy` target would also point the link the wrong way per
  §12.1's OSLC upstream-link-direction rule — the `View` should hold the reference, not the
  `ViewpointDef`.
- `ViewpointUsage.type_name` is a non-`Option<String>` (empty-string sentinel for "untyped"), unlike
  `ViewUsage.type_name`'s `Option<String>` — an empty string is treated as absent when building
  `typedBy:`.
- No recursion into a `viewpoint def`/`viewpoint` usage's own body — `RequirementDefBody`'s
  nested-element variants (`RequirementUsage`, `AttributeDef`, ...) are not walked here, matching
  `convert_requirement_def`'s own existing posture for the identical body type.

**Acceptance criteria:** a package-wrapped `viewpoint def` with `stakeholder`/`purpose` members
synthesizes a real `ViewpointDef` with `stakeholders:`/`concerns:` populated and `methods:`/
`satisfiedBy:` absent; a `viewpoint` usage typed by it synthesizes a real `View` (not a separate
`Viewpoint` type, which doesn't exist); `W500` resolves a synthesized `View`'s `viewpoint:` against a
synthesized `ViewpointDef` exactly as it would hand-authored input.
