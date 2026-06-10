---
id: REQ-TRS-PLAN-001
type: Requirement
name: TestPlan Native Element and Schema
title: Tool shall recognise a native TestPlan element with a stable opaque id and validated frontmatter schema
status: draft
reqDomain: software
verificationMethod: test
---

A **TestPlan** is a first-class, named aggregation of TestCases offered as a unit of
verification evidence (e.g. a smoke plan, an integration plan, a certification plan).
The tool **shall** recognise a native element declared with `type: TestPlan` and
carrying a **stable opaque identifier**, mirroring the existing native `TestCase`
(`TC-*`), `Requirement` (`REQ-*`) and `ADR` (`ADR-*`) elements.

### Identifier

- The `id` **shall** match the pattern `^TP(-[A-Z0-9]{2,12})+-[0-9]{3,8}$` (prefix `TP`,
  one or more uppercase-alphanumeric segments of 2–12 chars, three-digit suffix), e.g.
  `TP-SMOKE-001`, `TP-CERT-BRAKE-001`. The id is stable and never changes.
- The id field is required; a missing or malformed `TP-id` **shall** raise `E600`.
- A **duplicate** TestPlan id **shall** raise the existing generic duplicate-id error
  `E101` (the same code used for any duplicated stable id); this requirement does not
  introduce a new duplicate code.

### Frontmatter schema

- **Required:** `id`, `title`, `status`. A missing `title` or `status` **shall** raise
  `E600` (the same code as a missing/malformed id).
- `status` **shall** be one of `draft | review | approved | active | retired`; any other
  value **shall** raise `E604`.
- `scope` is a single free-form string with a **recommended vocabulary** of
  `unit | smoke | integration | hil | certification | security | regression`. A
  `scope` value outside this vocabulary **shall** be accepted (free-form) but **shall**
  raise the advisory warning `W610`.
- **Optional:** `configurations`, `demonstrates`, `testCases`, `selection`, `tags`
  (their semantics are defined in [[REQ-TRS-PLAN-002]], [[REQ-TRS-PLAN-003]] and
  [[REQ-TRS-PLAN-004]]).

### Placement and dormancy

- TestPlans live **by convention** in a `TestPlans/` area of the model, but this is
  **not enforced** — there is no validation code for placement.
- A TestPlan is a valid element **regardless** of whether a feature model is present;
  the element and its schema checks are not gated on variability being active.

**Source:** GH #38 (native TestPlan element).

**Acceptance criteria:** an element with `type: TestPlan` and a well-formed `TP-id`,
`title` and valid `status` parses cleanly; a missing/malformed `TP-id`, missing `title`
or missing `status` raises `E600`; a `status` outside the enum raises `E604`; a `scope`
outside the recommended vocabulary still parses but raises `W610`; two TestPlans sharing
one id raise the existing `E101`; a TestPlan in a model with no `FeatureDef` is still a
valid, schema-checked element. `syscribe template TestPlan` emits a `type: TestPlan`
skeleton (TP-* id, `title`, `status`, `scope`, a `testCases` member), and `TestPlan`
appears among the `template` command's known native types.
