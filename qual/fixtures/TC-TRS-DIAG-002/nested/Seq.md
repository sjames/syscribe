---
type: Diagram
name: Seq
diagramKind: Sequence
subject: Cmd
shapes:
  ll-a:
    ref: Cmd
    kind: lifeline
  ll-b:
    ref: Cmd
    kind: lifeline
edges:
  e-init:
    ref: Cmd
    source: ll-a
    target: ll-b
    kind: message
---

Sequence diagram for `Cmd`. Omits an edge for the nested `Cmd::abort` SendAction — must raise W080.
