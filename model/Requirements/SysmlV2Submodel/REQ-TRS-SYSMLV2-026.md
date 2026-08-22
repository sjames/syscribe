---
type: Requirement
id: REQ-TRS-SYSMLV2-026
name: "A SysMLv2 case def/case maps to the native CaseDef/Case schema — subject, actors, objectives, result"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - cases
---

A `case def` shall be synthesized into a native `CaseDef` element carrying `supertype:`/`subject:`/
`actors:`/`objectives:`/`result:`/`isAbstract:`/`doc`. A named `case` usage shall be synthesized into
a native `Case` element carrying the same fields with `typedBy:` in place of `supertype:`.

## Rationale

`ElementType::CaseDef`/`ElementType::Case` already existed in the native schema, with every field
§8.12.1's common case-fields table documents already on `RawFrontmatter`, but were unreachable from
SysMLv2 ingestion. `CaseDef`/`CaseUsage` are already reachable from `PackageBodyElement` and
`PartDefBodyElement` — no parser-level ceiling blocks the base mapping there, though (see Scope)
`PartUsageBodyElement` carries no variant for either.

## Scope

- `CaseDef`/`CaseUsage`, along with `AnalysisCaseDef`/`AnalysisCaseUsage` and
  `VerificationCaseDef`/`VerificationCaseUsage`, share exactly one AST body type: `UseCaseDefBody` —
  confirmed directly against the parser's own AST, not `RequirementDefBody` like Concern/Viewpoint. A
  shared `case_body_fields` helper extracts `subject:` (`SubjectDecl.type_name`), `actors:`
  (`ActorUsage.type_name`, one per member), `objectives:` (`Objective.requirement`'s own name,
  falling back to its type when anonymous, one plain string per member), `result:`
  (`CaseReturnDecl.type_name`, first one wins — a single-string field, multiple `return` declarations
  are legal per real fixtures), and `doc` (the body's direct `Doc` variant — confirmed present, unlike
  `EnumerationBody`'s absence).
- `is_abstract` is a plain bool field directly on both `CaseDef` and `CaseUsage`, mapped onto the
  existing generic `RawFrontmatter.is_abstract`.
- **Out of scope, no AST source**: `verifies:`/`verdictExpression:`/`verdictType:` are
  `VerificationCaseDef`-specific fields, not part of `CaseDef`'s own schema — not applicable here at
  all (see `REQ-TRS-SYSMLV2-028` for their treatment on `VerificationCaseDef`, where they are also
  out of scope).
- `PartUsageBodyElement` carries **no** `CaseDef`/`CaseUsage` variant at all — confirmed absent from
  that enum. A `case`/`case def` declared directly inside a `part` *usage* body fails to parse
  outright, degrading to `W541`, not a silent per-kind skip.
- No new validator check — `CaseDef`/`Case` have zero validation rules today, for any origin,
  unrelated to this mapping (see the ADR addendum).

**Acceptance criteria:** a package-wrapped `case def` synthesizes a real `CaseDef` with `subject:`/
`actors:`/`objectives:`/`result:`/`isAbstract:`/`doc` all populated from a fixture exercising each; a
named `case` usage synthesizes a real `Case` with `typedBy:`; a `case`/`case def` nested inside a
`part` usage body stays invisible, raising `W541`.
