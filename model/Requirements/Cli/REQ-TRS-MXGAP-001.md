---
type: Requirement
id: REQ-TRS-MXGAP-001
name: "The coverage matrix lists each gap with the action and ids needed to close it"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

`matrix --json` and the MCP `coverage_matrix` tool shall carry, beside the grid, a `gaps` list with one entry per cell that needs work — state `gap`, `planned`, `failing` or `open` — naming the requirement and its file, the configuration (null in the flat view), the action to take (`create-test`, `activate-test`, `fix-failing-test`, `decide-binding`, `widen-test`), the tests already linked with their status and file, and, for `create-test`, the next free test id in the requirement's area (GH #424).

The property is part of the published `matrix` schema; it is additive, so the schema version is unchanged.
