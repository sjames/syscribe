# Diagrams

`FORMAT · DIAGRAMS`

Diagrams are `type: Diagram` elements. The `diagramKind:` field selects the rendering path.

## Diagram kinds

| `diagramKind` | Rendering | Description |
|---|---|---|
| `BDD` | SVG (server) | Block Definition Diagram — part/item type hierarchy and compositions |
| `IBD` | SVG (server) | Internal Block Diagram — part usages, ports, and connections within a block |
| `StateMachine` | SVG (server) | State machine — states, transitions, and guards |
| `Requirement` | SVG (server) | Requirement diagram — requirements, derivation, and verification links |
| `Mermaid` | Mermaid.js (client) | Any diagram expressible in Mermaid graph syntax |
| `PlantUML` | PlantUML (future) | Reserved for PlantUML integration |

## Structured diagrams (BDD, IBD, StateMachine, Requirement)

These diagrams declare their content in YAML frontmatter. The web server builds the SVG from the `shapes`, `edges`, and `layout` fields.

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

## Mermaid diagrams

Set `diagramKind: Mermaid` and include a fenced ` ```mermaid ` block in the document body. The validator fires **E400** if the block is absent.

```yaml
---
type: Diagram
name: RequirementTrace
diagramKind: Mermaid
subject: Requirements
---

Requirement derivation tree.

```mermaid
graph TD
  PERF["REQ-UAV-PERF-000<br/>Mission Performance"]
  COMM["REQ-UAV-COMM-001<br/>Data Link ≥ 5 km"]
  PERF --> COMM
```
```

The Mermaid.js runtime is loaded from CDN and renders the diagram client-side when the tab is activated.

## Validation rules for diagrams

| Code | Severity | Condition |
|---|---|---|
| E400 | Error | `diagramKind: Mermaid` but body has no ` ```mermaid ` block |
| E401 | Error | `diagramKind: PlantUML` but body has no ` ```plantuml ` block |
| E402 | Error | `svgFile:` path does not exist on disk |
| W400 | Warning | Diagram element has no `diagramKind` — rendering mode ambiguous |
| W401 | Warning | `subject:` does not resolve to a known element |
| W402 | Warning | A shape `ref:` does not resolve (and is not a sub-feature of a known element) |
| W403 | Warning | An edge `source` or `target` is not a defined shape id in this diagram |
