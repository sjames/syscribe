# Design: Visualisation — one Diagram IR, ELK layout in the browser

**Status:** Proposed — decisions ratified in `ADR-SYS-VIS-001`, requirements `REQ-TRS-VIS-000..015`
**Date:** 2026-10-07

> **Implementation note.** Phase 0 (§9) landed on 2026-10-07: `syscribe_model::vis::{ir,
> manifest, sprotty}`, `E405`/`W416`, the PlantUML writer on the IR, the nested
> `/api/diagrams/model` contract with a client that renders it, and the removals of §11
> (`PI-VIS-001`). Phase 1 followed the same day (`c2adb37c`, `d348f9be`): `vis::derive::{bdd,
> ibd}`, source selection by frontmatter, `include:`/`exclude:`, `W417`/`W418` (`PI-VIS-002`).
> Phase 2 landed in `cacff379` (backend) and `d2e72234` (client), `PI-VIS-003`: one visual
> language in Rust (`vis::style`) carried per node/port/edge in the sprotty graph; `sprotty-elk`
> and `elkjs` vendored into the esbuild bundle with ELK layout on open over measured labels,
> orthogonal routing and edge keywords/labels placed by ELK; pins honoured through ELK's
> interactive layering (or the `fixed` algorithm when every node is pinned); `PATCH` `null`
> unpins one shape and `DELETE /api/diagrams/layout` clears every pin; `PUT /api/diagrams/svg`
> saves a companion SVG; the *Pin all* / *Auto-layout* / *Save companion SVG* buttons; a
> port-aware connect gesture that sends dotted feature chains; and `npm test` running the ELK
> layout check (`REQ-TRS-VIS-006/007/008/011/012`). The §1 table now describes history.
> Phases 3 and 4 are open.
> This document records the design the four user decisions of 2026-10-07 fixed (§3) and
> is the reference the `REQ-TRS-VIS-*` requirements and `PI-VIS-*` planning items point
> at. Sections are updated in place as phases ship; §6 describes the client as built.
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
- `size` on every node, port, compartment and label — the Rust-computed box of
  `vis::size`, or a pin's own `w`/`h` when it records both (Phase 3b,
  `REQ-TRS-VIS-017`; before it, only a pin's size was sent and sprotty measured the rest);
- `position` present only for pinned nodes;
- `edge` per IR edge with `kind` and optional `routingPoints` from pinned waypoints;
- a root-level `layoutOptions` map (ELK option ids, from `LayoutHints`) and a `pinned`
  array of node ids;
- the resolved style per element (§6.5): `style: {fill, stroke, headerFill, text, dashed}`
  and optional `banners` on a node, `style: {fill, stroke, glyph}` and `side` on a port,
  `style: {stroke, dash, width, arrowTarget, arrowSource, keyword}` on an edge.

`PATCH /api/diagrams/layout/{*qname}` is unchanged in shape and gains one semantic: an
entry it writes is a pin (`x`/`y`, plus `w`/`h` when given; a later `x`/`y`-only patch
keeps the size). Writing `null` for a shape id removes the pin. A new
`DELETE /api/diagrams/layout/{*qname}` removes every pin — it drops the diagram's whole
`layout:` key (the *Auto-layout* button). `PUT /api/diagrams/svg/{*qname}` takes
`{ "svg": "<svg …>" }` and writes the companion file (§7). All three are guarded writes
that return the `WriteResponse` delta; an unknown qname or a non-`Diagram` target is
refused with a reason, and a `PUT` body that is not an SVG document writes nothing.
The module doc of `vis/sprotty.rs` is the normative description of the JSON.

### 6.2 Layout in the client

The DI container binds sprotty's `IModelLayoutEngine` to `sprotty-elk`'s
`ElkLayoutEngine`, with `elkjs`'s bundled build as the ELK factory. `needsClientLayout`
turns on so label bounds are measured in the hidden render pass before ELK runs.

As built (`frontend/src/layout.ts`), the client does three things before and after ELK:

