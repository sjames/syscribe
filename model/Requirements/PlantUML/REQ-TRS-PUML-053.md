---
id: REQ-TRS-PUML-053
name: "Per-file PlantUML failures are reported without aborting the batch"
type: Requirement
status: draft
reqClass: derived
reqDomain: software
derivedFrom: REQ-TRS-PUML-050
breakdownAdr: Decisions::PlantUMLADR
tags: [diagram, plantuml, render]
---

When PlantUML exits non-zero for an individual `.puml` file, `plantuml render` prints
the error output to stderr, counts the failure, and continues processing remaining
files. After all files have been attempted, the command prints a summary line:
`<n> rendered, <m> failed` and exits non-zero if any file failed.

This ensures a single malformed diagram does not prevent valid diagrams from being
rendered.
