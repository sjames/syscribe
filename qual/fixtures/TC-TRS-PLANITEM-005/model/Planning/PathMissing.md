---
type: PlanningItem
id: PI-P5-PATHMISSING-001
name: "Missing local path: entry"
status: todo
achieves: REQ-P5-001
evidence:
  - path: docs/does-not-exist.txt
---

A `path:` entry naming a local file that does not exist, and carrying no `rationale:` — must be
rejected.
