# Syscribe Frontmatter Field Reference

The frontmatter fields the parser recognises. Optional unless marked **required**.
`serde(rename_all = "camelCase")` — use camelCase in YAML. Any other top-level key is kept
but warned `W047` (unknown key) — put project-specific data under `custom_fields:`, computed
values under `derive:`, and project-declared relationships under `links:` instead. Reverse
indices (`verifiedBy`, `derivedChildren`, `children`, …) are
computed by the tool and never authored.

## Identity and classification

| Field | Applies to | Type | Default | Notes |
|---|---|---|---|---|
| `type` | All | string | **required** | Element type from the type inventory |
| `name` | **All** | string | filename stem (name-identified) | The single human-readable label on **every** element type. For name-identified types (SysML structural, `Package`, `Diagram`, `FeatureDef`) it is only a label: the QName segment is the file/directory stem (which `W042` checks as a basic name), and a `name:` differing from the stem is `W058`. For id-identified types (native Req/TC/TP/Config/ADR/safety/security) it is **required** free prose — spaces/punctuation allowed, `W042` does not apply. |
| `shortName` | All | string | absent | Abbreviated name for display |
| `qualifiedName` | All | string | absent | Never an identity override — the qname is always path-derived. Only meaningful on a §3.10 locale variant (with `locale:`), where it names the documented element (`E026` if unresolved); elsewhere a differing value is `W049` and ignored |
| `visibility` | All | string | `public` | `public` or `private` |
| `id` | id-identified types + `FeatureDef` | string | **required** | Stable opaque ID matching the type's pattern. **Mandatory `FEAT-*` id on `FeatureDef`** too (E201 if missing) — a feature stays name-labelled but must carry a stable id. |
| `title` | — | — | — | **REMOVED.** No longer a label field on any element; use `name`. A stray `title:` on any element is error `E025`. |
| `status` | id-identified types (Req/TC/TP/ADR/Config/Baseline/PlanningItem/RR/TRD/Zone/Conduit/most safety & security types) | string | **required** | Lifecycle status; vocabulary is per type (`syscribe template <Type>` shows it) |
| `extRef` | All | string or list | absent | External reference(s) — this element represents an artifact in another tool (DNG, a SysML tool). Opaque (URI or `tool:id`). Look up with `extref <ref>`; duplicate across elements warns `W028`. Not a model cross-ref target. |

## Classification flags

