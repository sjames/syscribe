---
type: PlanningItem
id: PI-RTH-IMPL-001
name: "Implement RTH controller logic"
status: in_progress
itemType: task
parent: PI-RTH-001
assignedTo: alice
tags:
  - rth
---

Implement the battery-threshold monitor and flight-path replanner in the
flight controller firmware. A **non-leaf** — `Planning::PI-RTH-IMPL-SW-001`
and `Planning::PI-RTH-IMPL-SW-002` both set `parent: PI-RTH-IMPL-001`, making
this a grandchild-bearing intermediate node under `Planning::PI-RTH-001`
(the multi-level breakdown `REQ-TRS-PLANITEM-002` describes). `status:
in_progress` and no `evidence:` of its own are both fine regardless — a
non-leaf is never constrained by the leaf-evidence rule.

`assignedTo: alice` (`REQ-TRS-PLANITEM-008`) — `alice` is declared in this model's own
`.syscribe.toml` `[users]` table, mapped to the display name `Alice Nakamura`; `syscribe show
PI-RTH-IMPL-001` resolves and prints it alongside the raw username.
