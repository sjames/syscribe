---
id: REQ-TRS-PUML-052
name: "--dry-run prints .puml paths without invoking PlantUML"
type: Requirement
status: draft
reqClass: derived
reqDomain: software
derivedFrom: REQ-TRS-PUML-050
breakdownAdr: Decisions::PlantUMLADR
tags: [diagram, plantuml, render]
---

When `--dry-run` is passed to `plantuml render`, the command prints the resolved
`.puml` file path for each companion that would be rendered (one per line) without
invoking PlantUML. No SVG files are written and no external processes are spawned.
