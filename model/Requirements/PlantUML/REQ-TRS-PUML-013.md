---
type: Requirement
id: REQ-TRS-PUML-013
name: "--dry-run prints planned output paths without writing files"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-PUML-000]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
---

When `--dry-run` is supplied, the `plantuml` command shall print the file path(s) that
would be written to stdout (one per line) without writing any files. In single-element
mode with `--output`, the resolved output path is printed. Useful for CI verification
that all companion paths are where the PlantUML toolchain expects them.

## Output format

Each line shall be the absolute or model-root-relative path of the `.puml` file that
would be written, for example:

```
model/Diagrams/SystemBDD.puml
model/Diagrams/PowertrainIBD.puml
model/Diagrams/SchedulerSM.puml
```

## Interaction with `--output -`

When `--dry-run` and `--output -` are both supplied in single-element mode, the path
printed shall be `<stdout>` to make clear that no file would have been created.

## CI use case

A CI step can run `plantuml --dry-run` to assert that the expected `.puml` files exist
on disk (having been previously generated and committed), without generating or
overwriting them:

```bash
syscribe -m model/ plantuml --dry-run | xargs -I{} test -f {}
```
