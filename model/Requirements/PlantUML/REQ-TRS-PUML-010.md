---
type: Requirement
id: REQ-TRS-PUML-010
name: "plantuml subcommand generates all companion .puml files in batch"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-PUML-000]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
---

`syscribe -m <root> plantuml` (with no positional argument) shall locate every `Diagram`
element in the model that has `pumlMode: companion`, generate the PlantUML source for
each, and write the output to the resolved `pumlFile` path (REQ-TRS-PUML-001). Elements
whose `diagramKind` is not supported (REQ-TRS-PUML-025) shall be skipped with a warning
to stderr. The command shall report the number of files written on completion.

## Completion message

On success the command shall print a summary line to stdout, for example:

```
plantuml: wrote 4 file(s).
```

When no eligible diagrams are found (no `Diagram` elements carry `pumlMode: companion`),
the command shall print a message indicating that no files were written and exit with
code `0`.

## Exit codes

| Condition | Exit code |
|---|---|
| All eligible diagrams generated successfully | `0` |
| One or more write errors | non-zero |
| Unsupported `diagramKind` (skip only) | `0` |
