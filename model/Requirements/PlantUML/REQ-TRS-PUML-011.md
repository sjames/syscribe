---
type: Requirement
id: REQ-TRS-PUML-011
name: "plantuml <qname> generates .puml for a single named Diagram element"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-PUML-000]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
---

`syscribe -m <root> plantuml <qname>` shall generate the PlantUML source for the single
named `Diagram` element, regardless of whether `pumlMode` is set on that element. The
output path is resolved from `pumlFile:` when present, otherwise defaults to `<stem>.puml`
next to the element's `.md` file, unless overridden by `--output`.

## Behaviour

- If `<qname>` does not resolve to a known element, the command shall exit with a non-zero
  status and print a human-readable error to stderr.
- If `<qname>` resolves to an element that is not a `Diagram`, the command shall exit with
  a non-zero status and an appropriate error message.
- If the `diagramKind` of the named element is not supported (REQ-TRS-PUML-025), the
  command shall exit with a non-zero status rather than silently skipping, since the user
  explicitly requested this element.

## Interaction with `--output`

When `--output` is supplied alongside `<qname>`, it overrides the resolved output path.
See REQ-TRS-PUML-012 for the full `--output` specification.
