---
id: REQ-TRS-PLANITEM-010
type: Requirement
name: A PlanningItem marked done shall warn when an achieves requirement lacks verification coverage
status: draft
reqDomain: software
verificationMethod: test
---

A `PlanningItem` at `status: done` **shall** raise warning `W310` for each `achieves:` entry
resolving to a native `Requirement` that does not yet meet the verification bar `validate`
already applies to that Requirement directly: at least one **active** `TestCase` when the
Requirement is a leaf (no `derivedChildren`), or at least one **active, integration-level**
(`testLevel: L3`, `L4`, or `L5`) `TestCase` when the Requirement is a parent (mirroring `W002`
and `W305` respectively — a parent's leaf descendants carrying coverage does not change what
`W305` itself requires directly on the parent). The rule **shall not** fire for a `PlanningItem`
at any other status (`todo`/`in_progress`/`blocked`), and **shall not** re-derive its own
dangling/wrong-kind checks — an unresolved (`E714`) or non-Requirement (`E715`) `achieves:`
target is silently skipped here, since those are already reported once by their own checks.
The rule applies at any tree position (leaf or non-leaf `PlanningItem`), unlike the leaf-only
`E719` evidence rule: an `achieves:` claim's verification state does not depend on whether the
claiming item itself has children.

**Motivation:** `W002`/`W305` already exist and would, in principle, catch the underlying gap —
but they are independent, per-Requirement warnings that show up in the general noise of a
`validate` run. Nothing ties "this specific `PlanningItem` you are about to mark done" to "here
specifically are the `achieves:` requirements that are not actually backed by evidence yet" —
the distinction that matters most exactly when it is needed most, right before a `PlanningItem`
is committed as finished.

**Source:** GitHub issue #114, `ADR-SYS-PLANITEM-001`.

**Acceptance criteria:**
- A `done` `PlanningItem` whose `achieves:` Requirement is a leaf with no active `TestCase`
  raises `W310`.
- A `done` `PlanningItem` whose `achieves:` Requirement is a parent with active TestCases but
  none at `testLevel: L3`/`L4`/`L5` raises `W310` (mirrors `W305`'s own bar, not a bare
  "any active TestCase" check).
- A `done` `PlanningItem` whose `achieves:` Requirement already meets the applicable bar raises
  nothing.
- A `todo`/`in_progress`/`blocked` `PlanningItem` raises nothing regardless of its `achieves:`
  requirements' verification state.
- `W310` fires once per (`PlanningItem`, `Requirement`) pair, on the `PlanningItem`'s own file —
  a distinct finding from any `W002`/`W305` already present on the Requirement's own file, not a
  duplicate of it.
