# Frontmatter Reference

`FORMAT · FRONTMATTER`

All fields are optional unless marked **Required**. `type:` is required on every element.

## Common fields

| Field | YAML type | Default | Description |
|---|---|---|---|
| `type` | string | — | **Required.** Element type (see [Element Types](elements.md)) |
| `name` | string | filename stem | Display name; defaults to the `.md` filename without extension |
| `id` | string | absent | Stable opaque identifier — required on Requirement, TestCase, ADR, Configuration |
| `supertype` | string or list | absent | Qualified name(s) of parent definition(s) |
| `subsets` | list of strings | absent | Features subsetted by this element |
| `redefines` | string or list | absent | Features redefined by this element |
| `multiplicity` | string | `"1"` | Cardinality, e.g. `"0..*"`, `"1..3"` |
| `isAbstract` | bool | `false` | Cannot be instantiated directly |
| `domain` | string | absent | `system`, `hardware`, or `software` — used in traceability rules §12 |
| `satisfies` | list | absent | Qualified names or REQ-* IDs of Requirements satisfied by this element |

## Features (`features:`)

A list of inline owned features. Each entry is a mapping:

| Sub-field | Description |
|---|---|
| `name` | Feature name |
| `type` | Optional: `Port`, `Action`, `Attribute` — overrides the inferred kind |
| `typedBy` | Qualified name of the definition typing this feature |
| `direction` | `in`, `out`, `inout` — for ports and parameters |
| `multiplicity` | Cardinality of this feature |
| `unit` | Unit string (e.g. `SI::kg`, `SI::V`) |
| `isDerived` | `true` if the value is computed |
| `isConstant` | `true` if the value cannot change after initialisation |

## Ports (`features:` with `type: Port`)

```yaml
features:
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortDef
    direction: in
  - name: telemetryOut
    type: Port
    typedBy: Interfaces::TelemetryPortDef
    direction: out
```

## Connections (`connections:`)

Internal connections between ports, declared on PartDef or Part files.

```yaml
connections:
  - from: avionics.telemetryOut
    to: telemetryOut
    typedBy: Interfaces::TelemetryConnectionDef
```

## Binding connections (`bindingConnections:`)

Binds two features to be identical (equality connector).

```yaml
bindingConnections:
  - left: airframe.telemetryOut
    right: telemetryOut
```

## Traceability fields (Requirements)

| Field | Description |
|---|---|
| `title` | Short human-readable title |
| `status` | `draft` · `review` · `approved` · `implemented` · `verified` |
| `reqDomain` | `system` · `hardware` · `software` |
| `silLevel` | IEC 61508 SIL 1–4 |
| `asilLevel` | ISO 26262 ASIL A–D |
| `derivedFrom` | List of parent REQ-* IDs |
| `breakdownAdr` | Qualified name of the accepted ADR that justifies this derivation |

## Traceability fields (TestCases)

| Field | Description |
|---|---|
| `title` | Short human-readable title |
| `status` | `draft` · `review` · `approved` · `active` · `retired` |
| `testLevel` | `L1` (unit) through `L5` (HIL) |
| `verifies` | List of REQ-* IDs verified by this test case |
| `testFunctions` | List of `{scenario, file, line}` mappings linking Gherkin scenarios to source |

## Diagram fields

| Field | Description |
|---|---|
| `diagramKind` | `BDD` · `IBD` · `StateMachine` · `Requirement` · `Mermaid` · `PlantUML` |
| `subject` | Qualified name of the element this diagram depicts |
| `shapes` | YAML mapping of shape-id → shape descriptor |
| `edges` | YAML mapping of edge-id → edge descriptor |
| `layout` | YAML mapping of shape-id → `{x, y, w, h}` |
| `svgMode` | `inline` — embed SVG directly in the response |

See [Diagrams](diagrams.md) for full shape and edge schemas.

## Operations (`operations:`)

Callable operations and async receptions on PortDef, InterfaceDef, ConnectionDef.

See [Operations](operations.md) for the full schema.
