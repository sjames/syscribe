---
type: Requirement
id: REQ-TRS-PUML-030
name: "W413: pumlMode companion body must contain an img tag referencing the anticipated SVG"
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
warning **W413** if the element's markdown body contains no `<img` tag.

## Rationale

The body is the display surface for the rendered diagram. Without an `<img>` tag pointing
to the anticipated SVG output (produced by the user's PlantUML toolchain from the
generated `.puml` file), the diagram is never visible in the web UI or rendered
documentation. The absence of an `<img>` tag almost always indicates that the author has
not yet wired up the display step.

## Warning message

```
`pumlMode: companion` but body contains no `<img` tag pointing to the anticipated SVG output
```

## Severity rationale

This is a warning rather than an error because the model is not semantically ill-formed;
the companion `.puml` may still be generated and rendered by external tooling. The warning
nudges the author to close the display gap without blocking the build.
