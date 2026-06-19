---
type: Requirement
id: REQ-TRS-PUML-001
name: "pumlMode and pumlFile frontmatter fields on Diagram elements"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-PUML-000]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
---

A `Diagram` element shall support a `pumlMode: companion` field to opt in to PlantUML
generation. `pumlFile:` (optional) shall specify the output path relative to the `.md`
file; when absent, it shall default to `<stem>.puml` in the same directory. These fields
shall be independent of and coexist with the existing `svgMode:`/`svgFile:` fields.

## Schema

```yaml
pumlMode: companion        # opt-in; currently the only recognised value
pumlFile: diagrams/foo.puml  # optional; defaults to <stem>.puml
```

## Interaction with svgMode

A `Diagram` element may carry both `svgMode: companion` (for Syscribe's native SVG
renderer) and `pumlMode: companion` (for PlantUML source generation) simultaneously.
The two pipelines operate independently: setting one does not imply or require the other.
