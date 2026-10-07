# Diagrams

`FORMAT · DIAGRAMS`

Diagrams are `type: Diagram` elements. The `diagramKind:` field selects the rendering path.

## Diagram kinds

| `diagramKind` | Rendering | Description |
|---|---|---|
| `BDD` | SVG (server / PlantUML) | Block Definition Diagram — part/item type hierarchy and compositions |
| `IBD` | SVG (server / PlantUML) | Internal Block Diagram — part usages, ports, and connections within a block |
| `StateMachine` | SVG (server / PlantUML) | State machine — states, transitions, and guards |
| `Requirement` | SVG (server / PlantUML) | Requirement diagram — requirements, derivation, and verification links |
| `Sequence` | SVG (PlantUML) | Sequence diagram — lifelines, messages, returns |
| `Mermaid` | Mermaid.js (client) | Any diagram expressible in Mermaid graph syntax |

## Structured diagrams (BDD, IBD, StateMachine, Requirement)

These diagrams carry their content in YAML frontmatter, from one of two sources. A `BDD` or `IBD` with a `subject:` and no `shapes:` is **derived** from the model (next section) — the preferred form for those kinds. A diagram with a `shapes:` block is **manifest**-sourced: the hand-listed alternative, where the author enumerates the `shapes`, `edges`, and optional `layout` pins, and the only form for kinds that have no generator yet. The presence of `shapes:` selects the source; there is no mode field.

### `shapes:` — mapping of shape-id to descriptor

```yaml
shapes:
  s-uavsystem: {ref: "UAV::UAVSystem", kind: PartDef}
  s-avionics:  {ref: "UAV::Avionics::AvionicsBay", kind: PartDef}
  s-fc:        {ref: "UAV::Avionics::FlightController", kind: Part, parent: s-avionics}
```

Each shape descriptor:

| Sub-field | Description |
|---|---|
| `ref` | Qualified name of the model element this shape represents; validated by W402 |
| `kind` | Rendering hint: `PartDef`, `Part`, `Port`, `boundary`, `state`, `initial`, `Requirement`, etc. |
| `parent` | Shape-id of the enclosing boundary shape (for IBD nesting) |

Sub-feature refs (e.g. `UAV::Avionics::FlightController::powerIn`) do not resolve as top-level elements — W402 suppresses the warning for these.

### `edges:` — mapping of edge-id to descriptor

```yaml
edges:
  e-comp:  {source: s-uavsystem, target: s-avionics, kind: composition}
  e-power: {source: s-fc-power,  target: s-imu,      kind: flowConnection}
```

Each edge descriptor:

| Sub-field | Description |
|---|---|
| `source` | Shape-id of the source end; validated by W403 |
| `target` | Shape-id of the target end; validated by W403 |
| `kind` | `composition`, `flowConnection`, `derivedFrom`, `verifies`, `allocatedTo`, `transition` |
| `ref` | Optional: model element this edge represents (e.g. a feature or connection usage) |

### `layout:` — pixel coordinates

```yaml
layout:
  s-uavsystem: {x: 20,  y: 20,  w: 200, h: 56}
  s-avionics:  {x: 20,  y: 140, w: 200, h: 56}
```

`layout:` is the trigger for the SVG renderer. A diagram without a `layout:` block renders as "no layout defined."

## Derived diagrams (BDD, IBD)

A `BDD` or `IBD` needs no manifest at all. Declare the kind and the subject, and the
generator reads the content from the model every time the diagram is built:

```yaml
---
type: Diagram
name: PowerSystemDerivedIBD
diagramKind: IBD
subject: UAV::Power::PowerSystem
---
```

Adding a part, a port or a `supertype:` to the model changes the picture; nothing in the
diagram file needs editing. The demo model ships one of each: `Diagrams::UAVSystemDerivedBDD`
(subject: the `UAV` package) and `Diagrams::PowerSystemDerivedIBD` (subject:
`UAV::Power::PowerSystem`; compare the hand-listed `Diagrams::PowerSystemIBD`, the same view
as a manifest).

### What the generator draws

