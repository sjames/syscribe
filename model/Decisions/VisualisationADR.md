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

## Addendum: the same ELK engine inside the executable (REQ-TRS-VIS-016, REQ-TRS-VIS-017)

**Context.** Decision 3 put layout in the browser and accepted that the CLI could only draw a
diagram that was fully pinned or had a saved companion SVG. On 2026-10-07 the user asked
whether the engine behind sprotty's layout quality could also produce SVG from the executable.
Sprotty itself cannot run outside a DOM and is not needed outside it — the Rust SVG writer
(`vis::svg`, `REQ-TRS-VIS-010`) plays its drawing role against the same `vis::style` — but the
engine behind it, ELK, is plain JavaScript. A spike embedded QuickJS (the `rquickjs` crate) in a
Rust binary, loaded the identical `elk.bundled.js` the browser bundle uses, and laid out an
IBD-shaped graph with nested ports and routed edges in roughly 0.4 s cold, including loading the
1.5 MB bundle; no Node, no browser, two global shims (`window`/`global`, a `setTimeout` queue).

**Decision.**
1. `syscribe-model` gains `vis::layout`: the vendored `elk.bundled.js` (EPL-2.0, committed under
   `crates/syscribe-model/vendor/elkjs/` with its licence) embedded with `include_str!` and run
   in-process under QuickJS. It takes an IR plus node sizes and returns absolute positions, port
   placements and edge routes. It uses the same ELK options the client's configurator sends
   (direction, hierarchy, port constraints, interactive layering for pins, `fixed` when every
   node is pinned), so the two renderers are one engine with one configuration.
2. Node sizing moves to Rust and becomes authoritative (`REQ-TRS-VIS-017`): the text metrics
   `svgkit` already holds for MagicGrid are promoted into `syscribe-model` (`vis::metrics`),
   every node, port and label in the sprotty graph is sent with a `size`, and the client uses
   those sizes instead of measuring in the DOM. Because ELK is deterministic, identical input
   sizes and options yield identical coordinates in the browser and in the CLI.
3. `vis::svg` accepts any diagram: a fully pinned one is drawn as before; otherwise it calls
   `vis::layout` first. `diagram export --format svg`, `export-html` and MkDocs therefore never
   need pins or a browser visit; `Save companion SVG` remains the way to freeze a hand-adjusted
   layout. Decision 6's placeholder chain is reduced to: draw; fall back to a companion only when
   the IR is empty.

**Rationale.** The user chose ELK for its quality; running the same engine on both sides removes
the one real cost of decision 3 without reopening the server-side-Node option the ADR rejected:
QuickJS is a small MIT-licensed C library compiled into the binary, not a runtime dependency.
Rust-side sizing is what makes "the same picture" literally true rather than approximately true.

**Consequences.** About 2.5 MB added to the binaries (ELK bundle plus QuickJS); a C compiler is
required at build time (QuickJS is built by `cc`); ELK under QuickJS is slower than under V8 —
acceptable for the diagram sizes a model holds, and bounded by `REQ-TRS-VIS-016`. The font family
used for metrics and the CSS font stack of the client must stay the same, and the metrics carry a
safety margin so a slightly wider browser font never clips a label. `REQ-TRS-VIS-010`'s pinned-only
rule and `ADR` decision 6's fallback chain are superseded as described above.
