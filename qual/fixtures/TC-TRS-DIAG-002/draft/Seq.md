---
type: Diagram
name: Seq
diagramKind: Sequence
status: draft
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

Draft sequence diagram for `Cmd`. Uncovered `Cmd::sendCmd`, but `status: draft` suppresses W080.
