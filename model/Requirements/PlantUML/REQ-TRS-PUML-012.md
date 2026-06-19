---
type: Requirement
id: REQ-TRS-PUML-012
name: "--output flag redirects generated .puml to a specified file or stdout"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-PUML-000]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
---

When `--output <file>` (short `-o`) is supplied with `plantuml <qname>`, the generated
PlantUML source shall be written to `<file>`. When `<file>` is `-`, the output shall be
written to stdout. `--output` is only valid when a single `<qname>` is given; using it
in batch mode (no positional argument) is a usage error.

## Usage error

If `--output` is supplied without a `<qname>` positional argument, the command shall
print a usage error to stderr and exit with a non-zero status:

```
error: --output / -o requires a single <qname> argument; it cannot be used in batch mode
```

## Stdout mode

When `--output -` is used, no file is created or overwritten. This is useful for piping
the generated source directly to a PlantUML JAR or preview tool:

```
syscribe -m model/ plantuml Diagrams::SystemBDD --output - | java -jar plantuml.jar -pipe > SystemBDD.svg
```
