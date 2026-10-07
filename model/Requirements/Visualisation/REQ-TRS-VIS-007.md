---
type: Requirement
id: REQ-TRS-VIS-007
name: "The browser lays diagrams out with ELK through sprotty-elk, honours pins, and never writes an automatic layout back implicitly"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - sprotty
  - elk
---

The sprotty client shall bind sprotty's model-layout hook to `sprotty-elk`'s `ElkLayoutEngine`
backed by the `elkjs` bundled build, both vendored into the existing esbuild bundle and served
from the binary with no CDN and no runtime Node. Client-side bounds measurement shall be enabled
so label sizes are measured before ELK runs.

Pinned nodes (the `pinned` set of `REQ-TRS-VIS-006`) shall keep their positions: the client
shall pass pinned positions to ELK with interactive layering and crossing-minimisation so
unpinned nodes are placed around them, and shall use ELK's fixed algorithm (edge routing only)
when every node is pinned.

An automatic layout result shall never be written to the model unless the user acts: a drag
pins the dragged node through the existing PATCH, *Pin all* writes every node's current
position in one PATCH, and *Auto-layout* clears every pin through the DELETE route and
re-runs ELK.

## Rationale

Layout in the browser was the user's choice (ELK via sprotty, 2026-10-07). Writing ELK's
output back on every open would make `git diff` on diagram files meaningless and overwrite a
human's placement with a machine's; pins are explicit, reviewable intent.

## Scope

- `frontend/package.json` gains `sprotty-elk` and `elkjs` at versions compatible with the
  installed `sprotty` 1.4; the bundle size increase is accepted (`ADR-SYS-VIS-001`).
- ELK runs on the main thread; moving it to a Web Worker is an optimisation, not a requirement.
