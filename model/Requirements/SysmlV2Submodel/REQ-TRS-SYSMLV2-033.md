---
type: Requirement
id: REQ-TRS-SYSMLV2-033
name: "A SysMLv2 constraint def/constraint maps to the native ConstraintDef/Constraint, carrying parameters, an opaque expression string, supertype/typedBy and doc"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - constraint
---

A `constraint def` shall be synthesized into a native `ConstraintDef` and a named `constraint`
usage into a native `Constraint`, at package level, and (usage only) nested in a `part def` body
and a `part` usage body. Both carry:

- `supertype:` (`constraint def X : Y`, the `:` of a definition is a specialization) or
  `typedBy:` (`constraint x : Y`, a usage's typing) — exactly one, never both;
- `parameters:` — each `in`/`out`/`inout` declaration as a `{name, typedBy, direction}` map;
- `expression:` — the body's expression(s), rendered to text and kept as an **opaque string**
  (several expressions are joined by newlines); absent for a bodyless constraint;
- the body's `doc /* ... */` as the element's doc text.

Nested `constraint` members of a constraint body, and the `assert`/negation prefixes (which the
parser does not surface on the usage), are not mapped. Anonymous constraints have no identity and
are skipped. Mapped constraints no longer count toward the advisory `W543`.
