---
type: Requirement
id: REQ-TRS-PUML-000
name: "PlantUML diagram source export from Diagram elements"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - diagram
  - plantuml
---

Syscribe shall provide a mechanism to export `Diagram` elements as PlantUML (`.puml`)
source files, enabling users to produce publication-quality diagrams via their own
PlantUML toolchain without manual transcription of the model structure.

## Rationale

Teams using PlantUML as their diagramming standard have an established pipeline —
local JAR execution, CI actions, Gradle/Maven plugins — that produces consistently
styled output across a project's documentation. Requiring engineers to manually author
`.puml` files from a Syscribe model defeats the single-source-of-truth principle: any
structural change in the model must be propagated by hand to every diagram, introducing
drift between the model and the published documentation.

By generating `.puml` source directly from `Diagram` elements, Syscribe closes the loop:
the model is the authoritative description of the system; the PlantUML source and the
rendered image are derived artefacts produced automatically by the toolchain.
