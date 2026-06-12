---
id: REQ-TRS-PLAN-005
type: Requirement
name: Tool shall provide a testplan command that lists plans, shows plan detail, and rolls up a verification verdict
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only `testplan` command that surfaces TestPlans, their
resolved membership, their coverage, and a rolled-up verdict.

### `testplan` (list)

`syscribe -m <root> testplan` **shall** list every TestPlan with: `id`, `title`,
`scope`, bound configurations, **effective-TestCase count** (per [[REQ-TRS-PLAN-003]]),
**coverage %**, and **verdict**.

### `testplan TP-X` (detail)

`syscribe -m <root> testplan TP-X` **shall** show, for that plan:

- the resolved member TestCases, each marked with an **escaping** flag where applicable
  (escaping membership is defined in [[REQ-TRS-PLAN-002]]);
- the **in-scope requirements** (see below);
- a **per-config coverage grid**;
- the rolled-up verdict.

### Coverage and scope

- **Coverage %** **shall reuse** the `matrix` coverage computation
  ([[REQ-TRS-VAR-004]]), scoped to the plan's **effective TestCase set** and its
  **in-scope requirements** — no duplicated coverage logic.
- **In-scope requirements** are:
  - with `demonstrates:` ([[REQ-TRS-PLAN-004]]) → the **goal-closure** requirements of
    the demonstrated goals;
  - without `demonstrates:` → the `verifies:` targets of the effective TestCase set.

### Verdict roll-up

- The plan **verdict** ∈ `pass | fail | incomplete | empty`, computed via the **existing**
  results / `tc_verdict` fold over the member TestCases — not a new verdict engine.
- **Results-gated** `W615`: an `approved` plan that has a member whose **ingested
  verdict** is `Fail` or `Missing` **shall** raise `W615`, **only when a results
  sidecar is loaded** (when no results are loaded, `W615` is not emitted).

### Output

- `--json` **shall** emit a single `schemaVersion`-stamped document carrying the listed
  fields, the per-config coverage grid, member/escaping data and the verdict.

**Source:** GH #38; reuses the `matrix` coverage computation ([[REQ-TRS-VAR-004]]) and
the results/`tc_verdict` fold.

**Acceptance criteria:** `testplan` lists each plan with id, title, scope, configs,
effective-TC count, coverage % and verdict; `testplan TP-X` shows resolved members with
escaping flags, the in-scope requirements, a per-config coverage grid and the roll-up
verdict; coverage % equals the `matrix` value for the plan's effective TC set and
in-scope requirements; the verdict is `empty` for an empty plan, `incomplete`/`pass`/`fail`
per the existing fold; `--json` emits one `schemaVersion`-stamped document; with a results
sidecar loaded, an `approved` plan with a Fail/Missing member raises `W615`, and with no
sidecar loaded it does not.
