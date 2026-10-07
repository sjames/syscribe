---
id: REQ-TRS-SYSMLV2-052
type: Requirement
name: export-sysml shall emit constraint and calc parameters and expression bodies
status: verified
reqDomain: software
verificationMethod: test
---

`ConstraintDef`/`Constraint` and `CalculationDef`/`Calculation` elements shall export their
`parameters:` entries (`in`/`out`/`inout x : T;`, `return r : T;` for `direction: return`) and
their `expression:`/`body:` text, one statement per line, inside the body, so a model ingested by
`REQ-TRS-SYSMLV2-033`/`-034` and exported again re-ingests with equal parameters and expression
text. Placeholder text such as `<conditional expression>` is never emitted as source; it becomes a
comment.

**Source:** `REQ-TRS-SYSMLV2-052` (product model), `ADR-SYS-SYSMLV2-002`.
