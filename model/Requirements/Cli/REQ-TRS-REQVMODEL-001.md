---
type: Requirement
id: REQ-TRS-REQVMODEL-001
name: "The Requirements Explorer can lay the graph out in V-model columns"
status: draft
reqDomain: software
reqClass: system
tags:
  - server
---

`syscribe-server`'s Requirements Explorer shall offer a V-model layout that places each drawn node in a column by its level instead of by its hop distance (GH #269, phase 7).

## Behavior

- A "Layout" selector chooses `hops` (default, the existing layering) or `v-model`. The V-model columns, left to right, are: 0 stakeholder requirements (`reqClass` `stakeholder`), 1 system requirements (`system`), 2 other requirements (any other or no `reqClass`, e.g. software, hardware), 3 architecture elements, 4 tests (category `tests`), 5 everything else (features, safety, security, planning, ...). Architecture and tests use the categories of REQ-TRS-REQTRACE-001.
- Only columns that contain a node are drawn, each with a header label; relative order is kept. Edges, filters, trace highlighting, selection, table, matrix and export behave as in the hop layout, and the root keeps its emphasis.
- The column of a node is a pure function of the node (`vmodelColumn`), and the layout function accepts it as the column assignment; layout remains overlap-free.
