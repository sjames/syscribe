---
id: REQ-TRS-VIS-021
type: Requirement
name: The Sequence generator derives lifelines, messages and fragments from an ActionDef subject's send and accept actions, and places them itself
status: verified
reqDomain: software
verificationMethod: test
---

For a derived `diagramKind: Sequence` whose subject is an `ActionDef` or `Action` (or a
`UseCaseDef`/`UseCase` with `actors:`), the generator **shall** produce:

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

Because a sequence diagram's geometry is fixed by order, the generator **shall** place every node
itself (lifelines left to right at a fixed pitch, messages top to bottom in order, fragments
around their span) as pins, and give every message edge horizontal waypoints from the source
stem to the target stem at its row, so every renderer draws it with the `fixed` algorithm and no
ELK run. Shape ids are `derived_shape_id("<subject>::<participant|action>")`. A subject of
another type is `W418`; `include:`/`exclude:` filter participants by qualified or short name.
`W080` (manifest completeness) is not raised on a derived Sequence diagram.

**Source:** `REQ-TRS-VIS-021` (product model).

**Acceptance criteria:** through the real walker and validator on a fixture `ActionDef` with two
perform steps, a send `to` a part, an accept `via` a port, an `IfAction` with a send in each
branch (one to an unresolvable chain), a `LoopAction` with a body send, successions that reorder
the declaration, and `actors:`: (a) the IR matches its golden snapshot; (b) the lifelines are
the subject, then the port's owning part, the `to:` part, a dashed lifeline for the unresolved
chain, then the actor; (c) the messages follow the succession order and descend into the
branches and the body, each labelled `name(Payload)`; (d) the `loop` and `alt` fragments enclose
exactly their messages; (e) the activation is a child of the subject's lifeline spanning the
first to the last row; (f) every node is pinned, every message has two horizontal waypoints at
the stems, `is_fully_pinned()` holds and the pinned path places every node and edge; (g) a
`PartDef` subject is one `W418` and an empty graph; (h) `include:`/`exclude:` keep or drop
participants by qualified or short name, never the subject, with `W417` for a stray entry; (i)
the derived diagram raises no `W080` while a manifest of the same subject still does; (j) the
SVG writer draws it through the pinned path (dashed stems, a stick-figure actor, the activation,
the fragment tab, the message along its row) and the Mermaid and PlantUML writers accept it.
