---
id: REQ-TRS-VIS-019
type: Requirement
name: "A new Action diagram kind: the generator derives action steps, control nodes and successions from an ActionDef or Action subject"
status: verified
reqDomain: software
verificationMethod: test
---

Spec §8.16 **shall** gain `diagramKind: Action` (an action-flow view, SysMLv2's action
definition diagram) with shape kinds `action`, `fork`, `join`, `decision`, `merge`, `initial`,
`final`, `note` and edge kinds `succession` and `flow`. For a derived `Action` diagram whose
subject is an `ActionDef` or `Action` (an `Action` subject reads its `ActionDef`), the generator
**shall** produce:

- one `Action` node per `subActions:` entry (spec §8.7.3), labelled by `name`, stereotyped by
  its kind (`action`, `perform`, `send`, `accept`, `assign`, `terminate`) with a compartment
  line for `typedBy` (last segment), `payload` (send/accept) and `via`/`to` chains;
- an `IfAction` as a `Decision` node labelled by its `condition`, its `then`/`else`
  sub-actions as nodes, `Succession` edges from the decision to the first action of each branch
  labelled `[then]`/`[else]`, and a `Merge` node that the last action of each branch succeeds
  to;
- a `LoopAction` as a container `Action` node stereotyped `loop` (`while`/`until`/`for`),
  labelled by its `name` and condition, holding its `body` sub-actions with successions in
  body order;
- one `Fork`/`Join`/`Decision`/`Merge` node per `controlNodes:` entry (and per `subActions`
  entry of those kinds);
- one `Succession` edge per `successionConnections:` entry (`after` → `before`, labelled by its
  `guard` when present), and one `Flow` edge per `flowConnections:` entry;
- an `Initial` node with a succession to every action that has no incoming succession, and a
  `Final` node reached from every action with no outgoing one, only when the subject declares
  at least one succession (otherwise the steps are drawn unordered).

Shape ids **shall** be `derived_shape_id("<subject>::<stepName>")` (nested branch/body names
included in the path); control nodes likewise. Layout hints: layered, top-to-bottom. A subject
of any other type **shall** raise `W418` and produce no nodes; `include:`/`exclude:` **shall**
apply to the top-level steps and control nodes by name or qualified name.

**Source:** `REQ-TRS-VIS-019` (product model).

**Acceptance criteria:** through the real walker and validator, on a fixture `ActionDef`
(`Beh::Mission`: `takeoff`/`land` perform-actions, a `checkWeather` if/else with a send and a
perform branch, a `navigate` for-loop whose body is an accept with a change trigger and a bare
action, `start` fork and `end` join control nodes, five successions one of which carries a
guard, and one flow between pins), (a) the validator raises no `W400`/`W417`/`W418` for the
`Action` kind and the golden IR matches `tests/vis_snapshots/derived/mission_action.json`;
(b) steps carry `perform`/`send`/`accept` stereotypes and compartments `: Takeoff`,
`send Command`/`via ctrlOut`, `accept Fix`/`when near(wp)`, and a bare action has none;
(c) the if is a `Decision` labelled `wind > 12` with `[then]`/`[else]` successions to its
branch steps, both branches succeeding to a `Merge`, and declared successions enter at the
decision and leave from the merge with `[ok]` as the guard label; (d) the loop is a container
stereotyped `loop` labelled `navigate [for wp in waypoints]` holding `awaitArrival` then
`advance` with a succession between them; (e) `start` is a `Fork`, `end` a `Join`, the declared
successions and the pin-chain flow are edges with deterministic ids; (f) an `Initial` node is
first and feeds only the fork, the join alone reaches the `Final` node, and an `ActionDef` with
no successions has neither; (g) `include: [takeoff, Beh::Mission::land, ghost]` keeps two steps
and no control node with one `W417` naming `ghost`, and `exclude: [checkWeather]` removes the
decision, its branches, its merge and every edge touching them; (h) a `PartDef` subject yields
no nodes and one `W418` naming the type; (i) `render_mermaid` returns a `flowchart TD` with
`{{ }}` diamonds, `[[ ]]` bars and `[then]` labels, `render_svg` lays the graph out with
`decision`/`fork`/`join`/`merge`/`initial`/`final` classes, and `render_plantuml` declines the
kind.
