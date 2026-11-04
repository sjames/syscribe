---
type: Requirement
id: REQ-TRS-PISTEP-001
name: "steps shows a PlanningItem's breakdown as a tree with a roll-up"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
---

`syscribe steps <PI>` shall render a `PlanningItem`'s child items — its steps — as a tree with a derived roll-up, with no new model fields (GH #436, `ADR-SYS-PLANITEM-001` addendum).

## Behavior

- Each node shows id, name, status, `claimedBy`/`assignedTo`, what it is `blockedBy`, and — for a leaf — whether it has evidence; the header gives `N of M leaves done` and the counts per status.
- `--open` hides finished subtrees, `--depth N` limits depth, `--json` emits the tree; an unknown item exits 1.
- The parent's status is not derived from its children. The view notes disagreement (all leaves done but the item not `done`; `done` with unfinished leaves) as a hint only, and raises no validation finding.
- MCP exposes the same data as a read-only `steps` tool.
