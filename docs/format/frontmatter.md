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

## TestPlan fields (`type: TestPlan`)

A `TestPlan` (stable `TP-*` id) groups reusable TestCases by product and scope.

| Field | Description |
|---|---|
| `title`, `status` | Required; status `draft` · `review` · `approved` · `active` · `retired` |
| `scope` | Recommended `unit·smoke·integration·hil·certification·security·regression` (other → `W610`); discriminates plans over the same config |
| `configurations` | A `Configuration` id or list of ids — the product variant(s) the plan is for; absent = config-agnostic. Each must resolve (`E606`) |
| `demonstrates` | Optional list of goals/requirements the plan is evidence for (`E603` if unresolved); not required |
| `testCases` | Explicit `TC-*` members (`E601` if not a TestCase) |
| `selection` | Additive query: `testLevels` (L1–L5, `E602`), `domains` (system/hardware/software, `E605`), `tags` |

Effective members = `testCases` ∪ `selection` matches. Surfaced by `testplan` and the `--plan TP-X` lens on `matrix`/`verification-depth`/`audit`. Codes `E600`–`E606` / `W610`–`W616` in the [Rule Reference](../validation/rules.md).

## Implementation trace (`implementedBy:`)

`Part`/`PartDef` elements may link to the source artifact(s) that realise them, closing the V-model leg `Requirement ─satisfies→ Architecture ─implementedBy→ Code ─verifies→ Test`.

| Field | Description |
|---|---|
| `implementedBy` | String or list of paths to the implementing source. Resolved like a TestCase's `sourceFile` (model-/repo-relative, `model:`/`repo:` prefixes, absolute, `file://`, remote `scheme://`). A missing **local** path on a non-`draft` element emits **W023** (§12.8); remote URIs are accepted as external pointers. Opt-in, draft-suppressed, gate with `--deny W023`. |

```yaml
type: PartDef
satisfies: [REQ-SCHED-001]
implementedBy:
  - src/scheduler/mod.rs
  - repo:src/scheduler/bitmap.rs
```

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

## Variability / product-line fields (§9)

Opt-in: ignored when the model declares no `FeatureDef`. See the [Variability guide](../model-guide/variability.md).

| Field | Applies to | Description |
|---|---|---|
| `appliesWhen` | any element (incl. `TestCase`) **or a `Package`** | Boolean expression over `FeatureDef` qualified names (`and`/`or`/`not`/parens; bare QName or list = AND). The element is active only in variants where it holds. On a `Package` it gates the whole subtree transitively (effective condition = own, else nearest ancestor package's); at most one declaration per path (`E228`), empty gated package → `W026`. |
| `groupKind` | `FeatureDef` | How this feature's **children** are grouped: `optional` · `alternative` (XOR) · `or` · `mandatory` (legacy shorthand for a mandatory member) |
| `mandatory` | `FeatureDef` | Boolean **membership** vs the parent, orthogonal to `groupKind`: `true` = selected whenever the parent is (or always, if top-level). Lets a node be both `mandatory: true` and `groupKind: alternative` (a mandatory XOR group). |
| `cardinality` | `FeatureDef` (or/alternative) | Selected-children bound, e.g. `"1..*"` |
| `parentFeature` | `FeatureDef` | Explicit parent (else inferred from directory nesting) |
| `requires` | `FeatureDef` | Qualified names of features that must also be selected |
| `excludes` | `FeatureDef` | Qualified names of features that must not be co-selected |
| `parameters` | `FeatureDef` | Typed parameters: each `{name, type, range:"min..max", enumValues, default, isFixed, isRequired, value}` (§9.7) |
| `featureModel` | `Configuration`, `FeatureDef` | Qualified name of the feature-model package |
| `features` | `Configuration` | **Map** of `<FeatureDef qname>: true/false` (the selection; absent = deselected) |
| `parameterBindings` | `Configuration` | Map of `<FeatureDef qname>::<param>: <value>` |
| `parameterConstraints` | package `_index.md` | List of cross-feature constraints `{id, expression, severity, appliesWhen}` |
| `tags` | any element | Free-text labels; filter with `--tag` (orthogonal to the feature model) |

A `Configuration` requires `id` (`CONF-*`), `title`, `status`, and `featureModel`. Validation codes for these fields are in the [Rule Reference](../validation/rules.md) (PLE errors `E2xx`, projection `E226`/`E227`, warnings `W01x`–`W022`).

## Custom fields (`custom_fields:`)

| Field | Description |
|---|---|
| `custom_fields` | On any element: a flat map of `string → scalar` or `list-of-scalars` for user-defined data. Freeform keys; serialised sorted. A value that is not a scalar/list-of-scalars warns `W041`. Query with `--where custom.<key>` on `ls`/`find`/`list`; rendered read-only by `show` and the web detail panel. See [Custom Fields](../model-guide/custom-fields.md). |

