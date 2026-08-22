---
type: Requirement
id: REQ-TRS-SYSMLV2-027
name: "A SysMLv2 analysis def/analysis maps to the native AnalysisCaseDef/AnalysisCase schema — subject, actors, objectives, result"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - cases
---

An `analysis def` shall be synthesized into a native `AnalysisCaseDef` element carrying
`supertype:`/`subject:`/`actors:`/`objectives:`/`result:`/`isAbstract:`/`doc`. A named `analysis`
usage shall be synthesized into a native `AnalysisCase` element carrying the same fields with
`typedBy:` in place of `supertype:`.

## Rationale

Sibling of `REQ-TRS-SYSMLV2-026`, sharing its `case_body_fields` mechanism entirely — `AnalysisCase`
specializes `Case` in real SysMLv2, reflected here by the identical AST body type
(`UseCaseDefBody`) and field extraction.

## Scope

- Same body-extraction mechanism, same `Spec` fields, same `is_abstract` mapping, and the same
  `verifies:`/verdict-field non-derivability as `REQ-TRS-SYSMLV2-026` — `AnalysisCaseDef` has no such
  fields in its own §8.12.2 schema anyway.
- **The one real reachability difference from `CaseDef`/`VerificationCaseDef` in this family**:
  `AnalysisCaseDef`/`AnalysisCaseUsage` are the *only* two of the six case-family kinds present in
  `PartUsageBodyElement` — confirmed directly against the AST. An `analysis def`/`analysis` nested
  directly inside a `part` usage body is therefore reachable and mapped, unlike `case`/`verification`
  in the same position.
- No new validator check, same rationale as `REQ-TRS-SYSMLV2-026`.

**Acceptance criteria:** a package-wrapped `analysis def` synthesizes a real `AnalysisCaseDef` with
`subject:`/`actors:`/`objectives:`/`result:`/`isAbstract:`/`doc` populated; a named `analysis` usage
synthesizes a real `AnalysisCase` with `typedBy:`; both kinds are reachable at package level, nested
inside a `part def` body, *and* nested inside a `part` usage body — the only case-family kind
reachable in all three positions.
