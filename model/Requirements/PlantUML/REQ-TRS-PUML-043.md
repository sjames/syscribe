---
id: REQ-TRS-PUML-043
name: "base_url config key for element detail links"
type: Requirement
status: draft
reqClass: system
reqDomain: software
derivedFrom: REQ-TRS-PUML-040
breakdownAdr: Decisions::PlantUMLADR
tags: [diagram, plantuml, config]
---

The `[plantuml]` section in `.syscribe.toml` accepts an optional `base_url` key (string).

When set, every shape that has a `ref` in the generated `.puml` receives a PlantUML URL
annotation `[[<base_url>/ui/detail/<ref>]]` appended to its declaration line, making it
a clickable hyperlink in PlantUML-rendered output (SVG, HTML image maps).

When unset the default is `http://localhost:3000`, matching the default Syscribe web
server port, so links work out of the box during local development.

To suppress links entirely, set `base_url = ""`.
