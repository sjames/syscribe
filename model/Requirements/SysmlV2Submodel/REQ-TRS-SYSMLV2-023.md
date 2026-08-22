---
type: Requirement
id: REQ-TRS-SYSMLV2-023
name: "A SysMLv2 concern def/concern maps to the native ConcernDef/Concern schema — subject, stakeholders"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - views
---

A `concern def`/`concern` usage shall be synthesized into a native `ConcernDef`/`Concern` element
carrying `subject:`/`stakeholders:`, so `ViewpointDef.concerns:`/`RequirementDef.concerns:` — which
already exist as native fields — have something real to eventually reference, and so a
SysMLv2-authored concern participates in browsing/cross-reference the same way a hand-authored one
would (were one to exist — none currently does anywhere in `model/`).

## Rationale

Direct follow-on to `REQ-TRS-SYSMLV2-020`/`-021`: `ElementType::ConcernDef`/`ElementType::Concern`
already exist in the native schema but have never been reachable from either hand-authored content
or SysMLv2 ingestion. Closing that gap is the natural next step after Viewpoint.

## Scope

- The vendored parser (`sysml-v2-parser = "0.54.0"`) has no separate `ConcernDef` struct at all — a
  single `ConcernUsage` AST node parses both `concern def X` and `concern x` textual forms;
  `is_definition: bool` is the sole discriminator. One conversion function branches on it, choosing
  `ElementType::ConcernDef` (`is_definition: true`) or `ElementType::Concern` (`is_definition:
  false`) — unlike Viewpoint, no folding onto a shared usage type is needed, since Syscribe's native
  schema already has both kinds.
- `ConcernUsage.type_name` carries a double meaning the AST itself doesn't disambiguate: it comes
  from the same shared usage-header parse regardless of `is_definition`. For `concern def X : Y`
  this is a supertype; for a bare `concern x : Y` usage it's a typedBy. Exactly one of `supertype`/
  `typedBy` is ever set on the synthesized element, never both, and the choice is driven entirely by
  `is_definition`.
- `ConcernUsage.body` is the exact same `RequirementDefBody` type `ViewpointDef`/`ViewpointUsage`
  already use. `stakeholders:` reuses `REQ-TRS-SYSMLV2-021`'s stakeholder-extraction helper
  (generalized in name only); `ConcernDef` has no `concerns:` self-field per §8.11.5, so the
  `Purpose`-derived half of that helper's return value is discarded here.
- `subject:` is new: `SubjectDecl.type_name` (the typed-declaration form, `subject <name> : <Type>;`)
  is lifted. The bare `subject;` shorthand parses as an empty `SubjectRef` AST node with nothing to
  extract, and stays unmapped — not an oversight, there is genuinely no data on that node.
- `requires:`/`assume:`/`parameters:` (§8.11.5's other `ConcernDef` fields) are explicitly **out of
  scope**. A `RequireConstraint`'s actual constraint content lives nested inside its own
  `ConstraintDefBodyElement` list (an `Expression` needing rendering work comparable to
  `render_expression`), a distinct, unattempted chunk of work — not even done for native
  `Requirement`/`RequirementDef`, which share the same gap. `parameters:` has no native
  `RawFrontmatter` field at all today for any element kind. Tracked as explicit follow-on.
- `ConcernUsage` is reachable **only** from `PackageBodyElement` in this parser version — confirmed
  absent from both `PartDefBodyElement` and `PartUsageBodyElement` (broader than
  `REQ-TRS-SYSMLV2-020`'s View-in-part-usage-only gap, which at least reached `PartDefBodyElement`).
  A `concern`/`concern def` nested inside *any* `part`/`part def` body fails to parse outright,
  degrading via the existing `W541` path (`REQ-TRS-SYSMLV2-006`), not a silent per-kind skip.
- **No new validator check is added by this requirement.** Whether `ViewpointDef.concerns:`/
  `RequirementDef.concerns:` entries should be checked against real `ConcernDef` elements
  (`W500`-style) is a separate design decision: both existing hand-authored Viewpoint files
  (`model/Viewpoints/{SystemsEngineerViewpoint,SafetyEngineerViewpoint}.md`) write `concerns:` as
  free descriptive prose today, not qnames, and a resolution check would immediately fire on that
  real, correct, already-committed content. Making `ConcernDef` real is a prerequisite for that
  future decision, not a mandate to make it now.

**Acceptance criteria:** a package-wrapped `concern def` synthesizes a real `ConcernDef` with
`supertype:` set from its `: Y` clause (never `typedBy:`); a bare `concern` usage synthesizes a real
`Concern` with `typedBy:` set (never `supertype:`); `subject:`/`stakeholders:` lift from a concern
body that declares them; `requires:`/`assume:` stay unset on a fixture that declares
`require`/`assume constraint` members, confirming the descope empirically; a `concern`/`concern def`
nested inside a `part def` body **and** inside a `part` usage body both stay invisible, each raising
`W541`.
