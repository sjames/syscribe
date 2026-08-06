---
type: PlanningItem
id: PI-RTH-DESIGN-001
name: "Design RTH trigger logic and flight-path replanning"
status: done
itemType: task
parent: PI-RTH-001
evidence:
  - path: docs/rth-design.txt
  - ref: PI-RTH-REVIEW-999
    rationale: "Design review was conducted verbally with the safety board; a formal ReviewRecord is tracked separately and hasn't been authored in this example yet."
tags:
  - rth
---

Design the battery-threshold debounce logic and the home-position
flight-path replanning algorithm. A **leaf** `PlanningItem` (no `children`)
at `status: done`, so `REQ-TRS-PLANITEM-006`'s leaf-evidence rule requires at
least one non-waived, resolving `evidence:` entry:

- `path: docs/rth-design.txt` — resolves (a real local file in this example
  tree; `REQ-TRS-PLANITEM-005`) and satisfies the rule on its own.
- `ref: PI-RTH-REVIEW-999` — deliberately **dangling** (no such element
  exists in this example), but carries its own `rationale:`, which waives the
  resolution check for this entry (`REQ-TRS-PLANITEM-005`). A waived entry
  never counts toward the "at least one resolving entry" total
  (`REQ-TRS-PLANITEM-006`) even though, as here, it happens to be included
  alongside a genuinely resolving one — the `path:` entry above is what
  actually satisfies the rule.
