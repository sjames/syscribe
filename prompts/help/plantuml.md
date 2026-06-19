# plantuml — Generate PlantUML source files from Diagram elements

## Synopsis

```
syscribe -m <root> plantuml [<qname>] [--output <file>|-] [--dry-run]
syscribe -m <root> plantuml render [--jar <path>] [--dry-run]
```

## Description

Two modes of operation:

**Source generation** (default): generates PlantUML `.puml` source files from
`Diagram` model elements. The `.puml` is text-only; you run PlantUML separately
to produce SVG/PNG — or use `plantuml render` (see below).

- **Batch** (no `<qname>`): every `Diagram` with `pumlMode: companion`, written to `pumlFile:` or `<stem>.puml`.
- **Single** (`<qname>` given): one element, regardless of whether `pumlMode` is set.

**SVG rendering** (`render` subcommand): invokes PlantUML on every companion
`.puml` file in the model and writes the resulting `.svg` alongside it.
PlantUML is resolved in order: `--jar` flag → `[plantuml] jar` in
`.syscribe.toml` → `PLANTUML_JAR` env variable → `plantuml` on `PATH`.

## Supported diagram kinds

| `diagramKind` | PlantUML output |
|---|---|
| `BDD` | Class diagram — `class "Name" <<part def>>`, `*--` composition |
| `IBD` | Component diagram — `rectangle` boundary, `component` blocks, port edges resolved to parent blocks |
| `StateMachine` | State diagram — `[*]` initial, `state "Name" as id`, transition arrows |
| `Sequence` | Sequence diagram — `actor`/`participant`, `->` message, `-->` return |
| `Requirement` | Class diagram — `<<requirement>>` stereotype, `..>` derivedFrom/verifies |

`Mermaid` and unknown kinds are skipped with a warning to stderr.

## Frontmatter fields

Add `pumlMode: companion` to a `Diagram` element to opt in to batch generation.
`pumlFile:` (optional) overrides the default output path.

```yaml
---
type: Diagram
diagramKind: BDD
pumlMode: companion
pumlFile: ./diagrams/MyBDD.puml   # optional; default: <stem>.puml
---

<img src="./diagrams/MyBDD.svg" alt="MyBDD" width="100%"/>
```

The markdown body should reference the anticipated `.svg` produced by your
PlantUML toolchain. The validator emits **W413** when the `<img>` tag is
missing and **W414** when the `.puml` file has not yet been generated.

## Options — source generation

| Flag | Description |
|---|---|
| `<qname>` | Qualified name of a single `Diagram` element to generate |
| `--output <file>` | Write output to `<file>` instead of the companion path |
| `--output -` | Write output to stdout |
| `--dry-run` | Print the file path(s) that would be written without writing |

`--output` is only valid in single-element mode.

## Options — `render` subcommand

| Flag | Description |
|---|---|
| `--jar <path>` | Path to `plantuml.jar`; invoked as `java -jar <path> -tsvg` |
| `--dry-run` | Print the `.puml` paths that would be rendered without invoking PlantUML |

## Examples

```bash
# Generate all companion .puml files in the model
syscribe -m model/ plantuml

# Render all companion .puml files to SVG (plantuml on PATH)
syscribe -m model/ plantuml render

# Render using an explicit JAR
syscribe -m model/ plantuml render --jar /opt/plantuml/plantuml.jar

# Preview which .puml files would be rendered without invoking PlantUML
syscribe -m model/ plantuml render --dry-run

# Generate a single diagram, print to stdout
syscribe -m model/ plantuml Diagrams::UAVSystemBDD --output -

# Check what .puml files would be written without writing
syscribe -m model/ plantuml --dry-run

# Generate to an explicit path
syscribe -m model/ plantuml Diagrams::PowerSystemIBD --output /tmp/PowerSystem.puml
```

## Validation diagnostics

| Code | Severity | Trigger |
|---|---|---|
| E403 | error | `pumlMode` has an unrecognized value (only `companion` is supported) |
| E404 | error | `pumlMode: companion` is set but `diagramKind` is absent |
| W413 | warning | `pumlMode: companion` but body has no `<img` tag |
| W414 | warning | `pumlMode: companion` but the `.puml` file does not exist on disk |
