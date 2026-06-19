---
id: REQ-TRS-PUML-040
name: "[plantuml] config section in .syscribe.toml"
type: Requirement
status: draft
reqClass: system
reqDomain: software
derivedFrom: REQ-TRS-PUML-000
breakdownAdr: Decisions::PlantUMLADR
tags: [diagram, plantuml, config]
---

The `[plantuml]` section in `.syscribe.toml` accepts two optional keys:

- `theme` — a PlantUML built-in theme name (string, e.g. `"spacelab"`).
- `style_file` — a path to a `.puml` snippet file, resolved relative to the model root.

Both keys are optional. When neither is set the generator falls back to built-in `skinparam` blocks. When both are set `style_file` takes precedence and `theme` is ignored.
