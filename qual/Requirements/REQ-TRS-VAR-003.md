---
id: REQ-TRS-VAR-003
type: Requirement
name: Tool shall accept boolean expressions over FeatureDefs in appliesWhen
status: draft
reqDomain: software
verificationMethod: test
---

`appliesWhen:` **shall** accept a boolean expression over `FeatureDef` qualified names using `and`, `or`, `not` and parentheses, in addition to the existing forms which remain valid and unchanged:

- a bare qualified name (trivial single-term expression);
- a list of qualified names (AND semantics).

Every operand qualified name in an expression **shall** be a resolved cross-reference to a `FeatureDef`; an unresolved operand, or one that resolves to a non-`FeatureDef` element, is `E209`.

Expressions **shall** be evaluated against a `Configuration`'s feature selections, where a feature absent from the selections takes its `FeatureDef` default / group rule. The tool **shall** implement standard truth-table semantics for `and`, `or`, `not`, and parenthesisation.

**Source:** Issue #11

**Acceptance criteria:** A bare-QName `appliesWhen:` behaves exactly as before; `"A and B"`, `"A or B"`, `"not A"`, and parenthesised combinations evaluate correctly against `Configuration` selections across the full truth table; an unresolved `FeatureDef` inside an expression yields `E209`; an expression is inert when the variability dimension is dormant (see [[REQ-TRS-VAR-001]]).
