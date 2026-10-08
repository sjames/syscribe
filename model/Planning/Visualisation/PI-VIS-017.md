---
type: PlanningItem
id: PI-VIS-017
name: "Planning dashboard: live status board of PlanningItems, who is working on what"
status: todo
itemType: feature
achieves: [REQ-TRS-VIS-027]
tags:
  - planning
  - dashboard
---

`GET /planning` and `GET /ui/planning/board` with `templates/planning.html` and
`planning_board.html`, board-building logic with tests, live refresh on the reload WebSocket plus a
timer, header link, styles, qualification mirror, browser guide.
