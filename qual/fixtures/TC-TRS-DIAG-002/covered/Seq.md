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
  e-send:
    ref: Cmd::sendCmd
    source: ll-a
    target: ll-b
    kind: message
---

Sequence diagram for `Cmd`. The `e-send` edge covers `Cmd::sendCmd` — must NOT raise W080.
