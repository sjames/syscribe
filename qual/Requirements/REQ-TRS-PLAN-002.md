---
id: REQ-TRS-PLAN-002
type: Requirement
name: Tool shall bind a TestPlan to zero or more Configurations and evaluate it per-config via the projection engine
status: draft
reqDomain: software
verificationMethod: test
---

A TestPlan declares **which products it is a plan for** via a single `configurations:`
field. This mirrors how a `TestCase` with no `appliesWhen:` is config-agnostic while one
with `appliesWhen:` is variant-specific.

### `configurations:` field

- `configurations:` is a scalar or a list of `Configuration` references.
  - **Present** → the plan is a plan for **exactly** those products (configurations).
  - **Absent** → the plan is **config-agnostic**: it applies to every `Configuration`
    (or, in a flat model with no feature model, to the whole model).
- Each listed entry **shall** resolve to a `Configuration` element; an entry that does
  not **shall** raise `E606`.

### Evaluation

- Evaluation **shall reuse** the existing projection engine ([[REQ-TRS-PROJ-001]]),
  running `project(model, selection)` **once per listed configuration** (or once per
  stored `Configuration` when `configurations:` is absent), and **shall** report both
  **per-config** and **aggregate** results.
- Coverage **shall reuse** the existing matrix coverage computation
  ([[REQ-TRS-VAR-004]]) — one shared coverage definition, never a reimplementation.

### Membership-is-computed law

- The "membership is computed, not stored" law is preserved at the **TestCase** level:
  a TestCase's activeness in a config is still decided by its own `appliesWhen:`. The
  plan only declares **which products it is a plan for**; it does not stamp config
  membership onto its TestCases.
- An **escaping member** — a member `TestCase` (per [[REQ-TRS-PLAN-003]]) that is active
  in **none** of the plan's bound configurations — **shall** raise `W611` (the plan
  carries a test that can never run in any product the plan targets).
- Two TestPlans with an **identical** `(configurations, scope)` pair **shall** raise the
  advisory warning `W616` (likely redundant or accidentally duplicated plans).

**Source:** GH #38; reuses ADR-PROJ-001 projection and the `matrix` coverage
computation.

**Acceptance criteria:** a plan with `configurations: [CONF-A, CONF-B]` evaluates and
reports per-config and aggregate results for exactly A and B; a plan with no
`configurations:` evaluates over every stored `Configuration` (or the whole flat model);
an entry that does not resolve to a `Configuration` raises `E606`; a member TestCase
active in none of the plan's configs raises `W611`; two plans with the same
`(configurations, scope)` raise `W616`; coverage numbers match those produced by the
`matrix` command for the same scoped sets.