- **Sizes from the server (Phase 3b, `REQ-TRS-VIS-017`).** Every node, port, compartment
  and label — the `«stereotype»`/`banner`/name labels of a node, a compartment's line
  labels, a port's name label, an edge's `«keyword»`/`label` children — arrives with a
  `size` computed in Rust (`vis::size`) from the shared text metrics (`vis::metrics`:
  Helvetica → Arial → Liberation Sans → DejaVu Sans, with an approximate fallback and an
  8 % + 2 px width margin), following the client's own `vbox` recipe. The client treats a
  carried size as authoritative (`prepareForLayout` keeps it, the hidden pass measures only
  what arrived without one), so the browser's ELK and the embedded ELK start from identical
  numbers. A sized leaf node is laid out at exactly its size (`PORTS MINIMUM_SIZE`); a sized
  compound node uses it as its minimum.
- **Configuration.** `SyscribeLayoutConfigurator` maps the root's `layoutOptions` onto the
  ELK graph with the project's spacing defaults and `elk.edgeRouting: ORTHOGONAL`; node
  sizes come from the measured bounds (`NODE_LABELS PORTS PORT_LABELS MINIMUM_SIZE`), port
  sides from `side` (or a direction-based fallback once any sibling port has a side), and
  compound nodes get an inner padding below their label stack.
- **Pre/post-processing.** Edges of the kinds in `syscribe.reversedEdgeKinds` (inheritance)
  are flipped before ELK so supertypes sit above subtypes and flipped back afterwards;
  every edge section is translated from its ELK container's coordinates into the root's,
  because sprotty keeps all edges at the root.

Pins are honoured by passing each pinned node's position (`elk.position`) with ELK's
interactive layering and ordering options enabled (`elk.interactive`,
`elk.layered.layering.strategy: INTERACTIVE`,
`elk.layered.crossingMinimization.strategy: INTERACTIVE`,
`elk.layered.considerModelOrder.strategy: NODES_AND_EDGES`), so ELK places unpinned
nodes around the pinned ones instead of ignoring them. The same options must be set on
every compound node, not only the root — ELK refuses a hierarchy whose children use a
different crossing-minimisation strategy. When **every** node is pinned, the `fixed`
algorithm is used instead: ELK takes the positions as given (a pinned `w`/`h` wins over
the smaller measured size), **routes each edge as a straight line** rather than
orthogonally, and places no labels at all, so the client's postprocessor places node
labels, port labels and edge labels itself in that mode (see §13).

