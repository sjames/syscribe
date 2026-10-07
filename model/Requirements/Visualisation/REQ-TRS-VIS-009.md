---
type: Requirement
id: REQ-TRS-VIS-009
name: "PlantUML and Mermaid are written from the IR and reachable through diagram export and MCP render_diagram"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - plantuml
  - mermaid
---

The PlantUML writer shall take the IR as its input, replacing `plantuml.rs`'s private manifest
parser, and its output for every `pumlMode: companion` diagram in the demo model shall be
captured as a snapshot before the swap and matched after it. A Mermaid writer shall be added,
mapping BDD to `classDiagram` and IBD to `flowchart` with `subgraph` nesting for the boundary
and owned parts, and emitting a `%% ref: <QualifiedName>` annotation for every node so the
existing `W408`/`W409` lints apply to generated text exactly as to hand-written blocks.

A new `syscribe diagram export <qname> --format plantuml|mermaid|svg [--out <file>]`
subcommand shall write the chosen output for one `Diagram` element, and MCP `render_diagram`
shall accept `format: mermaid` alongside `plantuml`. An unresolvable `<qname>` shall print
`error: element '<qname>' not found` on stderr, write nothing to stdout and exit `1`; this
replaces the not-found rule of the retired `REQ-TRS-DIAG-004`.

`syscribe plantuml` and `plantuml render` keep their interface; only their input becomes the
IR.

## Rationale

The user chose to keep both text backends. Deriving them from the IR removes the last
duplicate parser and gives Mermaid, which today is pass-through only, a generator for the
first time.

## Scope

- Snapshot tests for both writers on the fixture IRs; the PlantUML before/after snapshot is
  the proof the parser swap preserved behaviour.
- Neither writer carries ELK positions; PlantUML and Mermaid lay themselves out.
