---
type: Requirement
id: REQ-TRS-PUML-033
name: "E404: pumlMode companion requires diagramKind to be set"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PUML-001]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
  - validation
---

When `pumlMode: companion` is set on a `Diagram` element, the validator shall emit
error **E404** if `diagramKind` is absent from the frontmatter.

## Contrast with svgMode

`svgMode: companion` suppresses the "missing `diagramKind`" warning because the SVG is
pre-rendered externally and Syscribe does not need to know the diagram type to serve it.
`pumlMode: companion` behaves differently: Syscribe must know the diagram type in order to
select the correct PlantUML template and generate structurally valid PlantUML source.
Omitting `diagramKind` when `pumlMode: companion` is active therefore makes the element
unprocessable and warrants a hard error.

## Error message

```
`pumlMode: companion` requires `diagramKind` to be set (e.g. `diagramKind: BDD`)
```

## Severity rationale

Without `diagramKind` the PlantUML generation cannot proceed — there is no fallback
template. Emitting an error rather than silently skipping the element prevents silent data
loss in generation pipelines where the author expects a `.puml` to be produced.
