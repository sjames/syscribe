---
type: Requirement
id: REQ-TRS-PUML-002
name: "Diagram body references anticipated .svg output from PlantUML toolchain"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-PUML-000]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
---

The markdown body of a `pumlMode: companion` diagram shall contain an `<img>` tag whose
`src` points to the `.svg` file expected to be produced by the user's PlantUML toolchain
from the generated `.puml`. The `.svg` path is typically the `pumlFile` path with the
`.puml` extension replaced by `.svg`. This decouples the Syscribe model from the rendering
toolchain — Syscribe generates the source; the user's build pipeline generates the image.

## Example

Given a `Diagram` element at `model/Diagrams/SystemBDD.md` with `pumlMode: companion` and
no explicit `pumlFile:`, the generated source is written to `model/Diagrams/SystemBDD.puml`.
The markdown body would contain:

```markdown
<img src="SystemBDD.svg" alt="System BDD">
```

where `SystemBDD.svg` is the file the user's PlantUML toolchain is expected to produce.

## Rationale

Embedding the `<img>` reference in the body means the Syscribe web browser can serve the
rendered image (once the user's toolchain has produced it) without any changes to the
server. The model remains the authoritative source; the `.svg` is an ephemeral derived
artefact that is regenerated whenever the `.puml` changes.
