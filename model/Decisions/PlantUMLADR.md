---
type: ADR
id: ADR-SYS-PUML-001
name: "Mirror svgMode:companion pattern for PlantUML with pumlMode:companion; text-only generation"
status: accepted
tags:
  - diagram
  - plantuml
---

## Context

The Syscribe web server produces SVG diagrams server-side from `Diagram` elements via its
own rendering pipeline. This works well for interactive browsing but leaves teams that
prefer publication-quality output from the PlantUML ecosystem — with its mature skinparam
system, themes, and PDF/PNG export — without a first-class workflow.

Two patterns were evaluated for how Syscribe could accommodate PlantUML:

1. **Render via PlantUML server** — Syscribe calls the PlantUML HTTP rendering API (or
   forks the PlantUML JAR) and writes the resulting `.svg` or `.png` directly, similar to
   how the existing SVG renderer writes SVG files.
2. **Text-only source generation** — Syscribe generates the `.puml` text file only; the
   user runs their own PlantUML toolchain (local JAR, Docker image, CI action) to produce
   the final image. The markdown body of the `Diagram` element references the anticipated
   `.svg` output via an `<img>` tag.

The existing `svgMode: companion` / `svgFile:` frontmatter fields provide the closest
analogue: they cause Syscribe to write an SVG companion file alongside the `.md` file, and
the body can embed the result via `<img src="...">`.

## Decision

Option 2: text-only `.puml` source generation, mirroring the `svgMode: companion` pattern.

Two new optional frontmatter fields are added to `Diagram` elements:

- **`pumlMode: companion`** — opt-in flag. When present and set to `companion`, the
  `plantuml` CLI subcommand includes this diagram in batch generation.
- **`pumlFile:`** — optional output path relative to the `.md` file. Defaults to
  `<stem>.puml` in the same directory when absent.

These fields are **independent of and coexist with** the existing `svgMode:`/`svgFile:`
fields; a diagram may carry both if the author wants both Syscribe's native SVG and a
PlantUML source alongside it.

The markdown body of a `pumlMode: companion` diagram contains an `<img>` tag whose `src`
points to the `.svg` file expected to be produced from the generated `.puml` by the user's
toolchain. Typically this is the `pumlFile` path with `.puml` replaced by `.svg`.

## Rationale

**Against Option 1 (render via PlantUML server):**

- Integrating the PlantUML HTTP API introduces an external network dependency. Offline
  development environments, air-gapped CI, and restricted corporate networks would all
  break silently or require additional configuration to point to an internal mirror.
- Bundling or forking the PlantUML JAR adds a Java runtime dependency, significantly
  increasing binary size and deployment complexity for what is a Rust-native tool.
- PlantUML's rendering behaviour and SVG output format change across JAR versions,
  creating a hidden coupling between Syscribe releases and the PlantUML release train.

**For Option 2 (text-only generation):**

- Keeps the Syscribe binary self-contained and dependency-free. The `plantuml` subcommand
  is pure Rust string generation.
- Teams already have PlantUML integrated into their CI pipelines (GitHub Actions, GitLab
  CI, Maven/Gradle plugins). Syscribe slots in as a source-generation step that feeds that
  existing pipeline, rather than replacing it.
- Mirrors the mental model engineers already have with the `svgMode: companion` pattern,
  reducing the learning surface.
- The `.puml` file is a human-readable, diffable text artefact — easier to review in
  pull requests than a binary or regenerated SVG.

## Consequences

- The `plantuml` subcommand is a pure text-generation pass: no network calls, no external
  process invocations, no binary output. It can be fully tested with string comparison.
- The user's build pipeline is responsible for running PlantUML on the generated `.puml`
  files. Syscribe provides a `--dry-run` mode to verify output paths without writing files.
- `diagramKind` values that have no PlantUML mapping (`Mermaid`, unknown kinds) are skipped
  with a stderr warning; they do not cause a non-zero exit code unless a write error occurs.
- Supported `diagramKind` values: `BDD`, `IBD`, `StateMachine`, `Sequence`, `Requirement`.
