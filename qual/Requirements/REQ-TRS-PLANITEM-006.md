---
id: REQ-TRS-PLANITEM-006
type: Requirement
name: A leaf PlanningItem marked done shall have at least one non-waived, resolving evidence entry
status: draft
reqDomain: software
verificationMethod: test
---

A **leaf** `PlanningItem` (empty computed `children`) at `status: done` **shall** have at least
one `evidence:` entry that resolves successfully (a `ref:` that resolves, or a `path:` that exists
locally or is an accepted remote URI). An entry excused by its own `rationale:` **shall not** count
toward satisfying this rule. A leaf `PlanningItem` in any other status (`todo`/`in_progress`/
`blocked`) **shall** raise nothing regardless of its `evidence:` content. A **non-leaf**
`PlanningItem` (has `children`) **shall not** be constrained by this rule at all, regardless of its
own `status`/`evidence:`.

**Source:** `REQ-TRS-PLANITEM-006` (product model), `ADR-SYS-PLANITEM-001`.

**Acceptance criteria:** a leaf, `status: done` item with resolving evidence validates cleanly; a
leaf, `status: done` item with no evidence, or with evidence entries that are all
`rationale:`-waived, is rejected as an error; a leaf in `todo`/`in_progress`/`blocked` with no
evidence raises nothing; a non-leaf `status: done` item with no evidence of its own raises nothing.
