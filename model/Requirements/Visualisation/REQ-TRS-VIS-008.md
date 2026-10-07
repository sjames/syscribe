---
type: Requirement
id: REQ-TRS-VIS-008
name: "Create, delete, port-aware connect and move work on manifest and derived diagrams through the guarded-write engine"
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

The editing gestures of `REQ-TRS-DE-004` shall work on every IR-backed diagram:

| Gesture | Manifest diagram | Derived diagram |
|---|---|---|
| Create node | creates the element, its manifest shape and a pin in one guarded write | creates the element under the subject (a `Part` usage for IBD, a definition for BDD); the view regenerates on reload, no manifest write |
| Delete node | deletes the element and prunes its shapes and edges from every diagram | deletes the element; the view regenerates |
| Connect | adds a `connections:` entry on the owning part and an edge to the manifest | adds the `connections:` entry only |
| Move | writes a pin | writes a pin |

The connect gesture shall be **port-aware**: the drag starts and ends on `port` children. A
drag between two blocks shall be accepted only when each block has exactly one port whose
direction is compatible with the other's, and refused with an explanatory toast otherwise.
Every write shall go through `syscribe_model::mutate`'s guarded-write engine and surface the
`WriteResponse` delta; a refusal shall revert the optimistic change, as today.

## Rationale

Full editing was the user's choice. Today's connect gesture joins any two nodes and writes
`connections:` entries between parts that have no ports, which the validator then rejects;
making the gesture port-aware removes a whole class of refused edits.

## Scope

- `ADR-SYS-DE-001`'s transactional diagram-sync rule still applies to manifest diagrams; on a
  derived diagram the model edit is the whole transaction.
- Rename and re-type from the diagram remain out of scope.
