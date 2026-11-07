---
type: Requirement
id: REQ-TRS-ITEMREF-002
name: "Prose references to requirement items and PlanningItem questions are resolved and checked"
status: draft
reqDomain: software
reqClass: system
tags:
  - requirements
  - references
---

`validate` shall resolve item and question references written in element bodies (GH #453).

1. {#grammar} **Grammar.** Outside fenced code: `REQ-X#slug` (item by anchor), `REQ-X#5` (item by position), `REQ-X item 5` (legacy form, `item` case-insensitive) and `PI-Y#Q31` (a PlanningItem question).
2. {#scope} **Scope.** A reference is checked only when its element part resolves, by stable id or qualified name, to a Requirement (item forms) or a PlanningItem (`#Q<n>`). An unresolved element id is left to the existing tooling and not reported again.
3. {#dangling} **Dangling references.** A reference to an anchor the requirement does not define, an item number beyond its item count, or a question the item does not carry is `W092` on the citing element.
