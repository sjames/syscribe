---
type: Requirement
id: REQ-TRS-VIS-019
name: "A new Action diagram kind: the generator derives action steps, control nodes and successions from an ActionDef or Action subject"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-015]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - action
---

Spec §8.16 shall gain `diagramKind: Action` (an action-flow view, SysMLv2's action definition
diagram) with shape kinds `action`, `fork`, `join`, `decision`, `merge`, `initial`, `final`, `note`
and edge kinds `succession` and `flow`. For a derived `Action` diagram whose subject is an
`ActionDef` or `Action` (an `Action` subject reads its `ActionDef`), the generator shall produce:

- one `Action` node per `subActions:` entry (spec §8.7.3), labelled by `name`, stereotyped by its
  kind (`action`, `perform`, `send`, `accept`, `assign`, `terminate`) with a compartment line for
  `typedBy` (last segment), `payload` (send/accept) and `via`/`to` chains;
- an `IfAction` as a `Decision` node labelled by its `condition`, its `then`/`else` sub-actions as
  nodes, `Succession` edges from the decision to the first action of each branch labelled
  `[then]`/`[else]`, and a `Merge` node that the last action of each branch succeeds to;
- a `LoopAction` as a container `Action` node stereotyped `loop` (`while`/`until`/`for`),
  labelled by its `name` and condition, holding its `body` sub-actions with successions in body
  order;
- one `Fork`/`Join`/`Decision`/`Merge` node per `controlNodes:` entry (and per `subActions` entry
  of those kinds);
- one `Succession` edge per `successionConnections:` entry (`after` → `before`, labelled by its
  `guard` when present), and one `Flow` edge per `flowConnections:` entry;
- an `Initial` node with a succession to every action that has no incoming succession, and a
  `Final` node reached from every action with no outgoing one, only when the subject declares at
  least one succession (otherwise the steps are drawn unordered).

Shape ids are `derived_shape_id("<subject>::<stepName>")` (nested branch/body names included in
the path); control nodes likewise. Layout hints: layered, top-to-bottom.

## Rationale

Behaviour diagrams are the second thing a reviewer asks for after structure, and the demo
`MissionExecution` already carries forks, joins, a conditional and a loop that only prose
currently describes.

## Scope

- The `Action` kind is manifest-authorable too; its shape and edge vocabulary is added to
  `NodeKind`/`EdgeKind` and the §8.16.8 tables.
- Parameter pins and object-flow items are rendered as compartment text, not as separate nodes.
