---
type: Requirement
id: REQ-TRS-PUML-025
name: "Unsupported diagramKind values are skipped with a stderr warning"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PUML-001]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
---

For `Diagram` elements with a `diagramKind` value not covered by REQ-TRS-PUML-020
through REQ-TRS-PUML-024 (currently: `Mermaid` and any unknown kind), the generator
shall skip the element and print a warning to stderr of the form:

```
warn: skipping '<qname>' — diagramKind '<kind>' has no PlantUML mapping
```

The batch command (REQ-TRS-PUML-010) shall continue processing remaining diagrams after
a skip; the exit code shall remain `0` unless a write error occurs.

## Currently unsupported kinds

| `diagramKind` | Reason |
|---|---|
| `Mermaid` | Mermaid syntax is already embedded in the body; PlantUML generation would duplicate it in a different format with no clear benefit. |
| Any unrecognised value | Future diagram kinds are unknown at the time of this writing; they default to unsupported until a mapping is specified. |

## Distinction from single-element mode

In single-element mode (`plantuml <qname>`), if the named element has an unsupported
`diagramKind`, the command shall exit with a non-zero status rather than silently
skipping (see REQ-TRS-PUML-011). The skip-and-warn behaviour applies only to batch mode.
