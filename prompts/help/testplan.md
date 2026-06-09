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
  `derivedFromSafetyGoal:`/`derivedFromSecurityGoal:` it (and their closure).
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
    syscribe -m model/ testplan TP-DELIVERY-INTEGRATION-001
    syscribe -m model/ testplan TP-DELIVERY-INTEGRATION-001 --json

## SEE ALSO
    matrix, audit, verification-depth (each accepts the `--plan TP-X` lens),
    ingest-results
