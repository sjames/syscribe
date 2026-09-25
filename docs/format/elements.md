# Element Types

`FORMAT · ELEMENT TYPES`

Every `.md` file in the model tree is one element. The `type:` field in YAML frontmatter selects the element type. A value outside the type inventory below is stored as `Unknown` and raises error `E005` (unrecognised `type:`); a missing `type:` is `E001`. The authoritative list is `ElementType::ALL` in `crates/syscribe-model/src/element.rs` — every type there appears on this page.

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
| `AllocationDef` | `allocation def` | Classifies allocations |
| `ConcernDef` | `concern def` | Classifies stakeholder concerns |
| `CaseDef` | `case def` | Base classifier for analysis/verification/use cases |
| `OccurrenceDef` | `occurrence def` | Classifies things with a temporal extent |
| `EventOccurrenceDef` | `event occurrence def` | Classifies momentary occurrences |
| `IndividualDef` | `individual def` | Classifies one specific individual |
| `SuccessionDef` | `succession def` | Classifies temporal orderings |
| `RenderingDef` | `rendering def` | Classifies view renderings |

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
| `Attribute` | `attribute` | Usage of an AttributeDef |
| `Constraint` | `constraint` | Usage of a ConstraintDef |
| `State` | `state` | Usage of a StateDef |
| `ExhibitState` | `exhibit state` | Referential usage exhibiting a StateDef |
| `Flow` | `flow` | Usage of a FlowDef |
| `UseCase` | `use case` | Usage of a UseCaseDef |
| `Concern` | `concern` | Usage of a ConcernDef |
| `Case` | `case` | Usage of a CaseDef |
| `Occurrence` | `occurrence` | Usage of an OccurrenceDef |
| `EventOccurrence` | `event occurrence` | Momentary observation/signal |
| `Individual` | `individual` | Usage of an IndividualDef |
| `Succession` | `succession` | Temporal ordering between actions/occurrences |
| `BindingConnector` | `binding` | Equality binding between two features |
| `Enumeration` | `enum` | Usage of an EnumerationDef |
| `Metadata` | `metadata` | Application of a MetadataDef |
| `Rendering` | `rendering` | Usage of a RenderingDef |

`Requirement` is listed with the native elements below (the native handler owns the type).

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
| `FeatureDef` | `FEAT(-[A-Z0-9]{2,12})+` (no numeric suffix needed) | `id` (name-identified: `name` is also its qname segment) |
| `Baseline` | `BL(-[A-Z0-9]{2,12})+` (no numeric suffix needed) | `id`, `name`, `status`; `seal:` written by `baseline create` |
| `FeatureModel` | *(name-identified, no id)* | `featureTree` — a flat list exploded into `FeatureDef` elements (§9.6a) |

**`Baseline`** (`ADR-SYS-BASELINE-001`) is a sealed, commit-anchored release snapshot of a model
scope (`frozenScope:`), created by `syscribe baseline create` and re-checked on every validate:
drift is `E520` when `released`, `W520` when `approved`, silent for `draft`, skipped for
`superseded`; seal/manifest tamper is `E521`, an unresolved `supersedes:` `E522`. Commands
`baseline create`/`verify`/`diff`/`list`/`show`.

**`FeatureModel`** authors a whole feature tree in one file: each `featureTree:` entry's `name:`
is a dot-separated path relative to the sheet (`Platform.CortexM`), `id:` is optional (derived as
`FEAT-PLATFORM-CORTEXM` when omitted), and optional `crossTreeConstraints:` /
`parameterConstraints:` sit on the same sheet. Codes `E231`–`E233`, `W048`. See
[Variability](../model-guide/variability.md).

**`Zone`** / **`Conduit`** (§13) model IEC 62443 industrial cybersecurity: a `Zone` (`ZN-*`)
groups parts under a Security Level (`targetSL`/`achievedSL`, each `1`–`4`, else `E925`); a
`Conduit` (`CD-*`) connects two zones. Both take `status:` `draft`/`review`/`approved`/`deprecated`
(else `E926`). Structural elements may carry `targetSL`/`achievedSL`/`inZone:`. Validation
`E950`–`E956`, `E925`, `E926`, `W950`–`W953`; commands `zones`, `conduits`, `zones --coverage`.

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
A `done` item whose `achieves:` Requirement lacks the verification bar
`W002`/`W305` apply (an active TestCase for a leaf, an active L3–L5 TestCase for a parent) warns
`W310`; two *active* items (`in_progress`, or carrying `claimedBy:`) that share an `achieves:`
Requirement or an `evidence[].path` warn `W311` (possible duplicate work). For concurrent
(multi-agent) work, `syscribe claim <PI-id> --by <agent-id>` sets the advisory
`claimedBy:`/`claimedAt:` markers (refused on a `done` item or one claimed by someone else) and
`syscribe release <PI-id>` clears them. Everything else is queried via the generic
`list`/`show`/`ls`/`find`/`refs` commands and written via `set` or the guarded MCP
`create_element`/`update_element`/etc. tools. Validation `E706`–`E717`, `E719`–`E723`,
`W308`–`W311`. See `examples/planning-item/` for a complete worked example.

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
| `Asset` | `ASSET-*` | ISO/SAE 21434 | Protected asset with `cybersecurityProperties`; optional `assetOwner`/`relatedSafetyGoal` |
| `ConfirmationMeasure` | `CM-*` | ISO 26262-2 / ISO/SAE 21434 | Confirmation review / FS audit / FS assessment / cybersecurity assessment (`measureType`, `independenceLevel` I1–I3, `confirms:`) |
| `Argument` | `ARG-*` | GSN | Safety-case node (`argumentType: claim \| strategy \| solution`, `supports:`, `evidence:`) |
| `AssumptionOfUse` | `AOU-*` | ISO 26262 | Safety-related application condition (SRAC); `appliesTo:` goals/arguments/requirements |

All of these require `id`, `name` and `status`. Every stable id needs at least one 2–12-character
category segment before the numeric suffix — `DS-BRAKE-001`, not `DS-001`.

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
| `AttackTree` | A | `AT-*` | ISO/SAE 21434 §15.7 | Attack-path tree; requires `status` and `threatRef:` (a ThreatScenario) |
| `AttackTreeGate` | A | `ATG-*` | ISO/SAE 21434 | `gateType: AND \| OR` with `inputs:`; placed under its AttackTree's directory |
| `AttackStep` | A | `ATS-*` | ISO/SAE 21434 | Leaf attacker action with `attackFeasibility:`; placed under its AttackTree's directory |

`TARASheet` explodes each row into the appropriate Tier 2 type (`DamageScenario`, `ThreatScenario`, `CybersecurityGoal`, `SecurityControl`) at parse time.

See [Safety Analysis](../model-guide/safety-analysis.md) for authoring examples.

## Namespace elements

| Type | Description |
|---|---|
| `Package` | Directory namespace — usually declared in `_index.md` |
| `LibraryPackage` | Standard library namespace (e.g. `Parts`, `Interfaces`) |
| `Namespace` | Generic namespace |
| `Dependency` | Directed client → supplier relationship (`clients:`, `suppliers:`) |

## Diagram elements

| Type | Description |
|---|---|
| `Diagram` | A diagram — `diagramKind:` selects the rendering path |

See [Diagrams](diagrams.md) for the full `diagramKind` list.
