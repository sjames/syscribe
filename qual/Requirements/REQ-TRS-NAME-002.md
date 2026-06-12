---
id: REQ-TRS-NAME-002
type: Requirement
name: Tool shall use name as the single human-readable label on every element
status: draft
reqDomain: software
verificationMethod: test
---

Every element has **exactly one** human-readable label field, and it is **`name`** on
**every** element — regardless of identity class. This unifies the model: the label of a
`Requirement`, a `TestCase`, a `PartDef`, a `Package` and a `FeatureDef` is always
spelled `name`. The previously type-dependent split (`title` on id-identified types,
`name` on name-identified types) is **removed** (see [[REQ-TRS-ID-006]]; SysMLv2:
`name` ≈ `declaredName`, `id` ≈ `declaredShortName`).

### The unified rule

- **`name`** is the human-readable label (`declaredName`) on **all** element types. It is
  **optional in general** but **required** on every type that previously required a
  `title` — the native and safety/security id-identified types (`Requirement`,
  `TestCase`, `TestPlan`, `Configuration`, `ADR`, `ConfirmationMeasure`,
  `HazardousEvent`, `SafetyGoal`, `DamageScenario`, `ThreatScenario`,
  `CybersecurityGoal`, `SecurityControl`, `VulnerabilityReport`, `TARASheet`,
  `FaultTree`, `FaultTreeGate`, `FaultTreeEvent`, `FMEASheet`, `FMEAEntry`, `AttackTree`,
  `AttackTreeGate`, `AttackStep`, `Argument`, `AssumptionOfUse`). A missing required
  `name` keeps that type's existing required-label error code, with the message now
  naming `name`.
- **`id`** (the stable shortName) remains the **identity** of the id-identified types —
  files stay `<id>.md` and cross-references (`verifies:`, `derivedFrom:`, …) still
  resolve by id. For those types `name` is a **free label** (the basic-name `W042`
  convention governs the *identity* segment — the `id` — not the free-text `name`, so a
  requirement's `name` may contain spaces and punctuation). For name-identified types
  `name` is **both** the label **and** the identity segment, so the basic-name convention
  (`W042`, [[REQ-TRS-NAME-001]]) applies to it.

### `title` is removed

- The `title` field is **no longer a recognized label field** on any element. Declaring
  `title:` on **any** element **shall** raise error **`E025`** — "the `title` field is
  removed; rename it to `name`". `E025` is type-independent: it fires for an id-identified
  element and a name-identified element alike.
- Error **`E024`** (formerly: a `name` field on an id-identified type) is **retired** —
  `name` is now the correct label on those types, so a `Requirement` carrying `id` +
  `name` validates clean.

### FeatureDef

A `FeatureDef` is name-identified (its label and qualified-name segment are its `name`)
and **also** carries a mandatory stable `FEAT-*` `id` ([[REQ-TRS-ID-006]]). Under the
unified rule it carries `name` (label) + `id` (shortName) and **no** `title` — clean of
both `E024` (retired) and `E025`.

**Source:** identity-model unification (Stage 2) — make `name` the universal label and
remove `title`. Supersedes the earlier fixed-by-identity-class rule.

**Acceptance criteria:**

- A `Requirement` (or any id-identified type) that declares `id` + `name` (no `title`)
  validates clean; **no** `E024` is ever emitted (the code is retired).
- A requirement's `name` may contain spaces/punctuation (free prose) without raising
  `W042` — only name-identified types' `name` is held to the basic-name grammar.
- Any element — id-identified or name-identified — that declares `title:` raises `E025`;
  removing `title:` clears it.
- A `FeatureDef` carrying a `FEAT-*` `id` **and** a `name` (no `title`) validates clean of
  both `E024` and `E025`.
- Every type that previously required `title` now requires `name` (same error code).
- The bundled `model`, `model_sil`, `model_auto` and the `qual` model validate with no
  `E024`/`E025` findings.
