---
id: REQ-TRS-VIS-024
type: Requirement
name: The browser adds an existing model element to a manifest diagram, unpinned so ELK places it, without creating or changing the element
status: verified
reqDomain: software
verificationMethod: test
---

The diagram toolbar **shall** offer an **Add existing element** action opening a picker that
searches the model's elements by qualified name or name (case-insensitive substring, prefix
matches first, a bounded list, diagrams excluded). Choosing one **shall** post it to
`POST /api/diagrams/shapes/{*qname}` with `{ref, x?, y?}`, which **shall** resolve `ref`, write
`shapes.<id> = {ref, kind}` — the kind taken from the element's type, the id derived from its
qualified name and made unique — and a `layout.<id>` pin only when `x`/`y` are given, in one
guarded write that never touches the element's file. It **shall** refuse an unresolved `ref`, a
`Diagram` as the element, an element already on the diagram, a **derived** diagram (with a reason
pointing at `include:`/`exclude:`) and a target that is not a `Diagram`.

**Source:** `REQ-TRS-VIS-024` (product model).

**Acceptance criteria:** (a) with `x`/`y` the shape is listed and pinned there and the element file
is byte-identical afterwards; (b) without them it is listed and not pinned, and the served graph
sends no position for it; (c) a second add of the same element is refused naming the existing
shape and writes nothing; (d) a derived diagram, an unresolved ref, a diagram as the element and a
non-diagram target are each refused with a reason and write nothing; (e) the picker's search
excludes diagrams, ranks exact and prefix matches first and bounds its results; (f) the request
built from a choice carries only the qualified name, and a missing, unknown or diagram choice is
refused before any request is sent.
