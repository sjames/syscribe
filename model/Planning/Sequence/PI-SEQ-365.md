---
type: PlanningItem
id: PI-SEQ-365
name: "Sequence diagrams from parts and operations (GH #365)"
status: in_progress
itemType: feature
achieves:
  - REQ-TRS-SEQ-001
  - REQ-TRS-SEQ-002
tags:
  - diagram
  - sequence
---

Done: a `PartDef`/`Part` subject derives its sequence from the actions it `performs:` (item 1). Done: a call that names an operation (`operation:`, #398) is a message labelled with its signature, with a return for a synchronous operation that has a return type (item 2, REQ-TRS-SEQ-002). Still to do in #365: async arrowheads, create/destroy; `opt`, `alt` else dividers, `par`, `break` and loop guards (item 3, which also needs the writers and the sprotty client to draw dividers); timeouts (item 4).
