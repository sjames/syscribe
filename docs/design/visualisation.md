# Design: Visualisation — one Diagram IR, ELK layout in the browser

**Status:** Proposed — decisions ratified in `ADR-SYS-VIS-001`, requirements `REQ-TRS-VIS-000..015`
**Date:** 2026-10-07

> **Implementation note.** Phase 0 (§9) landed on 2026-10-07: `syscribe_model::vis::{ir,
> manifest, sprotty}`, `E405`/`W416`, the PlantUML writer on the IR, the nested
> `/api/diagrams/model` contract with a client that renders it, and the removals of §11
> (`PI-VIS-001`). The §1 table now describes history. Phases 1–4 are open.
> This document records the design the four user decisions of 2026-10-07 fixed (§3) and
> is the reference the `REQ-TRS-VIS-*` requirements and `PI-VIS-*` planning items point
> at. Sections are updated in place as phases ship.
>
> **Compatibility.** The same day the user ruled that no backwards compatibility is
> owed to any existing rendering path: the legacy server renderer, the CLI `diagram`
> toolkit and the private manifest parsers are removed outright in Phase 0 rather than
> kept until parity (§9, §11). Model *data* (`Diagram` frontmatter, companion SVG and
> PlantUML files) is unaffected except where §8 says otherwise.

Syscribe's diagrams should be what the rest of the tool already is: derived from the
model, validated like the model, and rendered the same way everywhere. Today they are
not. This note explains why, fixes the architecture, and lays out the phases that get
from here to there.

---

## 1. Motivation

A systems model that cannot be *seen* is only half a model. SysMLv2 defines its views
(block definition, internal block, state, action, requirement, sequence) precisely so
that the same underlying elements can be looked at from several angles. Syscribe has
the elements. What it lacks is a single, dependable way to turn them into pictures.

The 2026-10-07 review found **five rendering paths that share no data model, no layout
and no styling**:

