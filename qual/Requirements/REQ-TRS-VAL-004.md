---
id: REQ-TRS-VAL-004
type: Requirement
title: Tool shall enforce integrity-level propagation errors E841–E843
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce integrity-level propagation rules E841–E843 and W808 as defined in §11.12. When any element in a traceability chain carries `asilLevel:`, `silLevel:`, or `plLevel:`, all downstream elements reachable via `derivedFromSafetyGoal:`, `derivedFrom:`, or `satisfies:` must also carry that field.

| Code | Condition |
|---|---|
| `E841` | Element with `derivedFromSafetyGoal:` missing integrity level when source carries one |
| `E842` | Element with `derivedFrom:` missing integrity level when parent carries one |
| `E843` | Element with `satisfies:` missing integrity level when satisfied requirement carries one |
| `W808` | Element integrity level lower than source without `breakdownAdr:` documenting decomposition |

**Source:** §11.12 (E841–E843, W808), §12.7

**Acceptance criteria:** A three-element chain with `asilLevel: D` at the root and no level on a derived element produces `E842`.
