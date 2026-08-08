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

These are not standard SysML usages — they carry a stable opaque identifier and their own required field sets. They are **id-identified**: their identity is the stable `id`, and their human-readable label is **`name`** (free prose — spaces and punctuation allowed, `W042` does not apply). The `title` field is **removed**; a `title:` on any element is error `E025`, and `E024` (formerly: `name` on an id-identified type) is **retired**. See [Frontmatter → Label field](frontmatter.md#label-field-name).

| Type | ID pattern | Required fields |
|---|---|---|
| `Requirement` | `REQ(-[A-Z0-9]{2,12})*-[0-9]{3,8}` | `id`, `name`, `status` |
| `TestCase` | `TC(-[A-Z0-9]{2,12})*-[0-9]{3,8}` | `id`, `name`, `status`, `testLevel`, `verifies` |
| `TestPlan` | `TP(-[A-Z0-9]{2,12})+-[0-9]{3,8}` | `id`, `name`, `status` |
| `ADR` | `ADR(-[A-Z0-9]{2,12})*-[0-9]{3,8}` | `id`, `name`, `status` |
| `ReviewRecord` | `RR(-[A-Z0-9]{2,12})+-[0-9]{3,8}` | `id`, `name`, `status`, `reviewType`, `reviews` |
| `TradeStudy` | `TRD(-[A-Z0-9]{2,12})+-[0-9]{3,8}` | `id`, `name`, `status`, `criteria`, `alternatives`, `scores` |
| `PlanningItem` | `PI(-[A-Z0-9]{2,12})*-[0-9]{3,8}` | `id`, `name`, `status` |
| `Zone` | `ZN(-[A-Z0-9]{2,12})+-[0-9]{3,8}` | `id`, `name`, `status`, `targetSL` |
| `Conduit` | `CD(-[A-Z0-9]{2,12})+-[0-9]{3,8}` | `id`, `name`, `status`, `fromZone`, `toZone` |
| `Configuration` | `CONF(-[A-Z0-9]{2,12})+-[0-9]{3,8}` | `id`, `name`, `status`, `featureModel` |

**`Zone`** / **`Conduit`** (§13) model IEC 62443 industrial cybersecurity: a `Zone` (`ZN-*`)
groups parts under a Security Level (`targetSL`/`achievedSL`); a `Conduit` (`CD-*`) connects
two zones. Structural elements may carry `targetSL`/`achievedSL`/`inZone:`. Validation
`E950`–`E956`, `W950`–`W953`; commands `zones`, `conduits`, `zones --coverage`.

**`ReviewRecord`** (§19) captures a formal review event (design / requirements / hazard /
test-readiness review, inspection, walkthrough) and the model elements it covers — a thin,
baselined traceability anchor whose `recordedAt:` points to the external review (e.g. a
GitHub PR). Validation `E700`–`E705`, `W700`, `W704`; commands `reviews`, `review`,
`reviews --coverage`. See [CLI → Reviews](../cli/index.md#review-records-reviews).

**`TradeStudy`** (§15) records a weighted-criteria evaluation of design alternatives
(`criteria` with weight + `maximize`/`minimize`, `alternatives`, a `scores` matrix, optional
`objective`/`decision`). The tool computes — never writes — normalised/weighted scores and
rankings. Validation `E869`–`E877`, `W061`–`W064`; command `trade-study`.

**`PlanningItem`** (`ADR-SYS-PLANITEM-001`) is the model's native representation of the day-to-day
work of getting from `Requirement` to satisfied/verified — the shape a Jira epic/story/task or a
GitHub issue hierarchy fills today, made durable and structurally part of the traceability graph.
A strict **single-parent tree** (`parent:`, at most one — not a DAG); a top-level item (no
`parent:`) must set `achieves:` (one or more `Requirement`s this branch of work exists to realise,
deliberately a separate field from `satisfies:`, which stays scoped to architecture semantics).
`status` (`todo`/`in_progress`/`blocked`/`done`) and `itemType` (`bug`/`task`/`feature`) reuse
GitHub's own current vocabulary verbatim. `blockedBy:` names one or more elements it's waiting on —
resolved permissively like `evidence.ref:` below, not restricted to `PlanningItem` — with dangling
and cycle checks; a non-empty `blockedBy:` while `status` isn't `blocked` warns (likely stale), but
`status: blocked` with an empty `blockedBy:` raises nothing — being blocked needs no proof, unlike
claiming done. `evidence:` is a list of duck-typed entries — `ref:` (any
resolvable element, unrestricted by kind) or `path:` (a file/doc, resolved like `implementedBy:`) —
each with an optional `rationale:` that waives that one entry's own check. A **leaf** item (no
children) claiming `status: done` must have at least one non-waived, resolving `evidence:` entry —
graded harder than the analogous `Requirement` rule (`W300`, a warning) since claiming done with no
proof is a correctness defect, not a time-bound gap. `assignedTo:` names a single Unix-style
username — not a cross-reference (users aren't model elements) — always format-checked
(`^[a-z_][a-z0-9_-]{0,31}$`, `E723`), and additionally checked against a project-declared roster
(`[users]` in `.syscribe.toml`, mapping username → display name) only when that roster is
non-empty (`E722`); roster membership is dormant otherwise, matching every other opt-in
`.syscribe.toml`-configured table — a malformed roster key is `W309` and excluded from the
effective roster. `show` resolves and prints the declared display name alongside the username.
No dedicated CLI subcommand or MCP tool yet — queried via the generic
`list`/`show`/`ls`/`find`/`refs` commands, and gets a working guarded MCP write path for free via
the existing `create_element`/`update_element`/etc. tools. Validation `E706`–`E717`, `E719`–`E723`,
`W308`, `W309`. See `examples/planning-item/` for a complete worked example.

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
