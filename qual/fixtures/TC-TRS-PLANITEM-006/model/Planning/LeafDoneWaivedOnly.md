---
type: PlanningItem
id: PI-P6-LEAFWAIVED-001
name: "Leaf, done, only rationale-waived evidence"
status: done
achieves: REQ-P6-001
evidence:
  - ref: PI-P6-NOPE-999
    rationale: "Tracked externally, not yet modeled here."
  - path: docs/not-written-yet.txt
    rationale: "Report is planned but not written yet."
---

A leaf at `status: done` whose `evidence:` list is non-empty but every entry carries its own
`rationale:` — nothing actually counts as proof, so this must still be rejected.
