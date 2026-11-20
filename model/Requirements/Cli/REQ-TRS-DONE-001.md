---
type: Requirement
id: REQ-TRS-DONE-001
name: "done reports whether a PlanningItem may be called done and exactly what is missing"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - planning
  - agents
---

`syscribe done <PI>` shall judge the model as it would be with the item `done` and report, per check, `pass`, `fail`, `skip` or `handoff`: `steps` (every child item is done), `questions` (none open), `blockers` (all met), `evidence` (no `E719` on the item, no `W310`), `validate` (no errors, without counting the evidence finding twice), `gates` (no `[gates]` stereotype condition or coverage threshold tripped), `trace-code` (clean; `skip` without a `[traceCode]` table) and `status` (`handoff` when `[agents] human_only_status` reserves `done` for a person and the invocation is an agent's). `ready` shall be true when no check fails. `done` shall never write; it exits `0` when ready, `2` when something is missing and `1` for an unknown or non-PlanningItem reference. The MCP tool `done_check` shall return the same data and treat its caller as an agent (GH #485).
