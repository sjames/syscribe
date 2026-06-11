---
id: REQ-TRS-NAME-002
type: Requirement
title: Tool shall enforce a single label field per element, fixed by identity class
status: draft
reqDomain: software
verificationMethod: test
---

Every element has **exactly one** human-readable label field. Which field carries the
label is **fixed by the element's identity class** — never both, and the wrong field is
an error. This removes the ambiguity that let elements drift into carrying both a `name`
and a `title`.

### Identity classes

An element is **id-identified** or **name-identified**:

- **id-identified** — the element's identity is a stable `id` (a shortName such as
  `REQ-*`, `TC-*`, `HE-*`). Its label field **shall** be **`title`**. The 24
  id-identified types are: `Requirement`, `TestCase`, `TestPlan`, `Configuration`,
  `ADR`, `ConfirmationMeasure`, `HazardousEvent`, `SafetyGoal`, `DamageScenario`,
  `ThreatScenario`, `CybersecurityGoal`, `SecurityControl`, `VulnerabilityReport`,
  `TARASheet`, `FaultTree`, `FaultTreeGate`, `FaultTreeEvent`, `FMEASheet`,
  `FMEAEntry`, `AttackTree`, `AttackTreeGate`, `AttackStep`, `Argument`,
  `AssumptionOfUse`.
- **name-identified** — the element's identity is its `name` (and the path-derived
  qualified-name segment). Its label field **shall** be **`name`**. This is **every
  other type**: all SysML structural types (`PartDef`, `Part`, `Port`, `ActionDef`, …),
  `Package`, `Diagram`, and **`FeatureDef`**.

`FeatureDef` is the single type that is name-identified yet **may** also carry an
optional stable `id` (the `FEAT-*` shortName, [[REQ-TRS-ID-006]]). Its label remains
`name`; the `id` axis (shortName) and the label axis (`name` vs `title`) are
independent. No element ever carries both `name` and `title`.

### Validation

- An **id-identified** element that declares a **`name:`** field **shall** raise error
  **`E024`**, naming the field and the element's type and stating that the label belongs
  in `title:`.
- A **name-identified** element that declares a **`title:`** field **shall** raise error
  **`E025`**, naming the field and the element's type and stating that the label belongs
  in `name:`.
- The existing required-label checks are unchanged: an id-identified element still
  **shall** carry a `title` (e.g. `E004` on a native `Requirement`); a name-identified
  element's `name` defaults to its filename stem when omitted.
- `E024`/`E025` are **errors** (not warnings): the rule is strict and there is no
  backward-compatibility allowance for the dual-field form.

**Source:** GH discussion (identity-model unification — "keep id primary", option (a):
id-identified types keep `title`). Closes the gap left open by [[REQ-TRS-NAME-001]]
(which constrained only the *characters* of a name, not *which* label field applies).

**Acceptance criteria:**

- A `Requirement` (or any id-identified type) that declares `name:` raises `E024`;
  removing `name:` clears it. A `Requirement` with only `id` + `title` validates clean.
- A `PartDef`/`Package`/`FeatureDef` (or any name-identified type) that declares
  `title:` raises `E025`; removing `title:` clears it.
- A `FeatureDef` carrying both an optional `FEAT-*` `id` **and** a `name` (no `title`)
  validates clean — the `id` and `name` axes are independent ([[REQ-TRS-ID-006]]).
- The bundled `model`, `model_sil`, `model_auto` and the `qual` model validate with no
  `E024`/`E025` findings.
