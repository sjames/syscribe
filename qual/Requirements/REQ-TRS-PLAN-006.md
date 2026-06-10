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

The `--plan TP-X` lens **shall** be accepted on `matrix` ([[REQ-TRS-VAR-004]]),
`verification-depth` ([[REQ-TRS-OUT-011]]), and `audit` ([[REQ-TRS-OUT-013]]).

### `audit --plan` scoped verdict (GH #40)

Because a TestPlan subset is **not reference-complete** (it omits referents such as a
requirement's `breakdownAdr` ADR or `derivedFrom` parents), validating the *slice*
directly would raise spurious escaping-reference findings (e.g. `E311`, `E102`). To
avoid this, `audit --plan` **shall**:

- compute the readiness **verdict** by running the **full** validation rule set over the
  **whole** model (so every reference resolves — no lens artifacts), then **counting only
  the findings whose element lies in the plan's scope** (the plan's in-scope requirements
  ∪ member TestCases ∪ their satisfying architecture elements). Errors, `W306` and
  profile-promoted findings **outside** the plan scope **shall not** affect the verdict;
- compute the dashboard **sections** (status split, integrity distribution, coverage,
  orphans) over the **plan scope**, **resolving all references against the full model**
  (so an in-scope TestCase that verifies an out-of-scope requirement is not mis-counted
  as dangling, and an in-scope requirement satisfied by an out-of-scope element still
  reads as satisfied).

`audit --plan` **shall compose** with `--config` and **shall** exit `1` on an unknown
plan id.

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
with no feature model `--plan` still scopes by membership without error. `audit --plan
TP-X` on a model whose plan scope is internally clean audits **clean** even when an
in-scope element references an out-of-scope element (no escaping-reference artifact in
the verdict), and FAILs when an **in-scope** element has a real error or `W306`.
