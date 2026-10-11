# Action flows

An `ActionDef` describes behaviour as sub-actions, control nodes and successions (§8.7). With a few additions it is *executable*: it can be compiled to an action-graph IR, run on a virtual clock with token semantics, checked with Gherkin scenarios, measured for coverage and mutated to test the scenarios (GH #276; requirements `REQ-TRS-ACTSCHEMA-001`, `REQ-TRS-ACTEXEC-001`). Everything is additive: an action without `variables:` or `assign:` validates and behaves as before, and its guards stay opaque text.

## A small executable action

```yaml
---
type: ActionDef
name: Classify
parameters:
  - {name: level, direction: in}
  - {name: verdict, direction: out}
variables:
  - {name: margin, value: "0"}
subActions:
  - {name: measure, kind: AssignmentAction, assign: "margin := level - 10"}
  - {name: high, kind: AssignmentAction, assign: "verdict := 1"}
  - {name: low, kind: AssignmentAction, assign: "verdict := 0"}
controlNodes:
  - {name: d, kind: DecisionNode}
successionConnections:
  - {after: measure, before: d}
  - {after: d, before: high, guard: "margin > 0"}
  - {after: d, before: low, guard: else}
---
Classifies a level against a threshold of 10.
```

* `parameters:` and `variables:` are the action's data. A name in a guard, a `condition:` or an `assign:` is a parameter, a variable, an `as` binding of an `AcceptAction`, or `<sub-action>.<output parameter>` of a performed action.
* `AssignmentAction` carries `assign: "name := expression; …"` in the expression language (`syscribe help eval`).
* A `DecisionNode` takes the first outgoing succession whose `guard:` holds, else the one marked `guard: else`.
* `IfAction` (`condition`, `then`, `else`) and `LoopAction` (`loopKind: while|until`, `condition`, `body`) run nested blocks; a loop is limited to 1000 iterations.
* `PerformAction` (or an action with `typedBy:`) runs another `ActionDef` to completion; its `parameters:` bind the callee's inputs (`{name, value: <expression>}`), and `flowConnections: [{from: a.p, to: b.q}]` and `bindingConnections` copy values when the source completes.
* `SendAction` records the event it sends; `AcceptAction` waits for an event (`payload`, `via`, `as`) or for `after: <duration>` on the virtual clock; `TerminateAction` ends the whole action.

Type and unit errors in an action's expressions (a succession guard that is a number, a loop condition comparing metres with a plain number, an assignment of the wrong dimension) are found without running anything and reported as `E935`; parameters without a default, sub-action pins and accept bindings are never errors (`docs/design/static-typing.md`).

## Semantics

Token semantics as in UML activities and fUML. The action starts with a token on every node that has no incoming succession (or is named after `start`). A node fires when every incoming succession holds a token (a `merge` fires on any one); an action then offers a token on every outgoing succession whose guard holds, a `fork` on all, a `join` once all its inputs are present, a `decision` on one. Enabled nodes fire in document order (sub-actions first, then control nodes), so parallel branches run deterministically; the trace records each firing. The run *completes* when no token is left and nothing waits, is *deadlocked* when tokens remain but nothing is enabled (a node with several incoming successions needs a token on each — join alternatives with a `merge`), is *waiting* while an `AcceptAction` has no event, and is *terminated* by `TerminateAction`.

Mutation analysis shows where this ordering matters: removing the succession into a decision node survives when the node before it is declared first, because the tie-break runs it first anyway.

## Running it

```bash
syscribe -m model compile Classify                       # the IR, with hash
syscribe -m model simulate Classify --do "set level=14; run"
syscribe -m model simulate Classify --coverage           # all scenario TestCases of the action
syscribe -m model mutate Classify --min-score 90
```

A `TestCase` with `subject: Classify` is executable when its steps use the closed vocabulary:

| Step | Meaning |
|---|---|
| `Given level is 14` | set an input parameter or variable |
| `When the action runs` | start it |
| `When Pkg::Go is injected with v = 7` | send an event to a waiting `AcceptAction` |
| `When 2 s elapses` | advance the virtual clock |
| `Then verdict == 1` | a boolean over parameters, variables and outputs |
| `Then high ran` / `Then low did not run` | whether a node executed |
| `Then the action completed` / `is waiting` / `terminated` / `deadlocked` | the run status |

A step outside it is `E886`. `--record` stores the verdicts as evidence exactly as for state machines.

## From a state machine

An `entryAction`, `doAction`, `exitAction` or `effect` that names an executable `ActionDef` runs it to completion at that point of a state-machine run: its `in` and `inout` parameters are read from the machine's variables of the same name, its `out` and `inout` values written back. An action that waits for an event, or writes a value of the wrong type or dimension, stops the run. The SCXML export lists such actions as notes: they have no SCXML form.

## Checks

`E883`/`E884` (variables and assignments, shared with state machines), `E887` (decision, loop and accept wiring) and `W935` (guards and conditions outside the expression language or naming unknown things) apply to executable actions only.

## Limits

Callee coverage is not merged into the caller's, and mutation seeds only the top-level nodes and successions of the action itself (not nested blocks or callees); an `AcceptAction` inside an `if`/`loop` block cannot suspend and is `E887`; `loopKind: for` is not supported; static types are checked when an expression runs, not when it is compiled.
