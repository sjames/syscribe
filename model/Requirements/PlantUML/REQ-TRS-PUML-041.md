---
id: REQ-TRS-PUML-041
name: "theme key emits !theme directive"
type: Requirement
status: draft
reqClass: derived
reqDomain: software
derivedFrom: REQ-TRS-PUML-040
breakdownAdr: Decisions::PlantUMLADR
tags: [diagram, plantuml, config]
---

When `[plantuml] theme = "<name>"` is set in `.syscribe.toml` and no `style_file` is present, every generated `.puml` file begins with:

```
!theme <name>
```

in place of the built-in `skinparam` block. The theme name is passed through verbatim; no validation is performed (PlantUML rejects unknown themes at render time).
