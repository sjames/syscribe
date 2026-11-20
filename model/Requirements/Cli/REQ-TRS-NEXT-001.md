---
type: Requirement
id: REQ-TRS-NEXT-001
name: "next offers the PlanningItem an agent should take, with the reason every other item is not offered"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - planning
  - agents
---

`syscribe next` shall offer the `PlanningItem`s that are `todo` or `in_progress`, unclaimed, leaves (a composite is worked through its steps), have every `blockedBy:` met (a PlanningItem `done`, an ADR `accepted`, a Requirement `implemented` or `verified`; an unresolved entry holds the item back) and no open question reserved to a person; unclaimed `in_progress` items first, then by id. Each candidate shall list its hazards (an open question an agent may answer, overlap with active work, `W311`), and every other item shall be listed with the reason. `next` shall never write; it exits `2` when nothing is offered and `1` on a usage error. The MCP tool `next_item` shall return the same data (GH #485).
