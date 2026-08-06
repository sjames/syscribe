---
type: PlanningItem
id: PI-ERR-NOEV-001
name: "Leaf marked done with no evidence at all"
status: done
itemType: task
achieves: REQ-ERR-DEMO-001
---

**Deliberate error demonstration 1 of 2.** A leaf `PlanningItem` (no
`parent:`, no `children:` naming it) at `status: done` with **no `evidence:`
field at all**. `REQ-TRS-PLANITEM-006` requires at least one non-waived,
resolving `evidence:` entry on a leaf at `status: done` — expect:

```
E719  leaf PlanningItem is `status: done` but has no non-waived, resolving `evidence:` entry
```