Auto-layout results are **never written back implicitly**. The model only changes when
the user moves a node (that node becomes pinned, via the existing PATCH), presses
*Pin all* (one PATCH carrying every placed node's current position and size) or
*Auto-layout* (the DELETE above, then a re-fetch and a fresh ELK run). This keeps
`git diff` on diagram files meaningful.

### 6.3 Editing

`REQ-TRS-DE-004`'s gestures stay and widen:

| Gesture | Manifest diagram | Derived diagram |
|---|---|---|
| Create node | creates element + manifest shape + pin in one guarded write (as today) | creates the element under the subject (a `Part` usage for IBD, a definition for BDD); the view regenerates on reload, no manifest write |
| Delete node | deletes element, prunes shapes/edges in every diagram (as today) | deletes the element; the view regenerates |
| Connect | port-to-port gesture adds a `connections:` entry on the owning part and an edge to the manifest | adds the `connections:` entry only |
| Move | writes a pin | writes a pin |

The connect gesture is **click-source-then-click-target**, not a literal drag: sprotty's
standalone package ships no edge-creation tool (that is the GLSP piece `ADR-SYS-DE-001`
declines), so *Connect* mode swaps in a `MouseListener` that remembers the first
connectable element clicked and fires on the second. A click on a label or compartment
walks up to the nearest `node` or `port`.

The gesture is **port-aware** (`frontend/src/connect-rules.ts`, pure functions over the
schema, as built):

- two ports are joined when their directions are compatible — `out`→`in` or `in`→`out`;
  an `inout` or undirected port goes with anything; two ports of the same direction are
  refused ("both ports have the same direction");
- a block may stand in for its **single** compatible port: when one or both ends are
  blocks, the candidate pairs are every (source port, target port) combination that is
  compatible; exactly one pair is accepted, none is refused ("no compatible port pair …
  an out port must meet an in port"), and more than one is refused as ambiguous, listing
  the pairs and asking for the two ports to be connected directly;
- a block with no ports at all is refused ("has no ports to connect from/to").

Every refusal is a toast and nothing is written. An accepted pair becomes a `connection`
edge between the two port ids (optimistically) and one `POST /api/connections` whose
`from`/`to` are the **dotted feature chains** `add_connection` resolves relative to the
owner — the diagram's `subject:` (falling back to the diagram's own qname): a port
`A::B::battery::powerOut` owned by `A::B` is sent as `battery.powerOut`, the owner's own
port `A::B::mainPowerOut` as `mainPowerOut`; when a manifest port's `ref` is not spelled
under the owner, the chain is rebuilt from the enclosing blocks' names, skipping the
boundary. The `diagram:` sync block (`ADR-SYS-DE-001`'s transactional edge-into-manifest
write) is sent only for a manifest diagram; a derived diagram has no manifest to sync, so
the `connections:` entry is the whole transaction and the view regenerates on reload.

The client decides "derived or manifest" by a **heuristic on shape ids**
(`isDerivedDiagram`): the diagram has a `subject:` and every root shape's id equals the
deterministic slug of its `ref` (`s-` + the lower-cased ref with non-alphanumerics
replaced by `-`, which is what `vis::derive` emits), whereas a manifest carries the
author's own keys. The endpoint does not yet flag this explicitly — see §13.

This replaces the earlier "connect any two nodes" behaviour, which produced
`connections:` entries between parts that had no ports.

Every write still goes through `syscribe_model::mutate`'s guarded-write engine and
returns the `WriteResponse` delta; a refusal reverts the optimistic change exactly as
`editor.ts` does today.

### 6.4 Vendoring

`elkjs` (`^0.8.2`) and `sprotty-elk` (`^1.4.0`, matching the installed `sprotty`) are
in `frontend/package.json` and bundled by esbuild into the existing
`static/js/diagram-editor.js`; no CDN, no runtime Node, in keeping with
`ADR-SYS-DE-001`'s consequence. The DI container (`container.ts`) loads `elkLayoutModule`,
binds the bundled `elkjs` build as the `ElkFactory`, rebinds `ILayoutConfigurator` and
binds the pre/post-processor pair from `layout.ts`, points `TYPES.IModelLayoutEngine` at
`ElkLayoutEngine`, and sets `needsClientLayout: true` so `LocalModelSource.setModel`
runs the hidden measuring pass before the engine sees the graph. The bundled ELK adds
roughly 1.4 MB to a bundle that was 1.1 MB (Mermaid alone is 3.3 MB). ELK runs on the
main thread; a Web Worker is an option if a real model proves slow, and sprotty-elk
supports it without API change.

### 6.5 Style

The client's views hold no colour table. `vis::style` (Rust) resolves a `NodeStyle`,
`PortStyle` or `EdgeStyle` for every IR element — element type first, then node kind,
dashed when unresolved; the §8.16.8 arrowhead/dash/keyword row per edge kind; the port
glyph per direction; one `«Name»` banner per applied `MetadataDef` stereotype — and
`vis::sprotty` serialises it camelCase onto the element (§6.1). `views.tsx` reads
`style`/`banners`/`side` and draws; the same values are what `vis::svg` will emit in
Phase 3. The `npm test` script (`frontend/test/elk-layout.test.mjs`) is the one place the
layout algorithm itself is exercised outside a browser: it runs the bundled `elkjs` over a
fixture IBD with the configurator's option values and asserts that every port sits on its
parent's border, no two siblings overlap, children and labels fit inside their parent, and
every edge is routed.

---

## 7. Exports and static output

| Output | Source | Layout | Where used |
|---|---|---|---|
| PlantUML `.puml` | `vis::plantuml` from the IR | PlantUML's own | `syscribe plantuml`, MCP `render_diagram format=plantuml`, docs |
| Mermaid text | `vis::mermaid` from the IR (BDD → `classDiagram`, IBD → `flowchart` with `subgraph` nesting) | Mermaid's own | `syscribe diagram export --format mermaid`, MCP `render_diagram format=mermaid`, GitHub Markdown |
| SVG | `vis::svg` from any IR with shapes | pins, else the embedded ELK (`vis::layout`) | `syscribe diagram export --format svg`, MCP `render_diagram format=svg`, `export-html`, MkDocs |

The static-SVG policy (Phase 3b, `REQ-TRS-VIS-016`; the ADR addendum "the same ELK engine
inside the executable") is: **draw the IR**. `vis::svg` draws a fully pinned diagram from its
pins and lays out every other one first with `vis::layout` — the vendored `elk.bundled.js`
(`crates/syscribe-model/vendor/elkjs/`, pinned to the client's `elkjs` version by a test) run
in-process under QuickJS with the client's own options (§6.2), the client's pin handling
(`elk.position` + interactive strategies; `fixed` when every node is pinned) and the
Rust-computed sizes of §6.2 — so the executable and the browser produce the same picture from
the same diagram, verified by a Node-vs-QuickJS determinism test. A companion SVG is no longer
needed to show a diagram anywhere; it remains the way to publish a picture on GitHub.

`export-html` embeds each diagram by the first applicable rule:

1. The IR has shapes: `vis::svg` draws it (from pins, or laid out).
2. Else if the diagram has a companion SVG (`svgMode: companion`, or a file the browser
   saved with *Save companion SVG*), that file is used.
3. Else if a PlantUML companion `.svg` exists, that is used.
4. Else a placeholder: the diagram's name, kind and subject, and a link that opens it in
   the browser.

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
| **2 — ELK in the browser** (landed: `cacff379`, `d2e72234`) | `sprotty-elk` + `elkjs` vendored; measured labels; pins semantics; Pin all / Auto-layout / Save companion SVG; port-aware connect; nested rendering with the shared style. | `-006`, `-007`, `-008`, `-011`, `-012` |
| **3 — Exports and static SVG** | `vis::mermaid`, `vis::svg`, `syscribe diagram export`, MCP `render_diagram format=mermaid`, the §7 fallback chain in `export-html`. | `-009`, `-010` |
| **3b — ELK in the executable** (landed) | `vis::metrics` (promoted from `svgkit`), `vis::size` (Rust-owned sizes carried as `size` on every sprotty element and consumed by the client), `vis::layout` (the vendored `elk.bundled.js` under QuickJS with the client's configuration), `vis::svg` drawing any diagram, `diagram export`/`export-html`/MCP `render_diagram` without pins, the Node-vs-QuickJS determinism test and the bundle-version pin test. | `-016`, `-017` |
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

1. ~~Whether `Save companion SVG` should also embed the ELK positions back as pins, so
   the static picture and the editable one never diverge.~~ **Decided in Phase 2: no.**
   *Save companion SVG* writes only the `.svg` (and `svgMode`/`svgFile`/the `<img>` once);
   pinning stays a separate, explicit *Pin all*, so saving a picture never silently
   changes `layout:`. `REQ-TRS-VIS-011` accepts either answer.
2. Whether `include:`/`exclude:` should accept glob patterns on qualified names.
   Start with exact names; extend if a real model needs it.
3. Port sides for the BDD compartment view (ports as compartment lines vs. drawn on the
   block). Start with compartment lines; IBD is where ports are drawn.
4. **An explicit `derived` flag on the graph root.** The client's `isDerivedDiagram`
   (§6.3) infers "no manifest to sync" from shape ids matching `vis::derive`'s slugs. It is
   correct for every diagram the generators emit, but a manifest author who happens to
   key every shape by its ref slug would be mis-read as derived and the connect gesture
   would skip the manifest sync. `vis::sprotty` should carry `derived: true|false` on the
   root (it knows the source) and the client should read that instead; `REQ-TRS-VIS-006`'s
   contract is extended, not changed, by the extra field.
5. **Orthogonal routing for fully pinned diagrams.** With every node pinned the client
   switches ELK to the `fixed` algorithm, which only draws straight lines between the
   pinned ends and places no labels (the postprocessor places them, §6.2). A pinned IBD
   therefore looks different from the same IBD one drag earlier. Options: keep `layered`
   with every node carrying `elk.position` and interactive options (routes orthogonally,
   but ELK may still nudge positions), or run `fixed` for nodes and a separate ELK
   edge-routing pass. Decide when a real model with a hand-pinned IBD shows the
   difference matters.
