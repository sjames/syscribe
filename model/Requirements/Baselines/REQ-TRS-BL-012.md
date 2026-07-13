---
type: Requirement
id: REQ-TRS-BL-012
name: "Trace-closure baseline scope"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BL-000]
breakdownAdr: Decisions::BaselineADR
tags:
  - baseline
  - scope
  - traceability
---

A baseline shall be able to freeze the **trace closure** of one or more seed elements — a
goal or requirement and everything transitively connected to it by trace links — so the
*safety case for a single goal* can be sealed and assessed (REQ-TRS-BL-003 reserved this).

- `frozenScope.closureFrom` shall be a list of seed references (ids or qualified names),
  typically a top-level `SafetyGoal`/`CybersecurityGoal`/requirement.
- When `closureFrom` is set, scope resolution shall restrict the base element set to the
  **transitive trace closure** of the seeds: the seeds themselves plus every element reachable
  by following trace links (`verifies`, `derivedFrom`, `satisfies`, `satisfiedBy`, `refines`,
  `implementedBy`, `supports`, `evidence`, `breakdownAdr`, the structural and domain links —
  the full link set of REQ-TRS-SUS-LINKS-003) in **either direction**. This yields the
  connected trace component of the goal — its decomposed requirements, their verifying tests
  and satisfying architecture, and the supporting arguments/evidence/rationale.
- The remaining `frozenScope` filters (`package`/`types`/`status`/`tags`) and the
  `Baseline`-exclusion shall apply **over** the closure, and `closureFrom` shall compose with
  `config` (the closure is computed within the projected variant, REQ-TRS-BL-011).
- Drift detection (REQ-TRS-BL-005) and `verify` (REQ-TRS-BL-008) shall recompute the closure,
  so that adding, removing, or editing any element in the goal's trace component is detected as
  drift. A seed that does not resolve contributes nothing; if the closure is empty, `create`
  refuses as for any empty scope.

Seeds are intended to be top-level goals/requirements, for which the bidirectional closure is
the goal's subtree plus its referenced rationale; seeding a mid-level element will also pull in
its ancestors and their other descendants.