**BDD** — subject: a `Package`, `PartDef` or `ItemDef`.

- One block per `PartDef`, `ItemDef`, `PortDef`, `InterfaceDef` or `ConnectionDef` that is a
  direct member of the subject package (for a definition subject: the definition itself plus
  its direct sub-definitions). Actions, states and requirements never appear.
- A compartment listing the block's attribute features (`mass : Real [kg]`) and ports
  (`port powerOut : PowerPort (out)`).
- `inheritance` edges from `supertype:`; `composition` edges from part usages (inline
  `features:` typed by a definition, and child `Part`/`Item` elements), labelled with the usage
  name and a non-1 multiplicity (`motor [2]`); an `association` edge for a `ConnectionDef`
  whose two end types are blocks on the diagram.
- An edge is drawn only when both of its ends are on the diagram.

**IBD** — subject: a `PartDef` or `Part`.

- A boundary for the subject with its own ports (a `Part` subject also shows its definition's
  ports).
- One block per owned part usage — inline `features:` typed by a `PartDef`, or child `Part`
  elements — labelled `name : Type [mult]`, with ports from the usage's own features and from
  its definition, each carrying its `direction`.
- Edges from the subject's `connections:` (connection), `flowConnections:` (flow),
  `bindingConnections:` (binding) and `successionConnections:` (succession). Endpoints are the
  usual dotted chains: `engine.powerOut` is port `powerOut` of usage `engine`; a single segment
  is a usage or a boundary port. A chain that reaches nothing draws no edge — the generator
  never invents a port.

Other kinds (`StateMachine`, `Requirement`, …) have no generator yet; a derived diagram of
such a kind is drawn empty, so keep using a manifest for them.

### Narrowing the view: `include:` / `exclude:`

```yaml
diagramKind: BDD
subject: UAV
include: [UAV::UAVSystem, Airframe]   # qualified name, or name relative to the subject
exclude: [Airframe]                    # applied after include
```

`include:` keeps only the named members (BDD: definitions in the package; IBD: owned part
usages) and the edges joining them; `exclude:` removes members, and every edge touching them,
from an otherwise complete view. Either takes a string or a list. An entry that names no member
of the subject is **W417**; `include:`/`exclude:` on a manifest diagram is one **W417** and is
ignored.

### Pins

Derived shape ids are deterministic: `s-` plus the element's qualified name lower-cased with
`::` and other non-alphanumerics replaced by `-` — `UAV::Power::PowerSystem::pdu` becomes
`s-uav-power-powersystem-pdu`. A `layout:` block therefore works on a derived diagram exactly as
on a manifest one: pins survive regeneration, and a renamed element simply loses its pin (W416)
and is laid out automatically again.

```yaml
layout:
  s-uav-power-powersystem-pdu: {x: 320, y: 80}
```

### Codes

| Code | Condition |
|---|---|
| W417 | `include:`/`exclude:` on a manifest diagram (ignored), or an entry that names no member of the subject |
| W418 | A derived diagram's `subject:` type is not valid for its `diagramKind:` (BDD: `Package`/`PartDef`/`ItemDef`; IBD: `PartDef`/`Part`); the diagram is drawn empty |

An unresolved `subject:` is still W401, and a derived diagram never raises W402/W403 for the
shapes it generates.

## Mermaid diagrams

Set `diagramKind: Mermaid` and include a fenced ` ```mermaid ` block in the document body. The validator fires **E400** if the block is absent.

````markdown
---
type: Diagram
name: RequirementTrace
diagramKind: Mermaid
subject: Requirements
---

Requirement derivation tree.

```mermaid
graph TD
  %% ref: REQ-UAV-PERF-000
  PERF["REQ-UAV-PERF-000<br/>Mission Performance"]
  %% ref: REQ-UAV-COMM-001
  COMM["REQ-UAV-COMM-001<br/>Data Link ≥ 5 km"]
  PERF --> COMM
