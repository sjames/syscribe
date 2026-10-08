---
type: PlanningItem
id: PI-VIS-017
name: "Planning dashboard: live status board of PlanningItems, who is working on what"
status: done
itemType: feature
achieves: [REQ-TRS-VIS-027]
evidence:
  - path: repo:crates/syscribe-server/tests/planning_dashboard.rs
  - path: repo:crates/syscribe-server/src/planning.rs
  - path: repo:crates/syscribe-server/frontend/test/planning-core.test.mjs
tags:
  - planning
  - dashboard
---

`GET /planning` and `GET /ui/planning/board` with `templates/planning.html` and
`planning_board.html`, board-building logic with tests, live refresh on the reload WebSocket plus a
timer, header link, styles, qualification mirror, browser guide.
