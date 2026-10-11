---
type: Requirement
id: REQ-TRS-ACTEXEC-001
name: "An action flow can be lowered to an action-graph IR and run with token semantics, scenarios, coverage and mutation"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - action-flow
---

An executable `ActionDef` shall be lowered to an action-graph IR and run by a token-based interpreter, with Gherkin scenarios, measured coverage and mutation analysis, sharing the machinery of REQ-TRS-SMIR-001, SMEXEC, SMSCEN, SMCOV and SMMUT (GH #276, part of #272).

## Behavior

- **Lowering.** `syscribe compile <ActionDef> [--json]` produces a deterministic, hashed IR: nodes (actions, `fork`, `join`, `decision`, `merge`, the implicit start) in document order with ids, kinds and source pointers; edges (successions, with guard and `else`) in document order; parameter and variable slots; item flows (`flowConnections`) as data copies; `PerformAction` and `typedBy` calls resolved to their callee IR. Cycles of calls, unresolved callees and the checks of REQ-TRS-ACTSCHEMA-001 are lowering diagnostics.
- **Token semantics** (UML activity / fUML, SysML v2 actions). The action starts with a token on every node that has no incoming succession. A node fires when every incoming edge holds a token (a `merge` fires on any one): it consumes them, executes, then offers a token on its outgoing edges. An action offers one on every outgoing edge whose guard holds (no guard: always); a `fork` on all; a `join` fires once all inputs hold; a `decision` on the first outgoing edge whose guard holds, else its `else` edge; a guard that selects no edge from an action leaves the token consumed and is a note (an error in strict mode). Ties between simultaneously enabled nodes are resolved in document order; strict mode reports two simultaneously enabled assignments that write the same variable (a race).
- **Node execution.** `AssignmentAction` runs its `assign:`. `IfAction` runs `then` or `else` as nested sequences. `LoopAction` runs its `body` while its `condition:` holds, or (`until`) once and then until it holds, at most 1000 iterations (an error beyond). `PerformAction`/`typedBy` runs the callee to completion with `parameters:` bound as inputs and makes its outputs readable as `<subaction>.<output>`. `SendAction` records the sent event and payload in the trace. `AcceptAction` waits for an injected event of its `payload`/`via` (binding `as:`), or for its `after:` duration on the virtual clock. `TerminateAction` ends the whole action. A generic `Action` with no body completes at once.
- **Object flow.** A `flowConnections` entry `{from: a.p, to: b.q}` copies the value of `a.p` to `b.q` when `a` completes; `bindingConnections` assign an output of a sub-action to a parameter of the action when it completes. Values are typed and unit-checked like state-machine variables; an assignment of a different type or dimension stops the run.
- **Completion.** The run completes when no token is left and nothing is running; it is *waiting* while an `AcceptAction` is enabled and has no event; it is *terminated* by a `TerminateAction`; it is *deadlocked* when tokens remain on successions but no node is enabled (a node with several incoming successions needs a token on each; use a `merge` to join alternatives). Outputs are the `out` parameters at that moment. Errors stop the run with the error recorded as a final `error` trace step; more than 10000 node firings is a livelock error.
- **Trace.** Each firing is recorded: virtual time, node, tokens consumed and offered, variables changed, events sent. The trace is deterministic and byte-identical on a re-run.
- **Scenarios.** A `TestCase` whose `subject:` is an `ActionDef` uses the closed vocabulary `Given <parameter or variable> is <expression>`, `When the action runs`, `When <event> is injected [with …]`, `When <duration> elapses`, `Then <boolean expression>` (over parameters, variables and outputs), `Then <action> ran`, `Then <action> did not run`, `Then the action completed|is waiting|terminated`; anything else is `E886`. Verdicts, `--record` evidence and `simulate <ActionDef> --do "<script>"` work as for state machines.
- **Coverage.** `simulate --coverage` reports nodes fired, edges taken, decision branches (every outgoing edge of a decision taken), loop bodies (run zero and at least once) and guard conditions with MC/DC, with the gates `--min-nodes`, `--min-edges`, `--min-guards`.
- **Mutation.** `mutate <ActionDef>` seeds: negate a guard or condition, relational operator and literal changes, remove a succession, remove a sub-action (it completes without effect), swap two decision branches, remove an assignment or replace it by a self-assignment, retarget a succession; scored as for state machines.
- **Connection to state machines.** An `entryAction`, `doAction`, `exitAction` or `effect` that names an executable `ActionDef` runs it to completion at that point of a state-machine run, with its outputs written to the machine's variables of the same name; its trace is nested in the step.