```
````

The Mermaid.js runtime is loaded from CDN and renders the diagram client-side when the tab is activated.

## PlantUML companion diagrams

`pumlMode: companion` opts a diagram into the PlantUML workflow. Syscribe generates
a `.puml` source file; your PlantUML toolchain (JAR, CI step, IDE plugin) renders it
to SVG. Every shape emits a `[[URL]]` hyperlink back to the element's detail page in
the web browser.

### Frontmatter fields

| Field | Description |
|---|---|
| `pumlMode: companion` | Opt in; the only supported value |
| `pumlFile: ./MyDiagram.puml` | Override companion path (default: `<stem>.puml`) |

The body must reference the anticipated SVG so the diagram is visible on GitHub
and other Markdown renderers. `syscribe plantuml` injects a Markdown image link
automatically if the body has no image reference yet. Both `![name](path.svg)` and
`<img src="path.svg">` satisfy W413.

```yaml
---
type: Diagram
name: UAVSystemBDD
diagramKind: BDD
pumlMode: companion
pumlFile: ./UAVSystemBDD.puml
subject: UAV::UAVSystem
shapes:
  s-uavsystem: {ref: "UAV::UAVSystem", kind: PartDef}
  s-avionics:  {ref: "UAV::Avionics::AvionicsBay", kind: PartDef, parent: s-uavsystem}
edges:
  e-comp: {source: s-uavsystem, target: s-avionics, kind: composition}
---

![UAVSystemBDD](./UAVSystemBDD.svg)
```

### CLI workflow

```bash
# 1. Generate .puml source files for all companion diagrams
syscribe -m model/ plantuml

# 2. Render .puml → .svg (needs plantuml on PATH or PLANTUML_JAR env var)
syscribe -m model/ plantuml render
syscribe -m model/ plantuml render --jar /opt/plantuml/plantuml.jar

# Generate a single diagram
syscribe -m model/ plantuml Diagrams::UAVSystemBDD --output -
```

### Style configuration (`.syscribe.toml`)

```toml
[plantuml]
theme = "spacelab"                    # any PlantUML built-in theme
# style_file = "style/custom.puml"   # !include — takes precedence over theme
# base_url   = "https://my-server"   # clickable link prefix (default: http://localhost:3000)
# jar        = "/opt/plantuml.jar"   # JAR path for `plantuml render`
```

### Supported `diagramKind` values for PlantUML generation

| `diagramKind` | PlantUML output |
|---|---|
| `BDD` | Class diagram — `class "Name" <<part def>>`, `*--` composition |
| `IBD` | Component diagram — `rectangle` boundary, `component` blocks |
| `StateMachine` | State diagram — `[*]` initial, `state "Name" as id` |
| `Sequence` | Sequence diagram — `actor`/`participant`, `->` messages |
| `Requirement` | Class diagram — `<<requirement>>` stereotype, `..>` edges |

`Mermaid` and unknown kinds are skipped with a warning.

## Validation rules for diagrams

| Code | Severity | Condition |
|---|---|---|
| E400 | Error | `diagramKind: Mermaid` but body has no ` ```mermaid ` block |
| E402 | Error | `svgFile:` path does not exist on disk |
| E403 | Error | `pumlMode` has an unrecognized value (only `companion` is supported) |
| E404 | Error | `pumlMode: companion` set but `diagramKind` is absent |
| W400 | Warning | Diagram element has no `diagramKind` — rendering mode ambiguous |
| W401 | Warning | `subject:` does not resolve to a known element |
| W402 | Warning | A shape `ref:` does not resolve (and is not a sub-feature of a known element) |
| W403 | Warning | An edge `source` or `target` is not a defined shape id in this diagram |
| W417 | Warning | `include:`/`exclude:` on a manifest diagram (ignored), or an entry that names no member of the subject |
| W418 | Warning | A derived diagram's `subject:` type is not valid for its `diagramKind:`; the diagram is drawn empty |
| W413 | Warning | `pumlMode: companion` but body has no `<img` tag |
| W414 | Warning | `pumlMode: companion` but the `.puml` companion file does not exist yet |
| W415 | Warning | `[plantuml] style_file` in `.syscribe.toml` does not exist on disk |
