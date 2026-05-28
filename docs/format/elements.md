# Element Types

`FORMAT · ELEMENT TYPES`

Every `.md` file in the model tree is one element. The `type:` field in YAML frontmatter selects the element type. Unknown values are accepted and stored as `Unknown` — the validator emits no error, though cross-reference checks still apply.

## Definitions

| Type | SysMLv2 keyword | Description |
|---|---|---|
| `PartDef` | `part def` | Classifies structural components |
| `ItemDef` | `item def` | Classifies things that flow through ports |
| `AttributeDef` | `attribute def` | Classifies scalar properties |
| `PortDef` | `port def` | Classifies interaction points |
| `ConnectionDef` | `connection def` | Classifies connections between ports |
| `InterfaceDef` | `interface def` | Specifies compatible connection ends |
| `ActionDef` | `action def` | Classifies behaviours |
| `ConstraintDef` | `constraint def` | Classifies constraint expressions |
| `RequirementDef` | `requirement def` | Classifies requirement text templates |
| `CalculationDef` | `calculation def` | Classifies calculations |
| `StateDef` | `state def` | Classifies state machines |
| `FlowDef` | `flow def` | Classifies flow connections |
| `UseCaseDef` | `use case def` | Classifies use cases |
| `ViewpointDef` | `viewpoint def` | Classifies viewpoints |
| `ViewDef` | `view def` | Classifies views |
| `MetadataDef` | `metadata def` | Classifies metadata annotations |
| `EnumerationDef` | `enumeration def` | Classifies enumeration types |
| `FeatureDef` | *(PLE)* | Product-line feature definition |
| `VerificationCaseDef` | `verification case def` | Classifies verification cases |
| `AnalysisCaseDef` | `analysis case def` | Classifies analysis cases |

## Usages

| Type | SysMLv2 keyword | Description |
|---|---|---|
| `Part` | `part` | Usage of a PartDef |
| `Item` | `item` | Usage of an ItemDef |
| `Port` | `port` | Usage of a PortDef |
| `Connection` | `connect` | Usage of a ConnectionDef |
| `Interface` | `interface` | Usage of an InterfaceDef |
| `Action` | `action` | Usage of an ActionDef |
| `Allocation` | `allocate` | Maps elements between domains |
| `View` | `view` | Usage of a ViewDef or ViewpointDef |
| `Calculation` | `calculation` | Usage of a CalculationDef |
| `VerificationCase` | `verification case` | Usage of a VerificationCaseDef |
| `AnalysisCase` | `analysis case` | Usage of an AnalysisCaseDef |

## Native elements (own schema)

These are not standard SysML usages — they carry a stable opaque identifier and their own required field sets.

| Type | ID pattern | Required fields |
|---|---|---|
| `Requirement` | `REQ(-[A-Z0-9]{2,12})+-[0-9]{3}` | `id`, `title`, `status` |
| `TestCase` | `TC(-[A-Z0-9]{2,12})+-[0-9]{3}` | `id`, `title`, `status`, `testLevel`, `verifies` |
| `ADR` | `ADR(-[A-Z0-9]{2,12})+-[0-9]{3}` | `id`, `title`, `status` |
| `Configuration` | `CONF(-[A-Z0-9]{2,12})+-[0-9]{3}` | `id`, `title`, `status`, `featureModel` |

## Tier 2 — Safety & cybersecurity elements (own schema)

These types support ISO 26262 HARA and ISO/SAE 21434 TARA workflows. Each carries a stable opaque identifier and validated required fields.

| Type | ID pattern | Standard | Description |
|---|---|---|---|
| `HazardousEvent` | `HE-*` | ISO 26262 | Hazardous situation with severity / exposure / controllability |
| `SafetyGoal` | `SG-*` | ISO 26262 | Top-level safety goal derived from a HazardousEvent; carries ASIL |
| `DamageScenario` | `DS-*` | ISO/SAE 21434 | Adverse consequence of a cybersecurity compromise |
| `ThreatScenario` | `TS-*` | ISO/SAE 21434 | Attack path referencing one or more DamageScenarios |
| `CybersecurityGoal` | `CSG-*` | ISO/SAE 21434 | Security property goal derived from ThreatScenarios; carries CAL level |
| `SecurityControl` | `SC-*` | ISO/SAE 21434 | Countermeasure implementing one or more CybersecurityGoals |
| `VulnerabilityReport` | `VR-*` | — | Tracked vulnerability with CVSS score and mitigation link |

## Tier 4 — Safety analysis containers

These analysis types use one of two authoring patterns:

- **Option A (file-per-element)** — each node is its own `.md` file; the parser loads them individually.
- **Option B (exploded container)** — a single container file holds all rows in frontmatter tables; the walker synthesises a first-class element per row so all cross-reference and query machinery works without changes.

| Type | Pattern | ID pattern | Standard | Description |
|---|---|---|---|---|
| `FaultTree` | A | `FT-*` | IEC 61025 / ISO 26262-9 | Top-level fault tree; `topEvent:` links to a SafetyGoal |
| `FaultTreeGate` | A | `FTG-*` | IEC 61025 | Boolean gate (AND / OR / XOR / NOT / inhibit) with `inputs:` list |
| `FaultTreeEvent` | A | `FTE-*` | IEC 61025 | Leaf event (basic / undeveloped / house); optional `failureRate:` |
| `FMEASheet` | B | `FMEA-*` | IEC 60812 / SAE J1739 | Container; each `entries:` row becomes an `FMEAEntry` element |
| `FMEAEntry` | B | `FM-*` | IEC 60812 | Failure mode row; RPN auto-computed from severity × occurrence × detection |
| `TARASheet` | B | `TARA-*` | ISO/SAE 21434 | Container with four section tables (damage / threat / goal / control) |

`TARASheet` explodes each row into the appropriate Tier 2 type (`DamageScenario`, `ThreatScenario`, `CybersecurityGoal`, `SecurityControl`) at parse time.

See [Safety Analysis](../model-guide/safety-analysis.md) for authoring examples.

## Namespace elements

| Type | Description |
|---|---|
| `Package` | Directory namespace — usually declared in `_index.md` |
| `LibraryPackage` | Standard library namespace (e.g. `Parts`, `Interfaces`) |
| `Namespace` | Generic namespace |

## Diagram elements

| Type | Description |
|---|---|
| `Diagram` | A diagram — `diagramKind:` selects the rendering path |

See [Diagrams](diagrams.md) for the full `diagramKind` list.
