---
type: Requirement
id: REQ-TRS-VIS-000
name: "Diagrams are derived from the model, laid out automatically, editable in the browser and exportable, all from one representation"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - diagram
  - visualisation
---

Syscribe shall provide a single visualisation mechanism in which every diagram is built as one
intermediate representation from the model — either from an author's shape/edge manifest or
derived from a subject element — laid out automatically with correct SysMLv2 notation, editable
in the web browser as a true authoring surface, and exportable as PlantUML, Mermaid and static
SVG from that same representation, so that a user never has to place boxes by hand to see a
well-formed diagram and never sees two renderings of the same diagram disagree.

## Rationale

A systems model that cannot be seen is only half a model. Today Syscribe has five unrelated
rendering paths that share no data model, no layout and no styling; diagrams are hand-listed,
render as a pile at the origin without a hand-written `layout:`, and the shared manifest parser
cannot even read the spec's own shorthand. Engineers judge a modelling tool by its pictures, and
the pictures are the artefact that ends up in reviews, documents and pull requests.

## Scope

- In scope: one Diagram IR in `syscribe-model`; manifest and derived sources; BDD and IBD
  generators first; ELK layout in the browser via sprotty; full create/delete/connect/move
  editing through the existing guarded-write engine; PlantUML, Mermaid and SVG writers off the
  IR; a static-export policy for docs; removal of the legacy rendering paths.
- Out of scope: a Rust layout engine; server-side ELK; pixel parity between the browser and
  the text exporters; changes to hand-authored Mermaid or inline PlantUML diagrams.
- Backwards compatibility with existing rendered output or the CLI `diagram` toolkit is
  explicitly not required (user decision, 2026-10-07).
- The architectural choices (ELK in the browser, IR, removal rather than coexistence) are
  recorded in `ADR-SYS-VIS-001`, not here.
