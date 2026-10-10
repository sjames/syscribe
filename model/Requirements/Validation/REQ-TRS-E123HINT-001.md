---
type: Requirement
id: REQ-TRS-E123HINT-001
name: "E123 names the remedy and the authoring guide explains attribute def vs item def"
status: draft
reqDomain: software
reqClass: system
tags:
  - validation
  - docs
---

`E123` shall tell the author how to fix the mismatch, and the authoring documentation shall carry a decision guide for data records versus flowing items (GH #254).

## Behavior

- An `Attribute` usage typed by an `ItemDef` (inline feature or standalone usage) gets the remedy "define '<name>' as an `AttributeDef` (a data record or value type), or change the usage to `type: Item` only if it is a flowing item".
- An `Item` usage typed by an `AttributeDef` gets the mirror remedy.
- Any other mismatch names the definition kinds that can type the usage.
- The authoring prompt and the format spec carry a short decision guide: data record / value type -> `attribute def` + `attribute`; thing that flows between parts or is sent/received -> `item def` + `item`; enumeration -> `enum def`; plus the usage-to-definition table.
