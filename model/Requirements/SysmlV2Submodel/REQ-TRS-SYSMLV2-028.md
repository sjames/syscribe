---
type: Requirement
id: REQ-TRS-SYSMLV2-028
name: "A SysMLv2 verification def/verification maps to the native VerificationCaseDef/VerificationCase schema — subject, actors, objectives, result"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - cases
---

A `verification def` shall be synthesized into a native `VerificationCaseDef` element carrying
`supertype:`/`subject:`/`actors:`/`objectives:`/`result:`/`isAbstract:`/`doc`. A named
`verification` usage shall be synthesized into a native `VerificationCase` element carrying the same
fields with `typedBy:` in place of `supertype:`.

## Rationale

Sibling of `REQ-TRS-SYSMLV2-026`/`-027`, sharing the `case_body_fields` mechanism entirely.

## Scope

- Same body-extraction mechanism, same `Spec` fields, same `is_abstract` mapping as
  `REQ-TRS-SYSMLV2-026`/`-027`.
- **`verifies:`/`verdictExpression:`/`verdictType:`** — §8.12.3's `VerificationCaseDef`-specific
  fields — are **deliberately not populated**. `UseCaseDefBodyElement` (the shared case-family body
  element enum) carries no verify-statement or verdict-semantics variant at all: the closest thing,
  `RequirementUsage`, is documented in the parser's own source as a "directed `in requirement …`
  parameter," not a `verify <target>;` statement. This lines up with the vendored crate's own
  compliance-doc caveat that the case family "still operate\[s\] as subset parsers... simplified
  handling of nested forms" — a real upstream ceiling, not a Syscribe choice. `returnType:` (which
  spec §8.12.3 documents as defaulting to `verdictType:` if absent) is populated the same way as
  `CaseDef`'s `result:` — from the first `CaseReturnDecl.type_name` — since the AST gives no way to
  distinguish a verdict-typed return from any other.
- `PartUsageBodyElement` carries **no** `VerificationCaseDef`/`VerificationCaseUsage` variant —
  confirmed absent, same as `CaseDef`/`CaseUsage`. A `verification`/`verification def` declared
  directly inside a `part` usage body fails to parse outright, degrading to `W541`.
- No new validator check, same rationale as `REQ-TRS-SYSMLV2-026`.

**Acceptance criteria:** a package-wrapped `verification def` with multiple `return` declarations
synthesizes a real `VerificationCaseDef` with `result:` set from the first typed one,
`subject:`/`actors:`/`objectives:`/`isAbstract:`/`doc` populated, and `verifies:`/
`verdictExpression:`/`verdictType:` confirmed absent (explicit descope check); a named `verification`
usage synthesizes a real `VerificationCase` with `typedBy:`; a `verification`/`verification def`
nested inside a `part` usage body stays invisible, raising `W541`.
