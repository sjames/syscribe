---
type: PlanningItem
id: PI-P5-PERENTRY-001
name: "One waived, one un-waived broken entry"
status: todo
achieves: REQ-P5-001
evidence:
  - path: docs/does-not-exist.txt
    rationale: "Report is planned but not written yet."
  - ref: PI-P5-NOPE-999
---

Two broken entries in one list: the first (a missing `path:`) carries its own `rationale:` and
must not be flagged; the second (a dangling `ref:`) carries no `rationale:` and must be flagged —
proving the waiver is per-entry, not blanket.
