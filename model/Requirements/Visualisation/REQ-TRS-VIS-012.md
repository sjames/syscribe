---
type: Requirement
id: REQ-TRS-VIS-012
name: "One visual language, owned in Rust, is shared by the sprotty views and the SVG writer"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
---

`vis::style` shall define the visual language once — per `NodeKind` and element kind the
stereotype text (`«part def»`, `«part»`, `«port»`, …), fill and stroke colours, header
treatment and the abstract (italic) rendering; per `EdgeKind` the stroke, dash pattern and
arrowhead (hollow triangle for inheritance, filled diamond for composition, filled arrow for
flow, dashed with `=` for binding, none for connection); per port direction the glyph
(`in`, `out`, `inout`); and, for an element that applies one or more `MetadataDef`
stereotypes (`REQ-TRS-META-001`), an additional `«Name»` banner per applied stereotype beneath
the kind stereotype, in the same font and colour (this subsumes the retired qualification
requirement `REQ-TRS-META-002`, which verified banners in the deleted CLI renderer). The
sprotty model shall carry the resolved style per node and edge so
the client's views read it rather than hold their own table, and `vis::svg` shall emit the same
values, so a diagram rendered in the browser and the same diagram saved or exported look alike.

## Rationale

Today `views.tsx` mirrors `renderer.rs`'s colours by hand and the two have already drifted.
SysMLv2 readers rely on notation — the arrowheads and stereotypes are semantics, not
decoration — so it has to be right once.

## Scope

- Notation follows spec §8.16.8's tables for BDD and IBD; other kinds inherit the defaults
  until their generators land.
- A `_diagram-symbols.svg` at the model root continues to be injected into saved SVG, as
  §8.16.5 specifies.
