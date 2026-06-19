---
id: REQ-TRS-PUML-042
name: "style_file key emits !include directive"
type: Requirement
status: draft
reqClass: derived
reqDomain: software
derivedFrom: REQ-TRS-PUML-040
breakdownAdr: Decisions::PlantUMLADR
tags: [diagram, plantuml, config]
---

When `[plantuml] style_file = "<path>"` is set in `.syscribe.toml`, every generated `.puml` file begins with:

```
!include <absolute-path>
```

where `<absolute-path>` is the `style_file` value resolved to an absolute filesystem path relative to the model root. Using an absolute path avoids PlantUML include-resolution issues when `.puml` files reside in subdirectories.

`style_file` takes precedence over `theme`; if both are set, `theme` is silently ignored.

If the resolved `style_file` path does not exist on disk, the validator emits warning **W415**.