| Field | Applies to | Type | Default |
|---|---|---|---|
| `isAbstract` | All | bool | `false` |
| `isVariation` | Def/Usage | bool | `false` |
| `isVariant` | Usage | bool | `false` |
| `isIndividual` | Occurrence | bool | `false` |
| `isReadonly` | Usage | bool | `false` |
| `isDerived` | Usage | bool | `false` |
| `isEnd` | Usage | bool | `false` |
| `isPortion` | Occurrence usage | bool | `false` |
| `isReference` | Usage | bool | `false` |
| `isComposite` | Usage | bool | `true` |
| `isConstant` | Usage | bool | `false` |
| `isOrdered` | Usage | bool | `false` |
| `isNonunique` | Usage | bool | `false` |
| `isConjugated` | Port | bool | `false` |
| `isParallel` | StateDef/State | bool | `false` |
| `isAsserted` | Constraint | bool | `false` |
| `isNegated` | Constraint | bool | `false` |
| `isSemantic` | MetadataDef | bool | `false` |
| `isDeploymentPackage` | PartDef/Part | bool | `false` |
| `range` | AttributeDef | string | absent | Numeric interval of the type's values: `lo..hi`, `[lo, hi)`, … (GH #366) |
| `pattern` | AttributeDef | string | absent | Regular expression a string value must match in full (GH #366) |
| `default` | AttributeDef | any | absent | Value taken when none is given; must satisfy `range`/`pattern` (GH #366) |
| `ram`, `flash`, `nvm`, `stack` | PartDef/Part | size | absent | Static memory a software part uses; `stack` counts toward RAM (GH #368) |
| `ramCapacity`, `flashCapacity`, `nvmCapacity` | PartDef/Part | size | absent | Memory a hardware part offers (GH #368) |
| `period`, `deadline` | PartDef/Part | duration | absent | Timing of a software part, e.g. `10 ms`; with `wcet` it is a task (GH #368) |
| `priority` | PartDef/Part | integer | absent | Scheduling priority of a software part (GH #368) |
| `cpuMargin` | PartDef/Part | percent | absent | Percentage of the schedulability bound a hardware part's tasks may use; opts in to the CPU check (GH #368) |
| `scheduling` | PartDef/Part | `edf`\|`rate-monotonic` | `edf` | Schedulability bound of a hardware part (GH #368) |
| `margin` | PartDef/Part | percent | `[budgets] margin`, else 100 | Percentage of capacity that may be used (GH #368) |
| `kind` | ArchitectureRule | string | **required** | `forbid-dependency`, `independence`, `allow-only`, `layering` or `max-depth` (GH #369) |
| `layers` | ArchitectureRule | list | required for `layering` | The layers, top first |
| `maxDepth` | ArchitectureRule | integer | required for `max-depth` | Longest allowed dependency chain |
| `waivers` | ArchitectureRule | list | none | `{element, rationale, expires}`; suppresses findings on `element` until `expires` (YYYY-MM-DD) |
| `layer` | PartDef/Part | string | absent | The architecture layer of the part; also a selector key (GH #369) |
| `level` | PartDef/Part | enum | absent | Design level of a software part: `component` or `unit`; a non-draft unit must be traced to code, a requirement, an L1 test and a component (GH #371) |
| `deploymentKind` | PartDef/Part | enum | absent | Makes the part a deployment node: `soc`, `core`, `partition`, `container`, `process`, `thread` (GH #370) |
| `replicated` | PartDef/Part | bool | false | A software part deliberately deployed on several nodes (GH #370) |
| `from` | ArchitectureRule | map | required except for `layering`, `max-depth` | Selector of the components the rule starts from: `package` (qname prefix), `tag`, `domain`, `stereotype`, ANDed |
| `to` | ArchitectureRule | map | required for `forbid-dependency`, `independence`, `allow-only` | Selector of the components the rule is about (same keys as `from`) |
| `via` | ArchitectureRule | list | all three | Edge sources that count: `connections`, `typedBy`, `allocations` |
| `profile` | Package | bool | `false` | `true` marks a stereotype profile; enabled by `[metadata] profiles` (`W316`) |
| `version` | Package | string | absent | The version of a stereotype profile |
| `checks` | MetadataDef | list | absent | Expressions run on every carrier: a string, or `{expr, message, severity: error\|warning}` (`E650`/`W650`) |
| `autoApply` | Package (profile) | list | absent | Rules `{select: {type, where}, apply: {type, key: value}}` deriving stereotype applications |

## Typing and specialization

| Field | Applies to | Type | Default |
|---|---|---|---|
| `supertype` | Def | string or list | absent |
| `typedBy` | Usage | string or list | absent |
| `subsets` | Usage | list | absent |
| `redefines` | Usage | list | absent |
| `conjugates` | PortDef | string | absent |
| `variantOf` | Part/Usage | string | absent |

## Structure

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `multiplicity` | Usage | string | Quoted: `"1"`, `"0..*"`, `"0..1"`, `"1..*"` |
| `direction` | Port, Parameter | string | `in` · `out` · `inout` |
| `features` | Def/Usage | list | Inline attribute/port/sub-element declarations |
| `connections` | PartDef/Part | list | `{from: a.p, to: b.q}` port bindings |
| `flowConnections` | PartDef/Part | list | Flow connection bindings |
| `successionConnections` | ActionDef/Action | list | Temporal ordering bindings |
| `bindingConnections` | Def/Usage | list | Equality bindings |
| `performs` | PartDef/Part | list | Action usages performed by this part |
| `exhibitsStates` | PartDef/Part | list | State machines exhibited by this part |
| `ends` | ConnDef/IntfDef | list | Connection end declarations |
| `timeSlices` | OccurrenceDef | list | Time slices |
| `snapshots` | OccurrenceDef | list | Snapshots |

## Behavior and calculation

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `parameters` | ActionDef/CalcDef/etc. | list | Parameter declarations |
| `returnType` | CalculationDef/VerificationCaseDef | string | Return type QName |
| `body` | CalculationDef/ActionDef | string | Expression body (opaque) |
| `bodyLanguage` | CalculationDef/ActionDef | string | absent |
| `subActions` | ActionDef/Action/CaseDef | list | Owned sub-actions |
| `controlNodes` | ActionDef/Action | list | Fork/join/decision/merge nodes |

## State machines

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `entryAction` | StateDef/State | string or map | Behaviour on entry |
| `doAction` | StateDef/State | string or map | Ongoing behaviour |
| `exitAction` | StateDef/State | string or map | Behaviour on exit |
| `isParallel` | StateDef/State | bool | Parallel (orthogonal) region container |
| `variables` | StateDef/State | list | Executable machines: `{name, value, typedBy?, unit?}` data of the machine (`E883`) |
| `initialEffect` | StateDef/State | map | Executable machines: `{assign}` run by the initial transition (`E884`) |
| `subStates` | StateDef/State | list | Nested states; each may itself carry `transitions`/`subStates` |
| `transitions` | StateDef/State | list | Each entry: `source` · `target` · `accept` (event/payload) · `guard`. The deprecated spellings `from`/`to`/`trigger` still parse but warn **W075** — prefer `source`/`target`/`accept`. |

## Executable behaviour, plants and architecture links

Opt-in; nothing changes for a model that does not use them. Narrative and examples: `syscribe spec behaviour`, `spec evidence`.

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `models` | Part/PartDef | list | Architecture elements (`PartDef`/`Part` with a `domain:`) this executable part is a behavioural model of (`E990` unresolved or not architecture) |
| `portMap` | Part/PartDef, or a `features:` usage | map | Executable port → architecture port; `null` exempts a port (`E991`) |
| `abstracts` | Part/PartDef | list | What the model leaves out: a port name (exempts `W990`) or a requirement id (the model cannot be evidence for it, `W992`) |
| `fidelity` | Part/PartDef | string | Prose on how faithful the model is |
| `role` | part usage (`features:` entry) | string | `environment` (verification set-up, exempt from architecture correspondence) or `external` (an actor outside the product in a context diagram); `E992` otherwise, never on a port |
| `flowKind` | PortDef, port feature, `connections:` entry | string | What flows (power, signal, command, …); styles diagram edges (spec §8.16.9) |
| `plant` | Part/PartDef | string or map | The part is a plant: `plant: <PlantDef>` or `{def: <PlantDef>, parameters: {…}}` or `{library: <name>, parameters: {…}}`; exclusive with `fmu:` (`E930`) |
| `fmu` | Part/PartDef | map | An FMU as the plant: `{file, fmiVersion, kind, sha256, parameters}`; checked only in a build with the `fmi` feature (`I011` otherwise) |
| `plants` | TestCase | list | The parts that are plants in this scenario run |
| `plantStep` | TestCase | string | Communication step with a unit, e.g. `"10 ms"` |
| `plantConnections` | TestCase | list | Wiring `Plant.variable` ↔ machine variable |

`type: PlantDef` fields: `parameters` (`{name, value}`), `inputs` (`{name, unit, value}`), `outputs` (`{name, unit}`), `states` (`{name, value}`; the initial value fixes the dimension), `equations` (`der(state) = expr`, `output := expr`), `method` (`rk4` default, or `euler`), `maxStep`.

Derived-diagram view fields (`type: Diagram`, spec §8.16.10): `include` (a map of `kinds`, `domains`, `tags`, `features`, `depth`, `members`), `style` (a `[diagram.style.<name>]` table of `.syscribe.toml`), `layout` (pins, plus `flow` and `layers` hints), `depth` (composition levels followed).

## Constraints and expressions

| Field | Applies to | Type | Default |
|---|---|---|---|
| `expression` | ConstraintDef | string | absent |
| `requires` | All | list | absent |
| `assume` | All | list | absent |

## Requirements and cases

| Field | Applies to | Type |
|---|---|---|
| `subject` | Req/Case | string |
| `actors` | Req/UseCase | list |
| `stakeholders` | Req/Viewpoint | list |
| `concerns` | Req/Viewpoint | list |
| `framedConcerns` | RequirementDef | list |
| `derivedFrom` | RequirementDef/Requirement | list |
| `satisfies` | Part/PartDef/etc. | list |
| `implementedBy` | Part/PartDef/Interface/InterfaceDef | string or list |
| `verifies` | TestCase / VerificationCase | list |
| `verifiesElement` | TestCase | list | `PartDef`/`InterfaceDef`/`ActionDef`/`StateDef` verified directly (qname or id); E102 unresolved, E129 wrong kind |
| `verdictExpression` | VerificationCase | string |
| `verdictType` | VerificationCaseDef | string |
| `objectives` | CaseDef | list |
| `result` | CaseDef | string |
| `includes` | UseCaseDef | list |
| `extends` | UseCaseDef | list |
| `extensionPoints` | UseCaseDef | list |

## Native Requirement extra fields

| Field | Type | Notes |
|---|---|---|
| `reqDomain` | string | `system` · `hardware` · `software` |
| `silLevel` | integer | 1–4 (IEC 61508); mutually exclusive with `asilLevel` (W006) |
| `asilLevel` | string | `QM`, `A`–`D` (ISO 26262; `E010` lists them; notation `B(D)` accepted); mutually exclusive with `silLevel` (W006) |
| `plLevel` | string | `a`–`e` (ISO 13849-1) |
| `verificationMethod` | string | `test` · `inspection` · `analysis` · `demonstration` |
| `wcet` | string | Worst-case execution time budget |
| `breakdownAdr` | string | ADR ID/QName for decomposition rationale (required when `derivedFrom` set) |
| `derivedFromSafetyGoal` | string | SafetyGoal ID/QName |
| `derivedFromCybersecurityGoal` | string | CybersecurityGoal ID/QName |
| `reqClass` | string | `stakeholder` · `system` · `derived` — position in the stakeholder/system decomposition (informational, not validated) |
| `requirementKind` | string | `stakeholder` · `system` · `software` · `hardware` · `process` · `regulatory` · `deliverable` (`E022` if other); the last three are not architecture-allocatable (no `W300`/`W302`) |
| `dalLevel` | string | `A`–`E` (DO-178C, `E019`); with `asilLevel` warns `W703` |
| `decompositionKind` | string | ASIL/SIL decomposition argument: `independent` · `redundant` · `diverse` (informational) |
| `decomposedFrom` | string | Original ASIL (A–D) before decomposition; the notation `asilLevel: B(D)` sets it implicitly. `asilLevel` also accepts `QM` (E878, E879) |
| `tags` | list | Free-form tags |

## Native TestCase extra fields

| Field | Type | Notes |
|---|---|---|
| `testLevel` | string | **required** — `L1` (unit, hosted) · `L2` (property-based/fuzz, hosted) · `L3` (formal proof) · `L4` (integration, emulator) · `L5` (hardware-in-the-loop). Tool semantics: `L1`/`L2` are leaf-level; `L3`–`L5` count as integration-level for a parent requirement's `W305`; `L5` is what `asilLevel: D` (`W702`) and a `wcet:` budget (`W029`) need |
| `securityTestMethod` | string | optional (ISO/SAE 21434 §13.3) — `fuzz` · `penetration_test` · `security_regression` · `vulnerability_scan` · `threat_modeling` (W809 if other). Orthogonal to `testLevel`; lets `verification-depth`/`matrix` distinguish security-method tests from functional ones |
| `sourceFile` | string | Path relative to model root (W004 if not found) |
| `testFunctions` | list | Each entry a function-name string or `{function: name, scenario: "title"}` |
| `coversVariability` | list | Late-bound features (FEAT id / qname) and parameters (`<feature>.<param>`) whose declared range or values this test exercises; clears the late-bound safety rule `W753` for a requirement it verifies; an entry naming nothing is `W754` (§9.8) |
| `coverageTarget` | string | `statement` · `branch` · `MCDC` (`E021`) |
| `tags` | list | Free-form tags |

## Native TestPlan fields

| Field | Type | Notes |
|---|---|---|
| `scope` | string | `unit`·`smoke`·`integration`·`hil`·`certification`·`security`·`regression` (`W610` if other) |
| `testCases` | string or list | Explicit member TestCases (id/qname) |
| `selection` | map | Additive query `{testLevels, domains, tags}` unioned with `testCases` |
| `configurations` | list | `CONF-*` variants this plan targets; absent = configuration-agnostic |
| `demonstrates` | list | Goals/requirements this plan is evidence for |

## ADR fields (§8.17)

| Field | Type | Notes |
|---|---|---|
| `date` | string | ISO-8601 date the decision was made |
| `deciders` | string or list | Decision-makers: stakeholder `PartDef` qnames or free-text names. Display metadata, not resolved; shown by `show`. ADR-only (`W047` on any other type) |
| `supersedes` | string or list | The `ADR`(s) this one replaces (`E320` unresolved/non-ADR, `E321` cycle, `W313` target not `superseded`, `W314` a `breakdownAdr:` still cites a superseded ADR). `supersededBy` is computed and shown by `show`/`links` |

## PlanningItem fields (§23)

| Field | Type | Notes |
|---|---|---|
| `parent` | string | At most one other `PlanningItem` (strict tree; cycle `E712`) |
| `achieves` | string or list | What this work realises — a `Requirement`, or a `SafetyGoal`/`CybersecurityGoal`/`ADR`/`Argument`/`TestPlan`/`Baseline` — **required** on a top-level item (`E713`–`E715`; `W315` when `done` but the target is still draft/review/proposed) |
| `itemType` | string | `bug` · `task` · `feature` (`E709`) |
| `blockedBy` | string or list | Any elements it waits on (`E720` dangling, `E721` cycle, `W308` if set while not `blocked`) |
| `assignedTo` | string | Unix-style username (`E723`); checked against `[users]` when that roster is non-empty (`E722`) |
| `evidence` | list | `{ref: <element>}` or `{path: <file>}`, each with optional waiving `rationale:`; a leaf `done` item needs one valid entry (`E719`) |
| `claimedBy` / `claimedAt` | string | Advisory ownership — set by `syscribe claim <PI> --by <agent>`, cleared by `syscribe release <PI>`; never hand-edit |

`status`: `todo` · `in_progress` · `blocked` · `done` (`E708`). `W310`: `done` but the achieved
requirement lacks the W002/W305 verification bar. `W311`: two active items overlap.

## Record fields (Baseline, ReviewRecord, TradeStudy)

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `date` / `approver` / `gitTag` / `gitCommit` | Baseline | string | `gitCommit` captured by `baseline create`; `gitTag` is distinct from the `BL-*` id |
| `frozenScope` | Baseline | map | `{package, config, closureFrom, types, status, tags}`; omit for the whole model |
| `seal` | Baseline | map | `{aggregateHash, elementCount, manifest}` — generated, never hand-edit (`E520`/`E521`) |
| `supersedes` | Baseline | string or list | Earlier `BL-*` (`E522` if unresolved) |
| `reviewType` | ReviewRecord | string | **required** — `design_review`·`requirements_review`·`hazard_review`·`test_readiness_review`·`inspection`·`walk_through` |
| `reviews` | ReviewRecord | list | **required** — elements covered by the review |
| `reviewDate` / `reviewedBy` / `recordedAt` | ReviewRecord | string / list / string | `recordedAt` points to the external review (e.g. a PR URL) |
| `items` | ReviewRecord | list | Action items `{id, description, disposition, closedBy}` |
| `criteria` | TradeStudy | list | **required** — `{name, weight, direction: maximize\|minimize, unit}` |
| `alternatives` | TradeStudy | list | **required** — `{name, element?}` |
| `scores` | TradeStudy | list | **required** — score matrix (alternative × criterion) |
| `objective` / `decision` | TradeStudy | string | Requirement informed / ADR recording the choice |

## IEC 62443 zones and conduits (§13)

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `targetSL` / `achievedSL` | Zone, Conduit, structural elements | int | Security Level `1`–`4` (`targetSL` **required** on Zone) |
| `members` | Zone | list | Parts/PartDefs in the zone |
| `rationale` | Zone | string | Why this SL |
| `fromZone` / `toZone` | Conduit | string | **required** — the two zones connected |
| `protocols` | Conduit | list | Protocols carried |
| `inZone` | PartDef/Part | string | Zone this element belongs to |

## Allocation

| Field | Applies to | Type |
|---|---|---|
| `allocations` | AllocationDef/Package/PartDef | list |
| `allocatedFrom` | `Allocation` element (with `allocatedTo`); on any other element only as a legacy input — it is the derived reverse of `allocatedTo` (§12.9) | string or list |
| `allocatedTo` | Any element — the source being allocated (§12.9 form 1), or an `Allocation` element | string or list |

## Domain and domain-independence

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `domain` | PartDef/Part/etc. | string | `system` · `hardware` · `software` |
| `reqDomain` | native Requirement | string | `system` · `hardware` · `software` |

## Views and rendering

| Field | Applies to | Type |
|---|---|---|
| `expose` | ViewDef | list |
| `rendering` | ViewDef | string |
| `satisfiedBy` | ViewpointDef | list |
| `methods` | ViewpointDef | list |

## Diagrams (`type: Diagram`)

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `diagramKind` | Diagram | string | `BDD` · `IBD` · `StateMachine` · `Sequence` · `Requirement` · `FeatureModel` · `FaultTree` · `AttackTree` · `SafetyCase` · `Traceability` · `ZoneConduit` · `ThreatGraph` · `Mermaid` · `PlantUML` |
| `subject` | Diagram | string | QName of the element the diagram depicts (W401 if unresolved) |
| `pumlMode` | Diagram | string | Only value: `companion` (E403 otherwise). Generates a `.puml` via `syscribe plantuml`, rendered to SVG by `syscribe plantuml render`. Requires `diagramKind` (E404) and an `<img>` tag in the body (W413); `.puml` must exist (W414). |
| `pumlFile` | Diagram | string | Path to the `.puml` companion source |
| `svgMode` | Diagram | string | `companion` (composed-SVG workflow) · `inline` (embedded SVG in body) |
| `svgFile` | Diagram | string | Path to a pre-rendered SVG companion |
| `shapes` | Diagram | map | Shape-id → `{ref, kind, parent}` (shape `ref:` warns W402 if unresolved) |
| `edges` | Diagram | map | Edge-id → `{source, target, kind}` (source/target warn W403 if not a shape-id) |

## Packaging and imports

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `imports` | Package | list | Import declarations |
| `aliases` | All | list | Alias declarations |
| `filterCondition` | Package, View, ViewDef | string | KerML opaque package filter |
| `dependsOn` | All | list | Dependency edges |
| `sysmlSubmodel` | Package `_index.md` | bool | Ingest the package's `.sysml` files as a native SysMLv2 submodel |
| `foreignFormat` | Package `_index.md` | string | Hand the package subtree to the stdio plugin `[plugins.<alias>]` in `.syscribe.toml` |
| `annotationFormat` / `marker` / `include` / `exclude` | Package `_index.md` | string / regex / globs / globs | Ingest elements from marker comment blocks in source files; mutually exclusive with `foreignFormat`/`sysmlSubmodel` (`W562`) |
| `repoImports` | Package `_index.md` | list | Multi-repo composition (§14, opt-in): each `{repo, qname, as}` mounts a peer-repo subtree. `repo` is an alias from `[repos]` in `.syscribe.toml` (E513), `qname` the element/package in that repo (E514), `as` the local mount name. Inert unless `[repos]` is configured. |

## Miscellaneous

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `metadata` | All | list | `{type: MetaDef::Name, field: value, ...}` |
| `about` | All | string or list | Qualified names or stable ids the body annotates: the file is a §3.10 `about:` comment — its body is attached as a note to every listed element and the file is not an element (`E027` unresolved entry; `W052` any field besides `type`/`name`/`about`/`locale`, a malformed entry, or `about:` on an `_index.md`) |
| `locale` | All | string | BCP 47 tag of the Markdown body. With `qualifiedName:` the file is a §3.10 locale variant: its body is attached to that element as `locale` documentation and the file is not an element (`E026` unresolved target; `W051` duplicate locale, differing `type:`, or any field besides `type`/`name`/`locale`/`qualifiedName`) |
| `rep` | All | string | SysML textual notation representation hint |
| `values` | EnumerationDef | list | **required** |
| `annotates` | MetadataDef | list | Restricts what types this metadata may annotate |
| `itemType` | FlowDef | string | QName of the item type flowing |
| `responsibility` | All work products | string | Accountable party/organisation (ISO 26262-8 §5 DIA/CIA split); drives W038. Opt-in. |
| `testExempt` | Any architecture element | string | Non-empty rationale suppressing W104 (no direct test). |
| `ffiRationale` | PartDef/Part/etc. | string | Freedom-from-interference argument for mixed-criticality on a shared resource; suppresses W034. Opt-in. |
| `traceBaselines` | Any link source | map | `<target id>: blake3:<hex>` — suspect-link baselines written by `suspect accept`; a changed target warns `W090`. Never hand-edit. |

## Custom fields

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `derive` | All | map | Computed fields: `fieldName -> formula string` (§3.18). Evaluated in dependency order; shown under Derived Fields by `show`. Cycle → `E504`; bad formula / non-mapping block / non-string formula → `E505`; unknown `elements["Q"]` → `E506`. Never `W047`. |
| `custom_fields` | All | map | Freeform user metadata: `string -> scalar \| list-of-scalars`. Keys are not validated. Values must be scalars or lists of scalars (nested map → `W041`). Serialised in sorted order. Read-only in UI/`show`. Queryable via `--where custom.<key>[=,=~,~=]<val>`. |

```yaml
custom_fields:
  supplier: Bosch
  partNumbers: [A-1001, A-1002]
```

## User-defined links (ADR-SYS-LINKTYPE-001)

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `links` | All | map | Instances of project-declared link types: `<linkType>: <ref> \| [<ref>, ...]`. Each key must be a link type declared in `[linkTypes.<name>]` of `.syscribe.toml` (`E630`, which lists the declared types); refs resolve by id or qname like `satisfies:` (`E632`). The element holding the entry is the source (§12.1). Discover the vocabulary with `link-types` / MCP `link_types` — never invent a type. |

```yaml
links:
  mitigates: [REQ-HAZ-001]
  conflictsWith: REQ-SYS-014
```

## Product Line Engineering (PLE) fields

| Field | Applies to | Type |
|---|---|---|
| `appliesWhen` | Any element (incl. TestCase), or a Package | string/list | Boolean expression over FeatureDef QNames: `and`/`or`/`not`/parentheses; a bare QName or a list (AND) also work. Element/TestCase is included only in variants where it holds. A TestCase with no `appliesWhen` runs in every Configuration. On a Package it gates the whole subtree transitively; one declaration per path (`E228`), empty gated package `W026`. |
| `featureModel` | FeatureDef/Configuration | string | QName of the system FeatureDef model root |
| `features` | Configuration | map | Feature selections: `<FeatureDef QName>: true/false` (§9.8) |
| `subConfigurations` | Configuration | string/list | **Optional** (§14.7, `ADR-SYS-HPLE-001`). One or more other `Configuration`s this one consolidates — reachable locally or via `[repos]` (peer-native qname, stable id, or `repoImports:` mount path), at any depth. Each entry must resolve to a real, internally-valid `Configuration` (`E516` dangling, `E517` wrong-type, `E518` not internally valid). A leaf tier with no lower tiers to consolidate simply omits it. |
| `parameters` | FeatureDef | list | Typed parameters (§9.7): each `{name, type, range, enumValues, default, isFixed, isRequired, value, buildVar}`. Optional `buildVar:` maps the parameter's bound (or default) value to a named build variable emitted by `build-config`. |
| `buildExports` | FeatureDef | list | **Optional.** Build variable declarations for `build-config`: each `{var, whenSelected, whenDeselected}`. `whenSelected` (default `1`) is emitted when the feature is selected; `whenDeselected` is emitted when not selected, or the variable is omitted when absent. Multiple entries allowed per feature. See E050/W050. |
| `bindingStage` | Configuration | string | the binding-time stage this Configuration binds at — a stage of `binding-times`; absent = unstaged. Unknown is `E751`; binding outside an item's window is `E752`; earlier than the `derivedFrom` base is `E753` (§9.8) |
| `parameterNarrowing` | Configuration | map | `<FeatureDef QName>.<param>` → a narrower `range` string (`"lo..hi"`) or `enumValues` list than the parameter (or an ancestor Configuration) allows; widening is `E755`, a binding outside it `E756` (§9.8) |
| `parameterBindings` | Configuration | map | Bind feature parameters: `<FeatureDef QName>.<param>: <value>` (dotted member; validated: E203–E206, E222, W017). A dotted key also resolves transitively through `subConfigurations:` at any depth, using the parameter's ordinary qname (peer-native or through a `repoImports:` mount path) — no new syntax; the cross-tier legality checks are `E519`/`E523`, and the opt-in completeness warning for what's still left open anywhere in the subtree is `W513` (§14.7). |
| `buildOverrides` | Configuration | map | **Optional.** Build variable overrides applied last by `build-config`, after `buildExports` and `parameterBindings`. Use for config-specific variables (version strings, SKU names) not tied to a feature. Wins on name collision. |
| `parameterConstraints` | Package `_index.md` | list | Cross-feature constraints `{id, expression, severity, appliesWhen}` — `expression` is a comparison over dotted refs, `appliesWhen` a boolean predicate; checked by `feature-check` (E213/W014, E221/W025) |
| `groupKind` | FeatureDef | string | child grouping: `optional` · `alternative` · `or` · `mandatory` (legacy member shorthand) |
| `mandatory` | FeatureDef | bool | membership vs parent (orthogonal to `groupKind`): `true` = selected whenever parent is / always at top level |
| `bindingTime` | FeatureDef | string | earliest stage at which the feature's selection may be bound — a stage of `binding-times` (built-in `compile` · `load` · `runtime`; unknown is `E230`); with `latestBindingTime` the *binding window* (§9.7). Parameters carry the same two keys inside their `parameters:` entry |
| `latestBindingTime` | FeatureDef | string | latest stage at which the selection may still be bound; earlier than `bindingTime` is `E750`; alone it means "no later than this" |
| `cardinality` | FeatureDef | string | For `or` groups: `"1..*"` etc. |
| `isFixed` | FeatureDef parameter | bool | Prohibits binding override |
| `isRequired` | FeatureDef parameter | bool | `W017` if unbound in a Configuration |
| `contributesTo` | Component FeatureDef | string | QName of system FeatureDef |
| `featureTree` | FeatureModel | list | Flat list of feature entries; `name:` is a dotted path relative to the sheet, `id:` optional (derived) (§9.6a; `E231`/`E232`) |
| `crossTreeConstraints` | FeatureModel | list | `{feature, requires, excludes}` edges kept in one section (`E233`) |

## Safety analysis fields (ISO 26262 / IEC 61508 / ISO 13849)

Full narrative + rules: `syscribe spec safety`. Integrity levels (`asilLevel` A–D, `silLevel` 1–4, `plLevel` a–e) also apply to `SafetyGoal` and propagate down the trace (`E841`–`E843`, `W808`).

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `severity` | HazardousEvent | string | ISO 26262 `S0`–`S3` |
| `exposure` | HazardousEvent | string | ISO 26262 `E0`–`E4` |
| `controllability` | HazardousEvent | string | ISO 26262 `C0`–`C3` |
| `operationalSituation` | HazardousEvent | string | Operating scenario (free text) |
| `consequence` | HazardousEvent | string | IEC 61508 risk graph `Ca`–`Cd` (alt. to S/E/C) |
| `freqExposure` | HazardousEvent | string | IEC 61508 risk graph `Fa`/`Fb` |
| `avoidance` | HazardousEvent | string | IEC 61508 risk graph `Pa`/`Pb` |
| `demandRate` | HazardousEvent | string | IEC 61508 risk graph `W1`–`W3` |
| `safeState` | SafetyGoal | string | Description of the safe state |
| `ftti` | SafetyGoal | string | Fault-tolerant time interval, number + unit (`ns` `us` `ms` `s` `min` `h`), e.g. `"20ms"` (`E880` otherwise) |
| `hazardousEvents` | SafetyGoal | list | `HazardousEvent` id/QName refs |
| `topEvent` | FaultTree | string | `SafetyGoal` ref (the top event) |
| `missionTime` | FaultTree | string | e.g. `"1e9 h"` |
| `gateType` | FaultTreeGate | string | `AND`·`OR`·`XOR`·`NOT`·`inhibit` |
| `inputs` | FaultTreeGate | list | Input gate/event refs |
| `eventKind` | FaultTreeEvent | string | `basic`·`undeveloped`·`house` |
| `failureRate` | FaultTreeEvent | float | Failure rate /h (≥ 0, E982) |
| `ccfGroup` | FaultTreeEvent | string | Common-cause group name (beta-factor CCF) |
| `ccfBeta` | FaultTreeEvent | float | Beta factor `0.0`–`1.0` (E984) |
| `probability` | FaultTree/Gate/Event | float | Cut-set or top-event probability |
| `entries` | FMEASheet | list | Inline `FMEAEntry` rows |
| `failureMode` | FMEAEntry | string | What fails |
| `effect` | FMEAEntry | string | Consequence |
| `cause` | FMEAEntry | string | Root cause |
| `fmeaSeverity` | FMEAEntry | int | 1–10 |
| `occurrence` | FMEAEntry | int | 1–10 |
| `detection` | FMEAEntry | int | 1–10 |
| `rpn` | FMEAEntry | int | Risk priority number (S×O×D) |
| `recommendedAction` | FMEAEntry | string | Mitigation |
| `fmeaRef` / `ftaRef` | FaultTreeEvent / FMEAEntry | string | FTA ⇄ FMEA reconciliation link |
| `diagnosticCoverage` / `latentDiagnosticCoverage` | architecture elements | float | DC / DCl, `0.0`–`1.0` |
| `measureType` | ConfirmationMeasure | string | `confirmation_review`·`functional_safety_audit`·`functional_safety_assessment`·`cybersecurity_assessment` |
| `independenceLevel` | ConfirmationMeasure | string | `I1`·`I2`·`I3` |
| `confirms` | ConfirmationMeasure | list | Work products confirmed: `SafetyGoal`, `CybersecurityGoal`, `HazardousEvent`, `Requirement`, `FaultTree`, `FMEASheet`, `TARASheet`, `ADR`, `Argument`, `TestPlan`, `Allocation` (`E851` unresolved, `E860` other type; confirm a sheet, not its exploded rows) |
| `argumentType` | Argument | string | `claim`·`strategy`·`solution` |
| `supports` | Argument | string or list | SafetyGoal or parent Argument argued for |
| `evidence` | Argument | list | Requirement/TestCase/Argument/AssumptionOfUse refs (strings) |
| `appliesTo` | AssumptionOfUse | list | SafetyGoal/Argument/Requirement the SRAC constrains |
| `covers` / `reactionTime` | SafetyMechanism | list / duration | Failure modes covered; reaction time checked against the goal FTTI (W894) |
| `quantities` | any | list | Timing values `{kind: ftti\|latency\|wcet\|reaction, value, unit: s\|ms\|us\|ns}`; budgets checked (W893) |
| `runsOn` | TestCase / TestPlan | list | TestEnvironment(s) the test executes on |
| `requiresCapabilities` | TestCase / TestPlan | list | Capabilities needed from the environment |
| `capabilities` / `environmentKind` / `calibrationStatus` / `calibrationDue` | TestEnvironment | list / enum / enum / date | What the rig offers and its calibration |
| `analyses` | DependentFailureAnalysis | list (≥2) | Elements argued independent |
| `sharedResources` | DependentFailureAnalysis | list | `{resource, kind, initiators, couplingFactor, mitigation}` |

## Security analysis fields (ISO/SAE 21434)

Full narrative + rules: `syscribe spec safety`.

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `damageTable` / `threatTable` / `goalTable` / `controlTable` | TARASheet | list | Row tables exploded into `DamageScenario`/`ThreatScenario`/`CybersecurityGoal`/`SecurityControl` |
| `damageSeverity` | DamageScenario | string | `severe`·`major`·`moderate`·`negligible` |
| `impactCategories` | DamageScenario | list | `safety`·`financial`·`operational`·`privacy` |
| `attackFeasibility` | ThreatScenario | string | `high`·`medium`·`low`·`very_low` |
| `attackVector` | ThreatScenario | string | `network`·`adjacent`·`local`·`physical` |
| `damageScenarios` | ThreatScenario | list | `DamageScenario` id/QName refs |
| `calLevel` | CybersecurityGoal | string | `CAL1`–`CAL4` |
| `securityProperty` | CybersecurityGoal | string | `confidentiality`·`integrity`·`availability`·`authenticity` |
| `threatScenarios` | CybersecurityGoal | list | `ThreatScenario` id/QName refs |
| `controlType` | SecurityControl | string | `prevention`·`detection`·`response`·`recovery` |
| `implementsGoals` | SecurityControl | list | `CybersecurityGoal` id/QName refs |
| `cvssScore` | VulnerabilityReport | float | 0.0–10.0 (`E824` if out of range) |
| `cveId` | VulnerabilityReport | string | `CVE-YYYY-NNNNN` |
| `affectedElements` | VulnerabilityReport | list | QNames of affected model elements |
| `mitigatedBy` | VulnerabilityReport | list | `SecurityControl` id/QName refs |
| `assets` / `hazardRef` | DamageScenario (/ThreatScenario) | list | `Asset` refs; `HazardousEvent`/`SafetyGoal` co-engineering link (`E844`) |
| `riskTreatment` / `residualRisk` | ThreatScenario | string | `avoid`·`reduce`·`share`·`retain` (`E845`) / free text |
| `cybersecurityProperties` / `assetOwner` / `relatedSafetyGoal` | Asset | list / string / string | Protected properties; owning element; SafetyGoal co-link |
| `threatRef` | AttackTree | string | **required** — the ThreatScenario the tree substantiates |
| `gateType` / `inputs` | AttackTreeGate | string / list | `AND`·`OR`; child gate/step refs |
| `attackFeasibility` | AttackStep | string | `high`·`medium`·`low`·`very_low` |
| `safetyImpact` / `financialImpact` / `operationalImpact` / `privacyImpact` | DamageScenario | string | Optional per-category ratings, `severe`·`major`·`moderate`·`negligible` (`E640`); overall impact = max of these and `damageSeverity` |
| `elapsedTime` / `expertise` / `knowledge` / `windowOfOpportunity` / `equipment` | ThreatScenario, AttackStep | string or int | Optional attack-potential factors (GH #222): a label from the factor's table (see `[cyber.attack_potential]`) or a non-negative integer of points. All five together derive `attackFeasibility` when it is not declared; a declared value wins (`W642` on mismatch); a partial set is ignored (`W641`); an unknown label is `E641` |
| `securityTestMethod` | TestCase | string | See Native TestCase fields |

## Additional fields (indexed from the full specification)

Fields defined in the full specification (`spec/markdown-sysml-format.md`) that are not listed in the sections above. "spec §N" names the section of the full specification that defines the field; Appendix A of that document indexes every field.

| Field | Applies to | Notes |
|---|---|---|
| `approvedBy` | spec §12.14 | who accepted it, a username (checked against `[users]` when declared); required when `accepted` |
| `assetTable` | TARASheet | list of row mappings, each exploded into an `Asset` element (`id`, `name`, `status`, `assetOwner`, …) |
| `baselineRef` | spec Appendix A | Opaque external baseline reference (e.g., a Git tag, CM baseline ID). |
| `campaignHash` | spec §12.14 | the composite hash `inject` printed when the decision was taken (at least 12 hex digits) |
| `clients` | spec §2.4 | Qualified names of the dependent (client) elements |
| `constraints` | spec §8.3.3 | Constraint usages asserted across this interface's ends (e.g., conservation laws) |
| `cvssSeverity` | VulnerabilityReport | declared severity bucket: `none` · `low` · `medium` · `high` · `critical`; a value outside the set, or one that disagrees with the bucket of `cvssScore`, is `W978` |
| `cvssVector` | VulnerabilityReport | CVSS vector string, e.g. `CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H`; a malformed one is `W978` |
| `depth` | spec §8.16.10 | As the diagram's `depth:`, which wins when both are set |
| `displayOrder` | spec §3.16, §8.11.6 | Primary sort key among peers. Ascending: lower sorts first. Unset sorts **after** every element that declares a value. Ties (equal or both-unset) fall back to stable-identifier order. / Reading/presentation order among peer requirements in the report and coverage matrix; ascending, unset last, id tie-break. Generic field — see §3.16. |
| `equations` | PlantDef | `der(state) = expression` for each state and `output := expression` for each output |
| `evaluate` | spec §22.2 | Qualified name of a `ConstraintDef` whose `expression:` bounds the result. When present, the tool evaluates the `body:` expression and compares against the constraint. |
| `excludes` | FeatureDef | qualified names of `FeatureDef` elements that must NOT be selected when this feature is (mutual exclusion) |
| `expires` | ResidualRisk | date by which the decision must be reviewed again; a proposed or accepted risk past it is `W943` |
| `faultSet` | spec §12.14 | the FMEA rows (`FMEAEntry`) and fault-tree events (`FaultTreeEvent`) whose joint failure the decision is about, by id or qualified name. It names a set exactly as `inject` reports it: `[A, B]` explains the set `A + B`, not `A` alone |
| `fixedIn` | VulnerabilityReport | version that fixes the vulnerability; a `mitigated`/`resolved`/`fixed`/`closed` report needs this or `mitigatedBy` (`W976`) |
| `goal` | ResidualRisk | the `SafetyGoal`(s) the defeated invariant protects (spec §12.14) |
| `groupBy` | derived IBD Diagram | `domain` frames the parts by their `domain:` (GH #336); `stereotype` frames them by a `frame = true` stereotype (GH #378) |
| `injection` | FMEAEntry row, FaultTreeEvent | injectable fault model for `inject`: a list of mappings with a `kind` (`stuck`, `offset`, `silent`, `drop`, `delay`, …); malformed is `E936` |
| `layout` | spec §8.16.2 | Shape id → `{x, y, w, h}` pixel coordinates (an edge id → `{points: [[x, y], …]}` pins its routing). Entries are **pins** (`REQ-TRS-VIS-007`): a pinned shape keeps its position while the browser lays the unpinned rest out around it; `w`/`h` are optional. A key naming no shape or edge is `W416`; an entry without numeric `x`/`y` is `E405`. Two keys are *hints*, not shape ids: `flow` (`left-to-right` / `top-to-bottom`) and `layers` (bands in order); see §8.16.10. |
| `maxStep` | PlantDef | longest integration sub-step; each communication step is split evenly below it |
| `method` | PlantDef | integration method: `rk4` (default) or `euler`, fixed step |
| `operations` | PortDef, InterfaceDef, ItemDef | callable operations (synchronous) and receptions (asynchronous) declared on a definition (spec §8.3.4) |
| `outputs` | PlantDef | list of `{name, unit}`; read by composite bindings (at least one) |
| `parentFeature` | spec Appendix A | Qualified name of the parent `FeatureDef`. Normally inferred from the directory hierarchy; set explicitly only when the file cannot be placed at the canonical path. |
| `portionKind` | spec §3.2 | Required when `isPortion: true`; `timeslice` (contiguous temporal extent) or `snapshot` (zero-duration instantaneous point) |
| `rateSource` | FaultTreeEvent, FMEAEntry | where the `failureRate` comes from (GH #343) |
| `rateStatus` | FaultTreeEvent, FMEAEntry | provenance of the rate: `verified` · `secondary` · `unverified` (`E986`) |
| `ref` | Diagram shape or edge entry | qualified name of the model element the shape or edge represents (spec §8.16.3–8.16.4) |
| `refines` | UseCaseDef, UseCase, ActionDef, StateDef (and their usages) | `Requirement`(s) this element refines; an operand that is not a requirement is `E316` (spec §12.10.4) |
| `states` | PlantDef | list of `{name, value}`; the initial value fixes the dimension |
| `steps` | ActionDef, UseCase | ordered step names; not walked by the `W080` sequence-completeness rule |
| `style` | spec §8.16.2 | A named style, `[diagram.style.<name>]` of `.syscribe.toml` (§8.16.10); one that names no style, or `style:` on an element that is not a `Diagram`, is `W421`. |
| `suppliers` | spec §2.4 | Qualified names of the elements depended upon (suppliers) |
| `target` | spec §3.7.1, §8.7.6 | Qualified name, optionally suffixed `::*` or `::**` / Feature chain identifying the occurrence whose feature is being assigned (e.g., `self`, `engine`) |
| `text` | `metadata:` entry | free text of an applied metadata entry, e.g. a `ModelingMetadata::Rationale` |
| `unit` | spec §3.6.1, §8.3.4 | Shorthand: qualified name of the unit for quantity attributes / Unit (for scalar quantities) |
| `value` | spec §3.6.1, §8.5.2 | Literal default value or expression body (opaque string) / Bound expression (when specializing a non-enum AttributeDef) |
| `valueKind` | spec §3.6.1, §8.5.2 | Binding semantics: `bound` (`=`), `initial` (`:=`), `default-bound` (`default =`), `default-initial` (`default :=`) / Binding semantics (default: `bound`); see §3.6.1 |
| `viewpoint` | View | qualified name of the `ViewpointDef` the view conforms to (`W500` if it does not resolve) |

## `.syscribe.toml` — project configuration reference

Everything below lives in **one file**, `<model_root>/.syscribe.toml`, never in model frontmatter.
Every table is **opt-in**: absent means built-in defaults, and a model with no `.syscribe.toml` at
all behaves identically to one with an empty file. Malformed individual entries are reported (a
warning, naming the file) and excluded rather than failing the whole table — the same posture
`W046` (`[ids.prefixes]`) and `W309` (`[users]`) both follow.

**Reading the `Required` column:** every table itself is optional — "Required" only means *once
you declare that table (or that entry within a table-of-tables), this field must be present too*.
Nothing in this file is ever required just to have a valid `.syscribe.toml`; an empty file (or no
file at all) is always legal.

| Table / Key | Field | Required | Type | Default | Notes |
|---|---|---|---|---|---|
| *(top-level)* | `repo_root` (alias `repoRoot`) | No | string | auto-detected (walks up for `.git`) | Git repo root; `repo:`-prefixed `sourceFile:`/`implementedBy:`/evidence `path:` values resolve against it. |
| `[ids]` | `max_digits` (alias `maxDigits`) | No | int | `8` | Max digits in a stable-ID numeric suffix (min `3`), §11, REQ-TRS-ID-005. |
| `[ids]` | `check_file_names` (alias `checkFileNames`) | No | bool | `false` | Enable `W065`: an id-identified element whose file stem differs from its `id`, GH #185. |
| `[ids.prefixes]` | `<TypeName> = [<prefix>, ...]` | No | map → list of strings | `{}` | Extra stable-ID prefixes per element type, additive to the built-in (`REQ`/`TC`/`ADR`/…). Each prefix must match `^[A-Z][A-Z0-9]{1,11}$`; a malformed prefix or unknown type key is `W046` and ignored. REQ-TRS-ID-007. |
| `[repos.<alias>]` | `path` | **Yes**, once the alias is declared | string | — | Peer repo root, relative to `.syscribe.toml`. §14. |
| `[repos.<alias>]` | `root` | No | string | `"model/"` | Path within the peer repo where its Syscribe model root lives. |
| `[repos.<alias>]` | `ref` | No | string | unset — pins nothing (`W510`) | Git tag/branch/SHA to pin via `repos sync`. Drives `repoImports:`, `subConfigurations:` (§14.7), and cross-repo `verifies:`/`derivedFrom:`/etc. resolution. |
| `[links]` | `base_url` (alias `baseUrl`) | No — but see note | string | unset | Hosted-source URL template, simple form: `<base_url>/<path>`. REQ-TRS-LINK-001. |
| `[links]` | `url_template` (alias `urlTemplate`) | No — but see note | string | unset | Escape-hatch template: `{path}`/`{qname}`/`{id}`/`{ref}` placeholders. **One of `base_url`/`url_template` must be set for `[links]` to do anything** — declaring the table with neither leaves the feature inert. |
| `[links]` | `ref` | No | string | `""` | Substituted for `{ref}` in `url_template`. |
| `[scripts]` | `path` | No | string | `.syscribe/scripts/` | Rhai extension-scripts directory, relative to the model root. REQ-TRS-SCRIPT-001. |
| `[plantuml]` | `theme` | No | string | unset | `!theme <name>` emitted into generated `.puml` files. |
| `[plantuml]` | `style_file` | No | string | unset | `!include <path>`; takes precedence over `theme`. REQ-TRS-PUML-040. |
| `[plantuml]` | `base_url` | No | string | `http://localhost:3000` | Base URL for clickable element links; `""` suppresses links. |
| `[plantuml]` | `jar` | No | string | unset | Path to a PlantUML `.jar` for `plantuml render`. REQ-TRS-PUML-051. |
| `[baselines]` | `element_dir` (alias `elementDir`) | No | string | `model/Baselines` | Output dir for the sealed `type: Baseline` element. REQ-TRS-BL-010. |
| `[baselines]` | `manifest_dir` (alias `manifestDir`) | No | string | `<git-root>/baselines` | Output dir for the JSON manifest. |
| `[users]` | `<username> = "<display name>"` | No | map | `{}` | Roster for `PlanningItem.assignedTo:` (§23.7). A key not matching the Unix-style username shape `^[a-z_][a-z0-9_-]{0,31}$` is `W309` and excluded from the roster. REQ-TRS-PLANITEM-008. |
| `[linkTypes.<name>]` | `description` | No | string | unset | Prose shown by `link-types` and in `--agent-instructions`. `<name>` must be lowerCamel `^[a-z][A-Za-z0-9]*$` and not a built-in link/reverse-index/edge name. Keys may be camelCase or snake_case. A malformed entry is `W630` and ignored as a whole; an unknown key is `W630` (entry kept). REQ-TRS-LINKTYPE-001. |
| `[linkTypes.<name>]` | `inverse` | No | string | unset | Reverse-direction name (lowerCamel) — used by `follow`, `links`, `impact`; must not collide with a built-in name, another type, or another inverse. |
| `[linkTypes.<name>]` | `sourceTypes` / `targetTypes` (`source_types`/`target_types`) | No | list of strings | any type | Element types allowed to hold / be the target of an instance (`E633`/`E634`); each must be a known `type:` name. |
| `[linkTypes.<name>]` | `cardinality` | No | string | `"0..*"` | Targets per source: `N`, `N..M`, `N..*`. Over the upper bound `E635`; under a non-zero lower bound on a non-draft element of a `sourceTypes` type `W631` (a non-zero lower bound requires `sourceTypes`). |
| `[linkTypes.<name>]` | `acyclic` | No | bool | `false` | Reject cycles (self-links included) formed by this type (`E636`). |
| `[linkTypes.<name>]` | `suspect` | No | bool | `true` | Participate in suspect-link baselining (`traceBaselines:`/`W090`/`suspect`). |
| `[linkTypes.<name>]` | `extends` | No | string | unset | `satisfies`·`verifies`·`derivedFrom`·`refines` — instances also count as the base link for its rules and reverse index. |
| `[linkTypes.<name>]` | `relax` | No (needs `extends`) | list of strings | `[]` | Base codes not raised for this type's instances: satisfies→`E312`,`E313`; verifies→`E104`; derivedFrom→`E105`,`E310`,`W303`; refines→`E316`. |
| `[linkTypes.<name>]` | `coverage` | No (needs `extends`) | bool | `true` | `false` keeps the base checks but withholds instances from the reverse index (no coverage credit, target not a parent). |
| `[cyber]` | `method` | No | string | `"simple"` | ISO/SAE 21434 risk/CAL method: `simple` (today's rank-sum, `score = impact + feasibility`, risk→CAL 1:1) or `annex` (built-in EXAMPLE tables modelled on the standard's informative annexes — not normative, verify against your copy). GH #222. |
| `[cyber.risk_matrix.<impact>]` | `<feasibility> = <level or int>` | No | map | base method's cell | Override a risk-matrix cell. `<impact>` ∈ `negligible·moderate·major·severe`; `<feasibility>` ∈ `very_low·low·medium·high`; value is `low·medium·high·critical` or an integer risk value resolved through `[cyber.risk_levels]`. Merges over the base method; bad entry `W640`. |
| `[cyber.risk_levels]` | `"<n>" = <level>` | No | map | `1,2→low 3→medium 4→high 5→critical` | Map an integer risk value to a level (drives W031). |
| `[cyber.cal_table.<impact>]` | `<vector> = "CALn"` | No | map | `annex`: example table; `simple`: none | Override impact × `attackVector` (`network·adjacent·local·physical`) → CAL cell. A threat with no vector, or a cell absent, falls back to `cal_by_risk`. |
| `[cyber.cal_by_risk]` | `<level> = "CALn"` | No | map | `low=CAL1 … critical=CAL4` | Risk-level → expected CAL (W032). |
| `[cyber.attack_potential.<factor>]` | `<label> = <points>` | No | map | example tables | Replace a factor table: `elapsed_time`·`expertise`·`knowledge`·`window_of_opportunity`·`equipment` (camelCase also accepted). |
| `[cyber.attack_potential.thresholds]` | `high_max` / `medium_max` / `low_max` | No | int | `13` / `19` / `24` | Points sum ≤ `high_max` → `high`; ≤ `medium_max` → `medium`; ≤ `low_max` → `low`; above → `very_low`. Must be strictly increasing (else `W640`). |
| `[profiles.<name>]` | `promote` | No | list of strings | `[]` | Warning codes this profile promotes to a gate failure. |
| `[profiles.<name>]` | `sil` / `status` / `tag` | No | string | unset (unscoped — promotes everywhere) | Optional scope filters; an entry with none of these applies to every element. |
| `[profiles.<name>]` | `magicgrid` | No | bool | `false` | Runs the gated MagicGrid validation pass under `--profile <name>`. |
| `[matchers]` | `<extension> = [<regex>, ...]` | No | map → list of strings | built-ins for Rust/Java/C/C++/Kotlin/shell | Per-extension function-definition patterns for `W009`; an override **replaces** the built-in list for that extension, not merges with it. |
| `[remote]` | `download` | No — but see note | string | unset | `sh -c` command template (`{url}`/`{dest}` placeholders) to fetch a remote `sourceFile:`. **Required for `[remote]` to do anything** — a table with no `download` loads as inert. Only runs under the explicit `validate --fetch-remote` flag — configuring it alone never executes anything. |
| `[remote]` | `cache_dir` (alias `cacheDir`) | No | string | `.syscribe/cache` | Where fetched remote sources are cached, relative to the model root. |

```toml
# .syscribe.toml — everything is optional; this shows every table at once
repo_root = "."

[ids]
max_digits = 6

[ids.prefixes]
Requirement = ["STK", "SYS"]

[repos]
avionics = { path = "../avionics-model", root = "model/", ref = "v2.1.0" }

[links]
base_url = "https://github.com/acme/model/blob/main"

[scripts]
path = ".syscribe/scripts"

[plantuml]
theme = "plain"
base_url = "https://model.internal:3000"

[baselines]
element_dir = "model/Baselines"
manifest_dir = "baselines"

[users]
alice = "Alice Nakamura"
bob = "Bob Patel"

[profiles.ci]
promote = ["W015", "W300"]
status = "approved"

[matchers]
rs = ["fn\\s+(\\w+)\\s*\\("]

[remote]
download = "curl -sSfL {url} -o {dest}"
cache_dir = ".syscribe/cache"
```
