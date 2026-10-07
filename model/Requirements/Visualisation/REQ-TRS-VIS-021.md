---
type: Requirement
id: REQ-TRS-VIS-021
name: "The Sequence generator derives lifelines, messages and fragments from an ActionDef subject's send and accept actions, and places them itself"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-015]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - sequence
---

For a derived `diagramKind: Sequence` whose subject is an `ActionDef` or `Action` (or a
`UseCaseDef`/`UseCase` with `actors:`), the generator shall produce:

- one `Lifeline` node per participant, in first-appearance order: the subject itself first, then
  every element a `SendAction`'s `to:` chain or an `AcceptAction`'s/`SendAction`'s `via:` chain
  resolves to (a port chain resolves to the part that owns the port; an unresolved chain becomes
  a lifeline labelled by the chain text, dashed), then each entry of the subject's `actors:` as an
  `Actor` node;
- one `Message` edge per `SendAction` (from the subject's lifeline to the `to:`/`via:` participant,
  labelled by the payload's last segment and the action name) and one per `AcceptAction` (from
  the `via:` participant to the subject's lifeline, labelled by payload or trigger), in execution
  order: `successionConnections:` topological order when declared, else declaration order,
  descending into `IfAction` `then`/`else` and `LoopAction` `body`;
- one `Fragment` node per `IfAction` (`alt`, labelled by its condition) and `LoopAction` (`loop`,
  labelled by its condition), spanning the messages it contains;
- an `Activation` node on the subject's lifeline spanning its messages.

Because a sequence diagram's geometry is fixed by order, the generator shall place every node
itself (lifelines left to right at a fixed pitch, messages top to bottom in order, fragments
around their span) as pins, so every renderer draws it with the `fixed` algorithm and no ELK
run. Shape ids are `derived_shape_id("<subject>::<participant|action>")`.

## Rationale

`W080` already checks that a hand-listed sequence diagram covers the subject's send and accept
actions; deriving the diagram makes that check moot for derived views and keeps the picture in
step with the behaviour.

## Scope

- Return messages, creation and destruction are not generated (no model field declares them).
- Nested `PerformAction`s are not expanded into their own definitions' messages.
