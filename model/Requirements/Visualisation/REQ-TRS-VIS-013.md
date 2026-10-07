---
type: Requirement
id: REQ-TRS-VIS-013
name: "The legacy renderer, manifest helpers and CLI diagram toolkit are removed outright in the first phase"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
---

The following shall be deleted, not deprecated or wrapped, in the first implementation phase:

- `crates/syscribe-model/src/renderer.rs` (`render_diagram`) and `export-html`'s use of it;
- `crates/syscribe-model/src/diagram.rs`'s `DiagramShape`/`DiagramEdge`/`ShapeLayout`,
  `parse_*` helpers and `default_size`;
- `plantuml.rs`'s private `parse_shapes`/`parse_edges`;
- `crates/syscribe/src/diagram/*` — the `list`, `render`, `measure`, `compose`, `layout`,
  `seq` and `req` subcommands, the taffy/Cassowary/A\* layout engine, and their fixed-path
  `/tmp` scratch files — together with `prompts/help/diagram.md`'s description of them, the
  `docs/cli/index.md` section, qualification requirement `REQ-TRS-DIAG-004` and test case
  `TC-TRS-DIAG-004` with its script.

Hand-authored `diagramKind: Mermaid` and inline `diagramKind: PlantUML` diagrams, the
`render <path>` command, `syscribe plantuml`, and every `Diagram` validation rule in
`E400`–`E404`/`W400`–`W415` shall be unaffected.

## Rationale

The user ruled out backwards compatibility for rendering on 2026-10-07. Keeping five paths
alive until an IR reaches parity would only enlarge the migration surface and invite one more
divergent visual language; deleting them first makes the IR the only path from day one.

## Scope

- `docs/cli/index.md`, `docs/browser/index.md`, `CLAUDE.md`'s diagram notes and the release
  notes record the removal in the same commit.
- `syscribe diagram export` (`REQ-TRS-VIS-009`) becomes the only `diagram` subcommand when it
  lands; between the removal and that landing the `diagram` command is absent.
