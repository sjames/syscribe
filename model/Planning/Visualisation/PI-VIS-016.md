---
type: PlanningItem
id: PI-VIS-016
name: "Element panel: clicking a diagram shape or edge shows the element's rendered Markdown"
status: done
itemType: feature
achieves: [REQ-TRS-VIS-026]
evidence:
  - path: repo:crates/syscribe-server/tests/element_card.rs
  - path: repo:crates/syscribe-server/frontend/src/selection-ref.ts
  - path: repo:crates/syscribe-server/frontend/test/selection-ref.test.mjs
tags:
  - visualisation
---

`GET /ui/element-card/{*qname}` and `templates/element_card.html` (feature fallback, unresolved
card, Mermaid), selection-to-panel wiring in the client with its pure selection logic and Node test,
panel markup and styles, server tests, qualification mirror `REQ/TC-TRS-VIS-026`, browser guide.
