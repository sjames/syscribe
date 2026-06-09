---
id: REQ-TRS-PLAN-004
type: Requirement
name: TestPlan Demonstrated Goals
title: Tool shall let a TestPlan declare the safety-case goals it demonstrates and flag goals with no verifying member
status: draft
reqDomain: software
verificationMethod: test
---

A TestPlan may be offered as **evidence** for one or more safety-case goals. The
optional `demonstrates:` list names the artifacts the plan stands as evidence for.

### `demonstrates:` field

- `demonstrates:` is an optional list naming `Requirement`, `SafetyGoal`,
  `CybersecurityGoal` or `Argument` elements.
- `demonstrates:` is **not mandatory** — a plan with no declared goal is a valid plan.
- Each `demonstrates:` target **shall** resolve to an element of one of those types; a
  target that does not resolve **shall** raise `E603`.

### Evidence-gap warning

- A plan whose `status` is `approved` or `active`, whose `demonstrates:` names a
  `Requirement`, but which has **no member TestCase** (per [[REQ-TRS-PLAN-003]]) that
  `verifies:` that requirement **or any requirement in its goal-closure** (a
  requirement that transitively `derivedFrom:` it), **shall** raise `W614` — the plan
  claims to demonstrate a goal it carries no test for. Demonstrating a high-level /
  parent goal whose **leaves** are tested is the normal safety-case pattern and
  **shall not** raise `W614` (a parent is demonstrated through its leaves, consistent
  with the parent suppression of W002/W300/W306 and [[REQ-TRS-OUT-013]] / GH #37).

### Integrity-level scope (v1)

- A TestPlan is **out of ASIL/SIL integrity-level propagation** for v1. It is an
  **aggregation** of existing TestCases, not a derivation, and **shall not** receive,
  alter, or propagate any integrity level onto or from its members or the goals it
  demonstrates.

**Source:** GH #38 (safety-case evidence aggregation).

**Acceptance criteria:** a plan with no `demonstrates:` is valid; a `demonstrates:`
target that does not resolve raises `E603`; an `approved`/`active` plan that
`demonstrates:` a `Requirement` for which no member TestCase has a matching `verifies:`
raises `W614`, while the same plan with a covering member does not; declaring or omitting
`demonstrates:` never changes any element's `silLevel`/`asilLevel`.
