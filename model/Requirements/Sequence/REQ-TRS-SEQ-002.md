---
type: Requirement
id: REQ-TRS-SEQ-002
name: "A call of an operation is a message with its signature, and a return"
status: draft
reqDomain: software
reqClass: system
tags:
  - diagram
  - sequence
---

A `SendAction` or `PerformAction` that names an operation with `operation:` shall be a message in a derived Sequence diagram, labelled from the operation's signature, followed by a return where the operation has one (GH #365, item 2; GH #398).

## Behavior

- The message goes from the subject to the participant named by the call's `to:` or `via:`, else to the element that declares the operation (its own lifeline).
- The label is `op(name: Type, …)` for the operation's input parameters, followed by `: ReturnType` when it has one. A type is shown by its last segment.
- A synchronous operation with a `returnType` is followed, on the next row, by a dashed return message back to the subject labelled with the return type and, when `errors:` are declared, `! Error1, Error2`. An asynchronous operation, or one with no `returnType`, has no return.
- A call whose `operation:` does not resolve is drawn as before: a `PerformAction` is not a message, a `SendAction` keeps its payload label.
- A diagram with no `operation:` call is unchanged byte for byte.
