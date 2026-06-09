---
id: REQ-TRS-PLAN-006
type: Requirement
name: --plan Lens on Analysis Commands
title: Tool shall provide a --plan TP-X lens that restricts analysis commands to a TestPlan's scope and composes with --config
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a `--plan TP-X` lens, **symmetric** to the `--config`
projection lens ([[REQ-TRS-PROJ-001]]), that restricts an analysis command to a single
TestPlan's scope.

### Affected commands

The `--plan TP-X` lens **shall** be accepted on the row-restricting report commands:

- `matrix` ([[REQ-TRS-VAR-004]]),
- `verification-depth` ([[REQ-TRS-OUT-011]]).

`audit` ([[REQ-TRS-OUT-013]]) is **deferred** for v1: its readiness verdict runs the
full cross-element validation rule set, and a TestPlan subset is not
reference-complete, so escaping references would surface as spurious findings (the
projection-aware-validation problem of GH #36, generalised to an arbitrary subset).
`audit --plan` requires projection-aware validation over the plan scope and is tracked
as a follow-up; it **shall not** be offered until that lands.

### Behaviour

- Given `--plan TP-X`, the command **shall**:
  - restrict its **rows** to the plan's **in-scope requirements** (per
    [[REQ-TRS-PLAN-005]]); and
  - restrict the **TestCase universe** to the plan's **member** TestCases (per
    [[REQ-TRS-PLAN-003]]).
- `--plan` **shall compose** with `--config` ([[REQ-TRS-PROJ-001]]): when both are given,
  the command operates over the plan's in-scope requirements and members **and** the
  active subset of the selected configuration.
- An **unknown / unresolvable** `TP-id` **shall** be a usage error (non-zero exit,
  exit `1`).
- `--plan` **shall** be **dormant-safe**, exactly like `--config`: on a model with no
  feature model the lens still scopes by plan membership without error.

**Source:** GH #38; symmetric to ADR-PROJ-001's `--config` lens, generalised to plan
scope.

**Acceptance criteria:** `matrix --plan TP-X` and `verification-depth --plan TP-X`
each restrict rows to the plan's in-scope requirements and the TestCase universe to the
plan's members; `matrix --plan TP-X --config CONF-A` composes both lenses (plan scope
intersected with the active subset of CONF-A); an unknown `TP-id` exits `1`; on a model
with no feature model `--plan` still scopes by membership without error.
