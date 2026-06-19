---
type: Package
name: PlantUML
---

Requirements for the PlantUML companion feature: the `pumlMode:` and `pumlFile:` schema
fields on `Diagram` elements, the `plantuml` CLI subcommand (batch and single-element
modes), output redirection, dry-run support, and the per-`diagramKind` PlantUML mapping
rules.

All requirements derive from `REQ-TRS-PUML-000` and are governed by `ADR-SYS-PUML-001`.
