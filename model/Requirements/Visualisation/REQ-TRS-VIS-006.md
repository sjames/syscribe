---
type: Requirement
id: REQ-TRS-VIS-006
name: "The diagram-model endpoint serves a nested sprotty graph with ports, labels, compartments, ELK layout options and the pinned set"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - sprotty
---

`GET /api/diagrams/model/{qname}` shall serialise the diagram's IR as a **nested** sprotty
`SGraph`: a `node` per IR node placed inside its parent's `children`, `port` children for
ports (carrying direction), a `label` child for the display name, a `compartment` child for
compartment lines, and an `edge` per IR edge carrying its kind and any pinned
`routingPoints`. `position` shall be present only for pinned nodes and `size` only when the pin
carries `w`/`h`, so the client measures and ELK sizes everything else. The root shall carry a
`layoutOptions` map of ELK option ids derived from the IR's layout hints and a `pinned` list of
node ids.

`PATCH /api/diagrams/layout/{qname}` shall keep its shape and write each entry as a pin; an
entry whose value is `null` shall remove that pin. A new `DELETE /api/diagrams/layout/{qname}`
shall remove every pin of the diagram. Both are guarded writes.

## Rationale

sprotty needs nesting to draw ports on blocks and blocks inside a boundary, and `sprotty-elk`
needs real measured sizes and the layout options to produce a good layout. Carrying the pin set
lets the client distinguish "fixed by a human" from "placed by ELK".

## Scope

- Axum integration tests assert the nesting, the port children, `layoutOptions` and `pinned`
  for a fixture diagram, and the PATCH/DELETE behaviour.
- The route paths are unchanged except for the new DELETE; `docs/browser/index.md` is updated.
