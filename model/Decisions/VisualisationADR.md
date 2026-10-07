---
type: ADR
id: ADR-SYS-VIS-001
name: "Visualisation: one Diagram IR in syscribe-model, ELK layout in the browser via sprotty-elk, PlantUML/Mermaid/SVG as IR backends, legacy renderers removed"
status: accepted
tags:
  - diagram
  - visualisation
  - sprotty
  - elk
---

## Context

A design review on 2026-10-07 (`docs/design/visualisation.md` §1) found five diagram
rendering paths that share no data model, no layout engine and no styling: the retired
server SVG renderer (`syscribe-model::renderer`), the CLI `diagram` toolkit (taffy +
Cassowary + A\*, ~6 k lines, requirement/sequence SVG only), the Mermaid pass-through, the
PlantUML companion generator with its own private manifest parser, and the sprotty editor
(`ADR-SYS-DE-001`), which has no layout engine at all. Every diagram is a hand-listed
`shapes:`/`edges:` manifest; nothing is derived from the model. The shared manifest parser
cannot read the spec's own string shorthand and ignores `parent:`, so a diagram written
exactly as §8.16.3 permits renders empty, silently, and an IBD's ports are flattened into
siblings. Sixteen of the seventeen demo diagrams have no `layout:` and open as a pile at
the origin. Test coverage across the ~5 k lines of rendering code is a handful of unit
tests on parse helpers.

The user's goal is a top-quality visualisation implementation. Four questions were put to
the user and answered the same day; a fifth constraint followed:

1. Where does automatic layout come from — a pure-Rust layered engine, ELK or Graphviz?
2. How much editing does the browser support?
3. Do PlantUML and Mermaid survive?
4. Which SysMLv2 view kinds come first?
5. Is backwards compatibility owed to any existing rendering output or command?

## Decision

1. **One Diagram IR in `syscribe-model` (`vis::ir`).** A plain, serialisable
   `DiagramGraph` of nodes (flat list, nesting by `parent`, closed `NodeKind` enum,
   optional pin geometry) and edges (closed `EdgeKind` enum, optional waypoints). Every
   consumer — the sprotty endpoint, the PlantUML writer, the Mermaid writer, the SVG
   writer, `export-html` — reads the IR and nothing else. One manifest parser
   (`vis::manifest`) builds it from `shapes:`/`edges:`/`layout:`, honouring the whole of
   spec §8.16.3–8.16.4.
2. **Derived content, selected by frontmatter, no mode field.** A `Diagram` that declares
   `shapes:` is manifest-sourced; one that declares only `subject:` is derived by the
   generator for its `diagramKind` (`vis::derive::bdd`, `vis::derive::ibd` first), with
   optional `include:`/`exclude:` member filters and deterministic shape ids so pins
   survive regeneration.
3. **Layout: ELK, in the browser, through `sprotty-elk` and `elkjs`.** sprotty remains
   the renderer and the editing surface. `elkjs` and `sprotty-elk` are vendored into the
   existing esbuild bundle (no CDN, no runtime Node). The server never computes
   positions; it ships per-kind ELK layout options and the set of pinned nodes with the
   graph. Pins (`layout:` entries) are honoured through ELK's interactive layering and
   crossing-minimisation; a fully pinned diagram uses ELK's `fixed` algorithm. An
   auto-layout result is never written to the model implicitly — only a drag, *Pin all*
   or *Auto-layout* (which clears pins over a new `DELETE /api/diagrams/layout/{qname}`)
   changes a file.
4. **Full editing, as `REQ-TRS-DE-004` is written**, on both manifest and derived
   diagrams: create, delete, connect, move, all through the shared guarded-write engine.
   The connect gesture becomes port-aware and writes a `connections:` entry on the owning
   part; on a derived diagram structural edits touch only the model and the view
   regenerates.
5. **PlantUML and Mermaid stay as export backends of the IR**, never as layout sources.
   The PlantUML writer's private parser is replaced by the IR; a Mermaid writer is added
   (`classDiagram` for BDD, `flowchart` with `subgraph` nesting for IBD) emitting
   `%% ref:` annotations so the existing lints apply. Both are reachable through
   `syscribe diagram export --format plantuml|mermaid|svg` and MCP `render_diagram`.
6. **Static SVG follows "the browser is the layout authority".** `vis::svg` draws an IR
   only when every node has a pin; otherwise `export-html` uses a companion SVG the
   browser saved (*Save companion SVG*, written through the guarded-write engine), then a
   PlantUML-rendered companion, then a placeholder linking to the browser.
