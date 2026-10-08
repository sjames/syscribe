---
id: REQ-TRS-VIS-023
type: Requirement
name: The browser creates a new Diagram element from a dialog, derived from a subject or blank, and opens it for editing
status: verified
reqDomain: software
verificationMethod: test
---

The web UI **shall** offer a **New diagram** control in the model browser, visible whether or not
a diagram is open, opening a dialog that collects a name (basic-name grammar), a kind that has a
generator (`BDD`, `IBD`, `StateMachine`, `Action`, `Sequence`, `Requirement`, `Allocation`), a
start-from choice (*derive from a subject* or *blank*), a subject — required when deriving,
optional when blank, with suggestions limited to the element types spec §8.16.8 allows for the
kind — and a package (default `Diagrams` when present, else the model root). Submitting
**shall** create the `Diagram` element through `POST /api/elements` with `diagramKind`, `subject`
when given, and `shapes: {}` for a blank diagram, then open it in a tab. A refusal **shall** be
shown in the dialog with the engine's reason and write nothing; warnings the new element raises
**shall** be shown after creation.

**Source:** `REQ-TRS-VIS-023` (product model).

**Acceptance criteria:** (a) a derived diagram request carries only `diagramKind` and `subject`
and the server serves it as a generated graph flagged `derived`; (b) a blank request carries
`shapes: {}`, is served as an empty manifest graph and then accepts an Add (element, shape and
pin in one write); (c) a name already in use is refused with a reason and the existing file is
unchanged; (d) a subject of the wrong type raises `W418` and an unresolved subject `W401`, both
returned to the dialog; (e) a diagram in a nested package is written under that package; (f) the
form logic refuses a bad name, an unknown kind, a missing or non-candidate subject, and builds the
documented request for the model root and nested packages; (g) the page script never shows a
stylesheet-hidden element by clearing its inline display.
