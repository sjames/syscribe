---
type: PartDef
name: BrakeController
domain: software
satisfies: [REQ-BRK-002]
links:
  partiallySatisfies: REQ-BRK-003
tags:
  - brakes
---

Software control loop that estimates wheel slip, detects lock onset and
commands pressure release.

`satisfies: [REQ-BRK-002]` is a same-domain (software to software) assignment.
`partiallySatisfies: REQ-BRK-003` targets a **hardware** requirement: a plain
`satisfies:` there would raise `E313`, but `partiallySatisfies` is declared
`extends = "satisfies"` with `relax = ["E313"]`, so this instance is exempt. It
is also `coverage = false`, so `REQ-BRK-003`'s `satisfiedBy` stays exactly
`HydraulicModulator` — the controller's contribution is recorded and
traversable without counting as a second satisfier.
