---
id: REQ-TRS-SAFE-009
type: Requirement
name: "W039 shall apply to silLevel 3 and 4 in addition to asilLevel D"
status: draft
reqDomain: software
verificationMethod: test
---

**W039** (high-integrity item lacking required independent assessment) **shall** fire for
`silLevel: 3` and `silLevel: 4` items as well as `asilLevel: D`.

IEC 61508-1 §8 mandates independent assessment at SIL 3 and SIL 4, equivalent to the
ISO 26262 requirement for ASIL D. The existing W039 check incorrectly treated `silLevel`
as out-of-scope, making the dormant-trigger fire (a `ConfirmationMeasure` exists) but
producing no W039 for SIL 3/4 goals.

The W039 message for SIL 3/4 items **shall** reference IEC 61508-1 §8 alongside ISO
26262-2 §6 to distinguish the triggering standard.

**Acceptance criteria:**

- A `SafetyGoal` with `silLevel: 4` in a model that contains at least one
  `ConfirmationMeasure` but no `functional_safety_assessment` at `I3` confirming it
  yields **W039** naming the goal.
- A `SafetyGoal` with `silLevel: 3` yields **W039** under the same conditions.
- A `SafetyGoal` with `silLevel: 2` does **not** yield W039.
- A `SafetyGoal` with `asilLevel: D` still yields W039 (regression check).
- A model with no `ConfirmationMeasure` emits no W039 regardless of `silLevel` (opt-in
  invariant preserved).
