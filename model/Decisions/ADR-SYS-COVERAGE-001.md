---
type: ADR
id: ADR-SYS-COVERAGE-001
name: "Required coverage (conditions, units, rigs) is declared on the TestPlan"
status: accepted
---

## Context

GH #500/#501: a TestCase can pass while a condition (e.g. hot ambient) or a
unit-under-test count was never exercised. Outline rows (`Examples:`) only say
what that one test happens to run, so a short table still looks complete. The
`TestPlan` already owns the rig axis (`runsOn:`, `plan_verdict`, W615).

## Decision

1. **The TestPlan declares required coverage; the TestCase stays the procedure.**
   A plan gains `conditions:` (named axes of named points with values),
   `minUnits:` (distinct units that must pass per point) and `mode:`.
2. **Conditions are inline on the plan.** No new element type now; promote to an
   element only if one envelope is reused across many plans or requirements.
3. **Combinations are explicit.** `mode: each` (default) requires every point of
   every axis on its own; `mode: product` requires every combination;
   `points:` lists an irregular set. Rigs (`runsOn:`) are one more axis.
4. **Records carry coordinates.** A result record names its condition point(s)
   and unit; the sidecar key becomes (test, scenario, environment, condition,
   unit). Existing records read as unconditioned and stay valid.
5. **Plan verdict.** A plan passes only when every required coordinate has a
   pass. A new verdict `incomplete` means: nothing failed, but a required point
   has no pass. Order: fail > incomplete > unknown > pass. `trace`, `matrix`,
   `testplan` and `audit` report it with the missing coordinates.
6. **Outlines stay the self-contained route** for tests with no plan. An outline
   with any unrun row is `unknown` (GH #500 a, PR #504), never `pass`.

## Consequences

- Adding a condition changes the plan, not the TestCase, so existing results
  are not made stale.
- One coverage concept (rigs, conditions, units as axes) instead of three.
- Sidecar key change needs a migration note; old sidecars load unchanged.
- `trace <REQ>` for a requirement verified by tests in several plans must
  define which plan decides; to be specified with the requirement text.
- Follow-ups: REQ/TC for plan fields, validation codes, ingest coordinates,
  `incomplete` in the reporting commands (#500 b–e, #501).