| Path | Where | What it does today |
|---|---|---|
| Server SVG renderer | `crates/syscribe-model/src/renderer.rs` | Builds SVG from `shapes:`/`edges:`/`layout:`. Retired from the HTTP route; still used by `export-html`. Draws nothing when `layout:` is absent. |
| CLI compose engine | `crates/syscribe/src/diagram/{compose,solver,astar,layout/*}.rs` (~4 k lines) | taffy + Cassowary + A\* layout for requirement/sequence SVG. Writes scratch files to `/tmp` by fixed name. |
| Mermaid pass-through | `routes/ui.rs` | Renders a hand-written ` ```mermaid ` block client-side. |
| PlantUML companion | `crates/syscribe-model/src/plantuml.rs` | Generates `.puml` text from the manifest; its own private manifest parser. |
| sprotty editor | `crates/syscribe-server/frontend/`, `routes/diagram_model.rs` | Editable block diagram. No layout engine: `needsClientLayout: false`, `needsServerLayout: false`. |

Concrete consequences, each verified in the code or the demo model:

- **Nothing lays anything out.** The sprotty endpoint emits every node at `(0, 0)` unless
  the diagram has a `layout:` block. In the demo model only `SafetyRequirementsD` has one;
  the other sixteen diagrams open as a pile of overlapping boxes.
- **The manifest schema is only half honoured.** `DiagramShape` requires `kind:` and
  cannot deserialise the spec's string shorthand (`engine-rect: VehicleSystem::Engine`),
  so a diagram written as §8.16.3 permits renders *empty*, silently. `parent:` is never read,
  so an IBD's ports and nested parts are flattened into siblings. A malformed
  `shapes:`/`edges:`/`layout:` falls back to an empty map with no finding.
- **Diagrams are hand-listed, not derived.** Every shape and edge is typed by a human
  (or an LLM) into frontmatter. Add a part to the model and no diagram changes.
- **Three manifest parsers** (`diagram.rs`, `plantuml.rs`, `renderer.rs`) and two visual
  languages (`renderer.rs` colours mirrored by hand into `views.tsx`).
- **Almost no tests**: a handful of unit tests on parse helpers; nothing on the
  sprotty endpoint's graph shape, nothing on layout, nothing on the client.

---

## 2. Goals and non-goals

**Goals**

1. One data model for every diagram, built once from the model, consumed by every
   renderer and exporter.
2. Diagrams that lay themselves out well without a human placing boxes, and that keep a
   human's adjustments when one does.
3. The browser remains a true authoring surface (`REQ-TRS-DE-004`): create, delete,
   connect, move.
4. Text exports (PlantUML, Mermaid) and a static SVG for docs, all from the same IR.
5. Correct SysMLv2 notation for the first two view kinds, BDD and IBD, with the rest
   following on the same rails.
6. Real test coverage at every layer.

**Non-goals**

- A general-purpose graph layout library in Rust (decided against, §3).
- Backwards compatibility with today's rendered output, the `diagram` CLI toolkit's
  subcommands or their SVG (user decision, 2026-10-07).
- Replacing hand-authored Mermaid or inline PlantUML diagrams; they keep working unchanged.
- GLSP or any diagram-server protocol (`ADR-SYS-DE-001` already rejected it).
- Pixel-identical output between the browser and the text exporters; PlantUML and
  Mermaid lay themselves out and are not expected to match ELK.

---

## 3. Decisions fixed on 2026-10-07

| # | Question | Decision |
|---|---|---|
| 1 | Where does automatic layout come from? | **ELK, in the browser, via `sprotty-elk`/`elkjs`.** sprotty stays the renderer. No pure-Rust engine, no server-side Node. |
| 2 | How much editing does the browser support? | **Full editing as `REQ-TRS-DE-004` is written** — create, delete, connect, move. |
| 3 | What happens to PlantUML and Mermaid? | **Both stay, as export backends generated from the IR.** |
| 4 | Which view kinds first? | **Structure first: BDD and IBD.** Behaviour and traceability views follow. |

The first decision shapes everything else. Because layout happens in the browser, the
**browser is the layout authority**: the CLI and the server never compute positions.
What they *can* do is hold positions a browser produced, as pins in `layout:` or as a
saved companion SVG. §7 spells out the static-export policy that follows.

---

## 4. Architecture

```
                 model (RawElement graph + Resolver)
                              │
          ┌───────────────────┴────────────────────┐
          │ syscribe-model::vis                     │
          │                                         │
          │   ir.rs        DiagramGraph / Node / Edge (§5)
          │   manifest.rs  shapes:/edges:/layout: → IR  (diagram declares shapes:)
          │   derive/      bdd.rs, ibd.rs → IR          (diagram declares only subject:)
          │   style.rs     one visual language (stereotypes, kinds, colours)
          │   sprotty.rs   IR → nested SGraph JSON + ELK options + pins
          │   plantuml.rs  IR → .puml          (replaces today's private parser)
          │   mermaid.rs   IR → mermaid text   (new)
          │   svg.rs       IR with full geometry → SVG per §8.16.5 (replaces renderer.rs)
          └───────────────────┬────────────────────┘
                              │
   ┌──────────────┬───────────┴────────────┬───────────────────┐
   │ syscribe-server          │ syscribe CLI / MCP       │ export-html       │
   │ GET /api/diagrams/model  │ plantuml, diagram export │ svg.rs or         │
   │ PATCH /api/diagrams/layout│ render_diagram           │ companion fallback│
   └──────┬───────┘
          │ JSON (sprotty SGraph, nested)
   ┌──────┴───────────────────────────────────────┐
   │ frontend (sprotty 1.4 + sprotty-elk + elkjs)  │
   │  measure labels → ELK layered layout → render │
   │  pins honoured; edits via guarded-write REST  │
   └──────────────────────────────────────────────┘
```

Everything above the JSON boundary is Rust and testable with `cargo test`. Everything
below it is the existing esbuild bundle plus two vendored libraries.

---

## 5. The Diagram IR

`syscribe_model::vis::ir` is a plain, serialisable value type. It knows nothing about
sprotty, PlantUML or SVG.

```rust
pub struct DiagramGraph {
    pub kind: DiagramKind,          // Bdd | Ibd | StateMachine | ... (closed enum)
    pub qualified_name: String,     // the Diagram element
    pub subject: Option<String>,
    pub nodes: Vec<Node>,           // flat list; nesting via `parent`
    pub edges: Vec<Edge>,
    pub layout_hints: LayoutHints,  // per-kind ELK options (direction, port constraints)
}

pub struct Node {
    pub id: String,                 // shape id (manifest key, or derived — §5.2)
    pub element_ref: Option<String>,
    pub kind: NodeKind,             // Boundary | Block | Port | Compartment | Label | Note | ...
    pub label: String,              // display name
    pub stereotype: Option<String>, // "part def", "port", ...
    pub parent: Option<String>,     // nesting
    pub port_side: Option<Side>,    // for ports: North/East/South/West or None (let ELK choose)
    pub direction: Option<PortDirection>, // in | out | inout
    pub lines: Vec<String>,         // compartment content (attributes, ports...)
    pub is_abstract: bool,
    pub pin: Option<Rect>,          // from `layout:` — x, y and optional w, h
}

pub struct Edge {
    pub id: String,
    pub element_ref: Option<String>,
    pub source: String,             // node id
    pub target: String,
    pub kind: EdgeKind,             // Connection | Flow | Binding | Inheritance | Composition | ...
    pub label: Option<String>,
    pub waypoints: Option<Vec<Point>>, // pinned routing, if any
}
```

Design rules:

- **Closed enums for kinds.** `NodeKind`, `EdgeKind` and `DiagramKind` enumerate exactly
  the §8.16.8 vocabulary. An unknown manifest `kind:` is reported (§8), not stringly
  passed through.
- **Flat node list, `parent` for nesting.** Simple to diff, simple to serialise, and
  the sprotty writer builds the tree in one pass.
- **Pins are optional geometry, never required.** A node without a pin is laid out by
  ELK. A node with one is fixed. The IR carries both; the consumer decides.
- **Element refs are resolved before the IR is built.** The IR carries resolved
  qualified names and display names. Unresolved refs become `W402` findings at
  validation time and `Node { element_ref: None }` in the IR, drawn with a dashed border
  rather than dropped.

A `Diagram` element gets its IR from one of two sources, chosen by what the
frontmatter declares. No new mode field is needed:

| Frontmatter | Source |
|---|---|
| has a `shapes:` block | **manifest** (§5.1) — the author listed the content |
| has a `subject:` and no `shapes:` | **derived** (§5.2) — the generator for `diagramKind` walks the subject |
| has neither | nothing to draw; `W400`/`W401` already cover the missing pieces |

### 5.1 Manifest source

`vis::manifest` turns `shapes:`/`edges:`/`layout:` into an IR, honouring the
**whole** of §8.16.3–8.16.4: string shorthand, map form, default `kind: block`,
`parent:`, `label:`. It is the single manifest parser; `diagram.rs`'s helpers,
`plantuml.rs`'s private copy and `renderer.rs` are removed in its favour.

Malformed manifests stop being silent (§8).

### 5.2 Derived source

```yaml
---
type: Diagram
diagramKind: IBD
subject: UAV::Power::PowerSystem
include: [UAV::Power::PowerSystem::battery, UAV::Power::PowerSystem::pdu]   # optional
exclude: []                                                                   # optional
layout:                                                                       # optional pins
  s-uav-power-powersystem-pdu: {x: 320, y: 80}
---
```

The generator for the diagram's kind walks the subject and produces the IR. Shape ids
are **deterministic**: `s-` plus the element's qualified name lower-cased with `::` and
non-alphanumerics replaced by `-`, so pins survive regeneration and a renamed element
simply loses its pin (it gets re-laid-out, nothing breaks). `include:` restricts the
content to the named members (and whatever edges join them); `exclude:` removes named
members from an otherwise complete view. Both are only meaningful on a derived diagram
(§8).

The two generators that ship first:

**BDD** (`subject:` is a `Package`, `PartDef` or `ItemDef`)
- One `Block` node per `PartDef`/`ItemDef`/`PortDef`/`InterfaceDef`/`ConnectionDef`
  that is a direct member of the subject package (or a direct sub-definition of the
  subject definition).
- `Inheritance` edges from each `supertype:`; `Composition` edges from each part usage
  whose `typedBy:` resolves to another block on the diagram; `Association` edges for
  `ConnectionDef` ends.
- A `Compartment` child listing `features:` (attributes with type and unit) and ports.
- Layout hints: layered, top-to-bottom, inheritance edges reversed so supertypes sit
  above subtypes.

**IBD** (`subject:` is a `PartDef` or `Part`)
- A `Boundary` node for the subject, with its own ports on the boundary edge.
- One `Block` child per owned part usage (a `Part` whose owner is the subject, or a
  `features:` entry typed by a `PartDef`), with the usage's ports as `Port` children.
- `Connection`/`Flow`/`Binding` edges from the subject's `connections:` entries and
  from `Connection`/`Flow` elements whose ends resolve inside the boundary, using the
  dotted-endpoint rules `graph.rs::resolve_endpoint` already implements.
- Port direction from the `PortDef`/`Port` `direction:` field; port side left to ELK
  unless pinned.
- Layout hints: layered, left-to-right, `hierarchyHandling: INCLUDE_CHILDREN`,
  `portConstraints: FIXED_SIDE` where a side is known, `FREE` otherwise.

Both generators are pure functions of `(subject, elements, resolver, filters)` and are
tested against fixture models with golden IR snapshots.

---

## 6. The sprotty contract and the client

### 6.1 Endpoint

`GET /api/diagrams/model/{*qname}` keeps its path and still returns a sprotty
`SGraph`, but a **nested** one:

- a `node` per IR node, with `children` for nested blocks, `port` children for ports,
  a `label` child for the name and a `compartment` child for compartment lines;
- `size` present only when the pin carries `w`/`h`; otherwise omitted so sprotty
  measures the rendered label and ELK sizes the node;
- `position` present only for pinned nodes;
- `edge` per IR edge with `kind` and optional `routingPoints` from pinned waypoints;
- a root-level `layoutOptions` map (ELK option ids, from `LayoutHints`) and a `pinned`
  array of node ids.

`PATCH /api/diagrams/layout/{*qname}` is unchanged in shape and gains one semantic: an
entry it writes is a pin. Writing `null` for a shape id removes the pin. A new
`DELETE /api/diagrams/layout/{*qname}` removes every pin (the "Auto-layout" button).

### 6.2 Layout in the client

The DI container binds sprotty's `IModelLayoutEngine` to `sprotty-elk`'s
`ElkLayoutEngine`, with `elkjs`'s bundled build as the ELK factory. `needsClientLayout`
turns on so label bounds are measured in the hidden render pass before ELK runs.

Pins are honoured by passing each pinned node's position with ELK's interactive
layering and ordering options enabled (`org.eclipse.elk.interactive`,
`org.eclipse.elk.layered.layering.strategy: INTERACTIVE`,
`crossingMinimization.strategy: INTERACTIVE` with `considerModelOrder`), so ELK places
unpinned nodes around the pinned ones instead of ignoring them. When **every** node is
pinned, the `fixed` algorithm is used and ELK only routes edges.

Auto-layout results are **never written back implicitly**. The model only changes when
the user moves a node (that node becomes pinned, via the existing PATCH), presses
*Pin all* (one PATCH carrying every node's current position) or *Auto-layout* (the
DELETE above). This keeps `git diff` on diagram files meaningful.

### 6.3 Editing

`REQ-TRS-DE-004`'s gestures stay and widen:

| Gesture | Manifest diagram | Derived diagram |
|---|---|---|
| Create node | creates element + manifest shape + pin in one guarded write (as today) | creates the element under the subject (a `Part` usage for IBD, a definition for BDD); the view regenerates on reload, no manifest write |
| Delete node | deletes element, prunes shapes/edges in every diagram (as today) | deletes the element; the view regenerates |
| Connect | port-to-port drag adds a `connections:` entry on the owning part and an edge to the manifest | adds the `connections:` entry only |
| Move | writes a pin | writes a pin |

The connect gesture becomes **port-aware**: the drag must start and end on `port`
children; connecting two blocks directly is offered only when each side has exactly one
compatible port, and is refused otherwise with a toast. This replaces today's
"connect any two nodes" behaviour, which produced `connections:` entries between parts
that had no ports.

Every write still goes through `syscribe_model::mutate`'s guarded-write engine and
returns the `WriteResponse` delta; a refusal reverts the optimistic change exactly as
`editor.ts` does today.

### 6.4 Vendoring

`elkjs` and `sprotty-elk` enter `frontend/package.json` and are bundled by esbuild into
the existing `static/js/diagram-editor.js`; no CDN, no runtime Node, in keeping with
`ADR-SYS-DE-001`'s consequence. The bundled ELK adds roughly 1.4 MB to a bundle that is
already 1.1 MB (Mermaid alone is 3.3 MB). ELK runs on the main thread; a Web Worker is
an option if a real model proves slow, and sprotty-elk supports it without API change.

---

## 7. Exports and static output

| Output | Source | Layout | Where used |
|---|---|---|---|
| PlantUML `.puml` | `vis::plantuml` from the IR | PlantUML's own | `syscribe plantuml`, MCP `render_diagram format=plantuml`, docs |
| Mermaid text | `vis::mermaid` from the IR (BDD → `classDiagram`, IBD → `flowchart` with `subgraph` nesting) | Mermaid's own | `syscribe diagram export --format mermaid`, MCP `render_diagram format=mermaid`, GitHub Markdown |
| SVG | `vis::svg` from an IR whose geometry is complete | pins, or a browser-saved companion | `export-html`, MkDocs, GitHub (companion mode) |

The static-SVG policy follows from "the browser is the layout authority":

1. If every node of the IR has a pin, `vis::svg` draws it.
2. Else if the diagram has a companion SVG (`svgMode: companion`, or a file the browser
   saved with *Save companion SVG*), that file is used.
3. Else if a PlantUML companion `.svg` exists, that is used.
4. Else `export-html` emits a placeholder: the diagram's name, kind and subject, and a
   link that opens it in the browser.

Mermaid text generated by Syscribe carries `%% ref:` annotations for every node, so the
existing `W408`/`W409` lints apply to generated output exactly as to hand-written blocks.

The browser gains three buttons beside *Add*/*Connect*/*Delete*: **Pin all**,
**Auto-layout**, and **Save companion SVG** (serialises the current render, with
`sysml:ref` attributes per §8.16.5, and writes `svgFile` through the guarded-write
engine so GitHub shows the picture the engineer approved).

---

## 8. Format and validation additions

Frontmatter (spec §8.16.2, extended when Phase 1 ships):

| Field | Values | Notes |
|---|---|---|
| `include` / `exclude` | list of qualified names | Derived diagrams only (§5.2). |
| `layout` | unchanged | Entries are pins. `w`/`h` optional. |
| `shapes` / `edges` | unchanged | Their presence selects the manifest source. |

Validation codes (added to `prompts/spec/validation.md` and `docs/validation/rules.md`
in the same commit as the code that emits them):

| Code | Severity | Condition |
|---|---|---|
| `E405` | error | `shapes:`, `edges:` or `layout:` is present but not a map of well-formed entries (a sequence, a scalar, an entry missing `ref`/`source`/`target`, an unknown `kind:`). Replaces today's silent empty fallback. |
| `W416` | warning | A `layout:` key names no shape in the diagram (stale pin). |
| `W417` | warning | `include:`/`exclude:` on a manifest diagram (ignored), or an entry that names no member of the subject. |
| `W418` | warning | `subject:` type is not valid for the `diagramKind:` on a derived diagram (§8.16.8's "valid subject types"); the diagram is drawn empty. |

`W400`–`W415`, `E400`–`E404` are unchanged.

---

## 9. Phases

| Phase | Delivers | Requirements |
|---|---|---|
| **0 — Clear the ground** | `vis::ir` + `vis::manifest` as the one parser; shorthand, `parent`, default kind honoured; `E405`/`W416`; sprotty endpoint emits nesting; `sub_mapping` no longer silently replaces a scalar. **Removed outright**: `renderer.rs`, `diagram.rs`'s helpers, `plantuml.rs`'s private parser (it reads the IR from here on), the CLI `diagram` toolkit (`render`/`measure`/`compose`/`layout`/`seq`/`req`, ~6 k lines, and its `/tmp` scratch files), their docs and qual cases. `export-html` embeds only companion SVG until Phase 3. | `REQ-TRS-VIS-001`, `-002`, `-013` |
| **1 — Derived BDD/IBD** | `derive::bdd`, `derive::ibd`, source selection by frontmatter, `include:`/`exclude:`, `W417`/`W418`, spec §8.16 update, golden tests. | `-003`, `-004`, `-005` |
| **2 — ELK in the browser** | `sprotty-elk` + `elkjs` vendored; measured labels; pins semantics; Pin all / Auto-layout; port-aware connect; nested rendering with the shared style. | `-006`, `-007`, `-008`, `-011`, `-012` |
| **3 — Exports and static SVG** | `vis::mermaid`, `vis::svg`, `syscribe diagram export`, MCP `render_diagram format=mermaid`, the §7 fallback chain in `export-html`. | `-009`, `-010` |
| **4 — Coverage** | Endpoint integration tests, Node-side ELK smoke test on a fixture IR, snapshot tests for every writer. Runs alongside every phase; listed separately so it is never "later". | `-014` |
| **Follow-on** | State, Action, Requirement, Sequence, Allocation generators on the same IR. | `-015` |

Each phase follows the repository's requirement-first order: requirements and planning
items committed first, then tests and implementation with the qualification mirror.

---

## 10. Testing

- **IR generators**: fixture models under `crates/syscribe-model/tests/fixtures/vis/`,
  golden JSON snapshots of the IR, one test per rule in §5.2 (inheritance edge present,
  composition inferred, port direction carried, filters applied).
- **Manifest parser**: every §8.16.3/8.16.4 form, plus one test per `E405`/`W416` trigger.
- **Writers**: snapshot tests for PlantUML, Mermaid and SVG against the same fixture IRs;
  the PlantUML snapshots must match today's output for the demo diagrams, byte for byte,
  so the parser swap is proven behaviour-preserving.
- **Server**: axum integration tests on `GET /api/diagrams/model` asserting nesting,
  ports, `layoutOptions` and `pinned`; on `PATCH`/`DELETE` layout.
- **Client**: `npm run typecheck` stays the build gate; a new `npm test` runs a Node
  script that feeds a fixture `SGraph` through `elkjs` with the production
  `layoutOptions` and asserts no two sibling nodes overlap and every port lies on its
  parent's border. This is the one place the layout algorithm itself is exercised.
- **Qualification**: `qual/` mirrors as `TC-TRS-VIS-*` with real `testFunctions`, as for
  every other feature.

---

## 11. Retirements

All of these go in Phase 0. There is no parity gate: the user ruled out backwards
compatibility for rendering, and keeping dead paths alive while the IR lands would
only multiply the surface to migrate.

| Component | Fate |
|---|---|
| `crates/syscribe-model/src/renderer.rs` | Deleted. `export-html` embeds companion SVG only until `vis::svg` lands in Phase 3. |
| `crates/syscribe-model/src/diagram.rs` parse helpers and `default_size` | Deleted; `vis::manifest` is the parser and ELK sizes nodes. |
| `plantuml.rs`'s private `parse_shapes`/`parse_edges` | Deleted; the PlantUML writer reads the IR. Output for the demo diagrams is snapshot-tested before and after so the swap is proven. |
| `crates/syscribe/src/diagram/*` (`render`, `measure`, `compose`, `layout`, `seq`, `req`, `list`) | Deleted with their CLI help, `docs/cli` section, `REQ-TRS-DIAG-004` (superseded by `REQ-TRS-VIS-009`'s not-found rule) and `TC-TRS-DIAG-004`. `syscribe diagram export` is the one diagram subcommand afterwards. |
| `views.tsx` colour table | Replaced by `vis::style` serialised into the sprotty model, so Rust owns the visual language (Phase 2). |

Hand-authored `diagramKind: Mermaid` and inline `diagramKind: PlantUML` diagrams are
untouched by every phase.

---

## 12. Alternatives considered

- **Pure-Rust Sugiyama layout.** One engine for every path, deterministic, testable
  offline. Rejected on 2026-10-07 for cost: a port-aware, hierarchy-aware layered layout
  with orthogonal routing is months of work to reach ELK's quality, and ELK already
  exists, is MIT/EPL-licensed and is the layout sprotty was designed around.
- **ELK on the server through Node.** Consistent output everywhere, at the price of a
  Node runtime for `syscribe` and `syscribe-server`. Rejected: the project's one
  JS build step is dev-time only and should stay that way.
- **Graphviz `dot` as an external tool.** Fast to integrate, but weak at ports and
  nested IBDs, and a second PATH dependency after PlantUML. Rejected.
- **Dropping PlantUML or Mermaid.** Both have users (CI docs, GitHub Markdown). Kept as
  backends; neither is a layout source.

---

## 13. Open points

1. Whether `Save companion SVG` should also embed the ELK positions back as pins, so
   the static picture and the editable one never diverge. Leaning yes; decide in Phase 2.
2. Whether `include:`/`exclude:` should accept glob patterns on qualified names.
   Start with exact names; extend if a real model needs it.
3. Port sides for the BDD compartment view (ports as compartment lines vs. drawn on the
   block). Start with compartment lines; IBD is where ports are drawn.
