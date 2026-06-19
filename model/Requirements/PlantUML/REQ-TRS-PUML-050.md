---
id: REQ-TRS-PUML-050
name: "plantuml render subcommand invokes PlantUML on all companion .puml files"
type: Requirement
status: draft
reqClass: system
reqDomain: software
derivedFrom: REQ-TRS-PUML-010
breakdownAdr: Decisions::PlantUMLADR
tags: [diagram, plantuml, render]
---

The command `syscribe -m <root> plantuml render` finds every Diagram element in the
model that has `pumlMode: companion` set, resolves the companion `.puml` path for
each (via `pumlFile:` or the default `<stem>.puml` convention), and invokes the
PlantUML tool once per file to produce an SVG output alongside the `.puml` file.

The SVG file is written to the same directory as the `.puml` file with the same stem
and a `.svg` extension (the default PlantUML `-tsvg` output behaviour).

Diagrams without `pumlMode: companion` are not processed. Diagrams whose `.puml`
companion file does not exist on disk are skipped with a warning to stderr.
