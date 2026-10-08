---
type: Requirement
id: REQ-TRS-VIS-026
name: "Selecting a shape or edge in the diagram editor shows a rendered view of the depicted element's Markdown in a side panel"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - sprotty
---

Clicking a node, port or edge in an open diagram shall show the depicted element in a **side
panel** over the diagram, without leaving it and without a modal. The panel shows, from a new
route `GET /ui/element-card/{*qname}`:

- the element's name, type, qualified name, stable id and status;
- the **body of its Markdown file rendered** by the same renderer the detail panel uses
  (headings, lists, tables, code, links, Mermaid blocks);
- a control that opens the full detail dialog (with editing) for the element.

The panel updates on every new selection of exactly one shape or edge that has a `ref`; selecting
several leaves it as it is; the connect gesture never changes it. It has a close control, and the
next selection reopens it. A `ref` that names an **inline feature** (a port, part usage or
attribute declared in an owner's `features:`) resolves to its nearest owning element: the card
labels it "feature of `<owner>`", lists the feature's declared properties (type, `typedBy`,
direction, multiplicity, unit) and shows the owner's rendered body. A `ref` that resolves to
nothing shows a card saying so, with the reference text, never an error page.

## Rationale

A diagram says what is connected to what; the Markdown body says what each thing is for. Reading
that today means leaving the diagram for the model browser. Showing it on click makes the
diagram the entry point to the model's documentation.

## Scope

- Reading only: editing stays in the full detail dialog.
- The panel overlays the diagram's right-hand edge; resizing and docking are not part of this.
