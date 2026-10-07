---
type: Requirement
id: REQ-TRS-VIS-011
name: "The browser offers Pin all, Auto-layout and Save companion SVG, each a single guarded write"
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

The diagram panel shall offer three actions beside the existing *Add*, *Connect* and *Delete*:

- **Pin all** — writes every node's current position (and size, when ELK sized it) as pins in
  one `PATCH /api/diagrams/layout/{qname}`;
- **Auto-layout** — clears every pin with `DELETE /api/diagrams/layout/{qname}` and re-runs
  ELK;
- **Save companion SVG** — serialises the current render as an SVG conforming to spec §8.16.5
  (`sysml:ref` attributes, kind classes) and writes it to the diagram's `svgFile:` (default
  `<stem>.svg`), setting `svgMode: companion` when absent, through the guarded-write engine.

Each action shall surface the `WriteResponse` delta and revert its optimistic state on
refusal, as the existing gestures do.

## Rationale

These three buttons are the whole bridge between "the browser is the layout authority" and
the static outputs of `REQ-TRS-VIS-010`: they are how a human's approved picture reaches
GitHub, MkDocs and `export-html`.

## Scope

- Whether *Save companion SVG* also writes pins is an open design point
  (`docs/design/visualisation.md` §13); either answer satisfies this requirement.
- The saved SVG embeds no external fonts; it uses the same generic font stack as the writer.
