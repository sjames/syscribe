---
id: REQ-TRS-SEC-003
type: Requirement
title: Tool shall model ISO/SAE 21434 attack paths as attack trees and roll up attack feasibility (min/max), reconciling it with the ThreatScenario
status: draft
reqDomain: software
verificationMethod: test
---

ISO/SAE 21434 §15.7 requires an **attack path analysis** work product: each
`ThreatScenario` should be substantiated by one or more attack paths whose
**attack feasibility** is determined from the per-step feasibility, not merely
asserted as a flat rating. Historically Syscribe carried only a single
`attackFeasibility` on a `ThreatScenario` and had no way to model the attack
paths themselves. This requirement adds three element types — mirroring the
`FaultTree`/`FaultTreeGate`/`FaultTreeEvent` family — plus a weakest-link
feasibility roll-up and a reconciliation warning. It is the security counterpart
of Fault Tree Analysis (#20, the "add the missing types" move for safety).

## New element types

The tool **shall** recognise three new element types:

- **`AttackTree`** (stable id pattern `AT-*`, regex `^AT(-[A-Z0-9]{2,12})+-[0-9]{3,8}$`)
  — the root of an attack tree. It carries a new field **`threatRef`** that
  **shall** resolve, via the standard cross-reference resolver, to a
  `ThreatScenario` (the threat the tree substantiates — the security analog of
  `FaultTree.topEvent`→`SafetyGoal`). `threatRef` is **required**.
- **`AttackTreeGate`** (stable id pattern `ATG-*`, regex
  `^ATG(-[A-Z0-9]{2,12})+-[0-9]{3,8}$`) — a combinator. It reuses the existing
  `gateType` and `inputs` fields. Here `gateType` **shall** be one of `AND`
  (a sequential path — all sub-steps required) or `OR` (alternative paths).
  `inputs` lists the ids of child gates/steps.
- **`AttackStep`** (stable id pattern `ATS-*`, regex
  `^ATS(-[A-Z0-9]{2,12})+-[0-9]{3,8}$`) — a leaf step. It reuses the existing
  `attackFeasibility` field (`high` | `medium` | `low` | `very_low`) for the
  per-step feasibility.

All three id patterns **shall** be recognised as stable ids by the resolver.

## Directory / nesting rule

Mirroring the FTA family, an `AttackTree`'s gates and steps **shall** live in a
subdirectory named after the tree file (so their qualified names are prefixed by
the tree's qualified name). An `AttackTree` with no `AttackTreeGate` or
`AttackStep` descendant under its qualified-name prefix is empty and **shall**
produce warning **`W036`** (the analog of FTA's `W900`).

## Feasibility roll-up (normative — weakest link)

Feasibility rank is shared with the existing risk model: `very_low`=0, `low`=1,
`medium`=2, `high`=3. The tool **shall** compute a tree's feasibility exactly as:

- An `AttackStep`'s value is the rank of its `attackFeasibility`.
- An `AttackTreeGate` with `gateType: AND` (a path — all steps needed) is the
  **MIN** of its children's values (a chain is only as feasible as its hardest
  step).
- An `AttackTreeGate` with `gateType: OR` (alternatives) is the **MAX** of its
  children's values (the attacker takes the easiest path).
- The `AttackTree`'s computed feasibility is the value of its single root child
  (the gate or step it contains). The resulting rank maps back to a label
  (0→`very_low`, 1→`low`, 2→`medium`, 3→`high`).

There **shall** be exactly one roll-up definition, exposed as a `pub` function in
`syscribe-model`, shared by any caller.

## Structural validation codes (E915–E921)

- **`E915`** — `id`, `title`, `status`, or `threatRef` is absent on an
  `AttackTree`.
- **`E916`** — an `AttackTree.id` does not match the `AT-*` pattern.
- **`E917`** — `AttackTree.threatRef` does not resolve, or resolves to an element
  that is not a `ThreatScenario`.
- **`E918`** — `id`, `title`, or `gateType` is absent on an `AttackTreeGate`; or
  the `id` does not match the `ATG-*` pattern.
- **`E919`** — `AttackTreeGate.gateType` is not `AND` or `OR`.
- **`E920`** — an entry in an `AttackTreeGate.inputs` does not resolve, or
  resolves to an element that is not an `AttackTreeGate` or `AttackStep`.
- **`E921`** — `id` or `title` is absent on an `AttackStep`; the `id` does not
  match the `ATS-*` pattern; or the `attackFeasibility` is not one of `high`,
  `medium`, `low`, `very_low`.

The error shapes **shall** mirror the FTA family (E900–E910).

## Reconciliation warning (W035)

The tool **shall** define warning code **`W035`**, emitted by `validate` for an
`AttackTree` whose computed feasibility (per the roll-up) does **not** match the
`attackFeasibility` declared on the `ThreatScenario` named by its `threatRef`.
The message **shall** name the computed level and the declared level. `W035`
fires only when the threat resolves and both feasibilities are computable. It is
a **warning** (exit code unchanged), gateable via `--deny W035`, and promotable
to error via a `[profiles]` policy (#18) — matching the convention that
completeness/consistency gaps are warnings.

**Source:** ISO/SAE 21434 §15.7 attack path analysis; GH issue #32. Companion to
the Fault Tree Analysis types (#20) which it mirrors.

**Acceptance criteria:** an `AttackTree` with `threatRef` to a `ThreatScenario`,
a root `OR` gate over an `AND` gate and a step, validates with **no errors**; its
computed feasibility follows the weakest-link rule (AND=min, OR=max) and maps
back to a label; when the computed feasibility differs from the linked
`ThreatScenario.attackFeasibility`, exactly one `W035` naming computed vs
declared is emitted; aligning the declared feasibility clears `W035`; a
`threatRef` to a non-`ThreatScenario` or an unresolved ref produces `E917`; the
new types resolve with no orphan/dangling-ref errors; `validate --deny W035`
exits non-zero in the presence of a `W035`; bundled models with no attack trees
emit zero new findings.
