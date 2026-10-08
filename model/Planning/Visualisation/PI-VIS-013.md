---
type: PlanningItem
id: PI-VIS-013
name: "New diagram dialog in the browser — derived or blank, with subject suggestions per kind"
status: todo
itemType: feature
achieves: [REQ-TRS-VIS-023]
tags:
  - visualisation
---

Tab-bar control and dialog in `templates/index.html`/`base.html`, pure form-to-request logic in
`frontend/src/new-diagram.ts` with a Node test, subject-type suggestions per kind, server
integration tests on `POST /api/elements` for a derived and a blank diagram, qualification mirror
`REQ/TC-TRS-VIS-023`, guide paragraph in `docs/browser/index.md`.
