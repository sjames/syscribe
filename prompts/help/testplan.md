# testplan — list TestPlans, show plan detail, and roll up a verdict

## SYNOPSIS
    syscribe -m <root> testplan [--json]
    syscribe -m <root> testplan TP-X [--json]

## DESCRIPTION
Read-only surface over `TestPlan` (TP-*) elements (GH #38). A TestPlan is a
curated, per-product verification artifact: it binds a set of TestCases (the
effective set = explicit `testCases:` ∪ `selection:` matches) to zero or more
`Configuration`s at one `scope`, and is optionally offered as evidence for the
goals it `demonstrates:`.

With no argument, lists every TestPlan with its id, title, scope, bound
configurations, effective-TestCase count, coverage % and rolled-up verdict.

With a `TP-X` argument, shows the detail for one plan: the resolved member
TestCases (each flagged `escaping` when active in none of the plan's configs),
the in-scope requirements, a per-config coverage grid and the roll-up verdict. An
unknown id, or an id that does not resolve to a TestPlan, is a usage error
(exit 1).

## IN-SCOPE REQUIREMENTS
- With `demonstrates:` → the goal-closure: each demonstrated `Requirement` plus
  the transitive closure of its `derivedChildren`; for a demonstrated
  `SafetyGoal`/`CybersecurityGoal`, the requirements that
  `derivedFromSafetyGoal:`/`derivedFromCybersecurityGoal:` it (and their closure).
- Without `demonstrates:` → the union of the `verifies:` targets of the plan's
  effective TestCase set.

## COVERAGE AND VERDICT
Coverage reuses the `matrix` coverage computation, scoped to the plan's effective
TestCase set and in-scope requirements. The verdict ∈ `pass | fail | incomplete
| empty`: `empty` when the effective set is empty; `fail` when any member's
ingested verdict is Fail; `pass` when every member passes; otherwise
`incomplete` (no/partial results). Verdicts use a loaded results sidecar (see
`ingest-results`).

## OPTIONS
    --json          Emit a single schemaVersion-stamped document.

## EXAMPLES
    syscribe -m model/ testplan
    syscribe -m model/ testplan --json
    syscribe -m model/ testplan TP-TRS-MCP-001
    syscribe -m model/ testplan TP-TRS-MCP-001 --json

## RESULTS PER RIG
A plan with `runsOn:` is judged on each listed TestEnvironment alone, using the results recorded on that rig
(`ingest-results --format records`): every rig must pass, a rig with no passing result leaves the plan
`incomplete`, results from other rigs are ignored, and results from a retired or out-of-calibration rig do
not count. Without `runsOn:` a failure on any rig fails the plan.

## RESULTS PER CONFIGURATION
A plan with `configurations:` is judged on the results of those Configurations only (`ingest-results --config`):
each one on the results recorded without a Configuration overlaid with its own, and the plan takes the worst of
them (`fail`, then `incomplete`, then `pass`). A result recorded for a Configuration the plan does not list never
changes it, and a member is judged only in the Configurations it is active in. When a listed Configuration has
results of its own, gaps read `<test> @ <rig> [axis=point] in <config> (…)`, and with several Configurations the
detail lists each verdict (`verdictByConfiguration` in `--json`). A plan without `configurations:` is judged on
the worst over everything recorded.

## REQUIRED COVERAGE
A plan may declare what it requires (ADR-SYS-COVERAGE-001): `conditions:` (axes of named points), `minUnits:`
(distinct units per point), `mode: each|product` or an explicit `points:` list. `validate` checks the declaration
(`E607` malformed, `E608` a `points:` name `conditions:` does not declare). The plan then passes only when every
cell of the grid — each member, each `runsOn:` rig (any rig when none), each required condition point — has a
pass, on at least `minUnits` distinct named units when that is set. A record counts toward a point when its
`condition` includes it (`ingest-results --condition`, `--unit`); results from a retired or out-of-calibration rig
count for nothing. A failure at any cell fails the plan; a cell with no pass leaves it `incomplete`, and `testplan
TP-X` lists the gaps as `<test> @ <rig> [axis=point] (missing|FAILED|n of m units)`. For an `approved` plan with
results, `W617` lists the same gaps.

## SEE ALSO
    matrix, audit, verification-depth, coverage tree (each accepts the `--plan TP-X` lens),
    ingest-results
