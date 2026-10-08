---
type: PlanningItem
id: PI-VIS-014
name: "Add an existing element to a manifest diagram from the browser — picker, POST /api/diagrams/shapes route, tests"
status: todo
itemType: feature
achieves: [REQ-TRS-VIS-024]
tags:
  - visualisation
---

`POST /api/diagrams/shapes/{*qname}` in `routes/mutate.rs`, the picker dialog and toolbar button
in `templates/index.html`, pure search/request logic in `frontend/src/add-existing.ts` with a Node
test, server integration tests, qualification mirror `REQ/TC-TRS-VIS-024`, browser-guide and API
table entries.
