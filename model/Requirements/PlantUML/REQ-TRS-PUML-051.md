---
id: REQ-TRS-PUML-051
name: "PlantUML tool resolution order for plantuml render"
type: Requirement
status: draft
reqClass: derived
reqDomain: software
derivedFrom: REQ-TRS-PUML-050
breakdownAdr: Decisions::PlantUMLADR
tags: [diagram, plantuml, render]
---

The `plantuml render` subcommand resolves the PlantUML executable using the
following precedence (first match wins):

1. `--jar <path>` CLI flag — path to a PlantUML `.jar`; invoked as `java -jar <path> -tsvg`
2. `[plantuml] jar = "<path>"` in `.syscribe.toml` — same invocation as above
3. `PLANTUML_JAR` environment variable — same invocation as above
4. `plantuml` on `PATH` — invoked directly as `plantuml -tsvg` (covers system packages
   and wrapper scripts)

If none of the above resolves to a working invocation the command prints a clear
error message explaining the resolution order and exits non-zero.