7. **Structure first: BDD and IBD.** State, Action, Requirement, Sequence and Allocation
   generators follow on the same IR (`REQ-TRS-VIS-015`).
8. **Legacy rendering paths are removed in Phase 0, with no parity gate.** The user ruled
   out backwards compatibility for rendering. `renderer.rs`, `diagram.rs`'s parse
   helpers, the PlantUML private parser and the entire CLI `diagram` toolkit (with its
   docs, `REQ-TRS-DIAG-004` and `TC-TRS-DIAG-004`) are deleted up front.
   `syscribe diagram export` is the one diagram subcommand afterwards. Hand-authored
   `diagramKind: Mermaid` and inline `diagramKind: PlantUML` diagrams are untouched.
9. **New validation codes** `E405` (malformed `shapes:`/`edges:`/`layout:`, replacing the
   silent empty fallback), `W416` (stale pin), `W417` (`include:`/`exclude:` misuse),
   `W418` (subject type invalid for the diagram kind). `E400`–`E404`, `W400`–`W415` are
   unchanged.

## Rationale

**One IR over five parsers.** Every defect the review found is a consequence of the same
manifest being re-read by different code with different gaps. A single parser with a
closed vocabulary makes "is this diagram well-formed" one question with one answer, and
makes every backend a pure function of the same value, which is what makes snapshot
testing possible at all.

**ELK in the browser over a Rust engine or server-side ELK.** A port-aware,
hierarchy-aware layered layout with orthogonal routing is the hardest part of any
diagramming stack; ELK already is one, is the engine sprotty was designed around, and is
MIT/EPL-licensed. Writing a Rust equivalent would cost months before reaching ELK's
quality on IBDs. Running ELK on the server would force a Node runtime onto `syscribe`
and `syscribe-server`, contradicting `ADR-SYS-DE-001`'s "dev-time only" JS posture. The
price of the chosen option is real and accepted: the CLI cannot auto-lay-out, so static
artefacts must come from pins or from a picture the browser saved. That is also the
honest workflow for engineering diagrams — a human looks at the layout before it goes
into a document.

**Derived by default, manifest by declaration.** The whole point of holding a model is
that views follow the model. Inferring the source from the presence of `shapes:` keeps
every existing diagram valid as written and lets a new diagram be two lines of
frontmatter. A separate mode field would be a third thing to keep consistent.

**Pins, not auto-written layouts.** Writing ELK's output back on every open would make
`git diff` on diagram files meaningless and would overwrite a human's placement with a
machine's. Pins are explicit, reviewable intent.

**Removal over coexistence.** With no compatibility obligation, keeping the legacy
renderers alive until "parity" would only enlarge the surface to migrate and invite one
more divergent visual language. Deleting them first makes the IR the only path from day
one.

## Consequences

- `syscribe-model` gains a `vis` module and loses `renderer.rs` and `diagram.rs`;
  `crates/syscribe/src/diagram/*` is deleted and replaced by one `diagram export`
  subcommand. `docs/cli/index.md`, `prompts/help/diagram.md`, `REQ-TRS-DIAG-004`,
  `TC-TRS-DIAG-004` and the corresponding qual script are retired in the same change.
- The frontend bundle grows by roughly 1.4 MB (ELK), on a page that already serves a
  3.3 MB Mermaid bundle. ELK runs on the main thread; a Web Worker is available through
  `sprotty-elk` if a real model proves slow.
- `export-html` and MkDocs can show a diagram only if it is fully pinned or has a
  companion SVG/PlantUML render; otherwise they show a placeholder. This is a visible
  regression for the demo model's unpinned diagrams until the browser is used once to
  save them, and is accepted.
- Spec §8.16 is extended with the source-selection rule, `include:`/`exclude:`, the pin
  semantics of `layout:`, and the four new codes; `prompts/spec/validation.md` and
  `docs/validation/rules.md` gain the codes in the same commit as the validator.
- `ADR-SYS-DE-001` stays in force for the editing architecture (guarded-write engine,
  sprotty standalone, transactional diagram sync); this decision changes what the editor
  renders and how it is laid out, not how it writes. `ADR-SYS-PUML-001`'s
  `pumlMode: companion` workflow is unchanged; only the generator's input becomes the IR.
- Behaviour and traceability views are deliberately later (`REQ-TRS-VIS-015`); until
  then State/Sequence/Requirement diagrams are reachable only as manifest diagrams and
  through the PlantUML backend.
