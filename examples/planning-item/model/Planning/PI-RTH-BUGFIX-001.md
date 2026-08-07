---
type: PlanningItem
id: PI-RTH-BUGFIX-001
name: "Fix false RTH trigger on transient battery-sensor noise"
status: done
itemType: bug
parent: PI-RTH-001
achieves: Requirements::REQ-RTH-003
evidence:
  - ref: TC-RTH-NOISE-001
  - path: "https://github.com/example-org/uav-firmware/commit/9f2a3b1"
tags:
  - rth
  - robustness
---

A child of `Planning::PI-RTH-001` with `itemType: bug` — sitting alongside
`itemType: task` siblings (`Planning::PI-RTH-DESIGN-001`,
`Planning::PI-RTH-IMPL-001`, …) at the same breakdown level, demonstrating
`itemType` is independent per node and never inherited or constrained by a
parent's `itemType` (`REQ-TRS-PLANITEM-001`).

Field testing surfaced a single noisy battery-sensor sample spuriously
triggering RTH mid-flight. Although non-top-level, this item **also** sets
its own `achieves:` (permitted, not required, per `REQ-TRS-PLANITEM-003`) —
`Requirements::REQ-RTH-003`, the **qualified-name form** of the target
(`Planning::PI-RTH-001`'s own `achieves:` uses the id form, so this example
exercises both accepted styles).

A leaf at `status: done` with two `evidence:` entries, both resolving with no
waiver needed:

- `ref: TC-RTH-NOISE-001` — a real, hand-authored `TestCase` confirming the
  fix (regression test).
- `path: https://github.com/example-org/uav-firmware/commit/9f2a3b1` — a
  **remote URI**, accepted as external evidence with no local existence
  check (resolved exactly like `implementedBy:`, `REQ-TRS-PLANITEM-005`).
