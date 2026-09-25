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
| `name` | **All** | string | filename stem (name-identified) | The single human-readable label on **every** element type. For name-identified types (SysML structural, `Package`, `Diagram`, `FeatureDef`) it is also the QName/identity segment and must be a basic name (`W042`). For id-identified types (native Req/TC/TP/Config/ADR/safety/security) it is **required** free prose — spaces/punctuation allowed, `W042` does not apply. |
| `shortName` | All | string | absent | Abbreviated name for display |
| `qualifiedName` | All | string | derived | Always derived from the file path; an authored value is parsed but never overrides the path-derived qname |
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
| `bodyLanguage` | CalculationDef/ActionDef | string | `"ocl"` (default) |
| `subActions` | ActionDef/Action/CaseDef | list | Owned sub-actions |
| `controlNodes` | ActionDef/Action | list | Fork/join/decision/merge nodes |

## State machines

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `entryAction` | StateDef/State | string or map | Behaviour on entry |
| `doAction` | StateDef/State | string or map | Ongoing behaviour |
| `exitAction` | StateDef/State | string or map | Behaviour on exit |
| `isParallel` | StateDef/State | bool | Parallel (orthogonal) region container |
| `subStates` | StateDef/State | list | Nested states; each may itself carry `transitions`/`subStates` |
| `transitions` | StateDef/State | list | Each entry: `source` · `target` · `accept` (event/payload) · `guard`. The deprecated spellings `from`/`to`/`trigger` still parse but warn **W075** — prefer `source`/`target`/`accept`. |

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
| `asilLevel` | string | `A`–`D` (ISO 26262); mutually exclusive with `silLevel` (W006) |
| `plLevel` | string | `a`–`e` (ISO 13849-1) |
| `verificationMethod` | string | `test` · `inspection` · `analysis` · `demonstration` |
| `wcet` | string | Worst-case execution time budget |
| `breakdownAdr` | string | ADR ID/QName for decomposition rationale (required when `derivedFrom` set) |
| `derivedFromSafetyGoal` | string | SafetyGoal ID/QName |
| `derivedFromCybersecurityGoal` | string | CybersecurityGoal ID/QName |
| `reqClass` | string | `stakeholder` · `system` · `derived` — position in the stakeholder/system decomposition (informational, not validated) |
| `requirementKind` | string | `stakeholder` · `system` · `software` · `hardware` (`E022` if other) |
| `dalLevel` | string | `A`–`E` (DO-178C, `E019`); with `asilLevel` warns `W703` |
| `decompositionKind` | string | ASIL/SIL decomposition argument: `independent` · `redundant` · `diverse` (informational) |
| `tags` | list | Free-form tags |

## Native TestCase extra fields

| Field | Type | Notes |
|---|---|---|
| `testLevel` | string | **required** — `L1` (doc review) · `L2` (analysis) · `L3` (unit/integration) · `L4` (system) · `L5` (HIL/physical) |
| `securityTestMethod` | string | optional (ISO/SAE 21434 §13.3) — `fuzz` · `penetration_test` · `security_regression` · `vulnerability_scan` · `threat_modeling` (W809 if other). Orthogonal to `testLevel`; lets `verification-depth`/`matrix` distinguish security-method tests from functional ones |
| `sourceFile` | string | Path relative to model root (W004 if not found) |
| `testFunctions` | list | Each entry a function-name string or `{function: name, scenario: "title"}` |
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

## PlanningItem fields (§23)

| Field | Type | Notes |
|---|---|---|
| `parent` | string | At most one other `PlanningItem` (strict tree; cycle `E712`) |
| `achieves` | string or list | `Requirement`s this work realises — **required** on a top-level item (`E713`–`E715`) |
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
| `supersedes` | Baseline | string | Earlier `BL-*` (`E522` if unresolved) |
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
| `diagramKind` | Diagram | string | `BDD` · `IBD` · `StateMachine` · `Sequence` · `Requirement` · `Mermaid` · `PlantUML` |
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
| `filterCondition` | Package | string | KerML opaque package filter |
| `dependsOn` | All | list | Dependency edges |
| `sysmlSubmodel` | Package `_index.md` | bool | Ingest the package's `.sysml` files as a native SysMLv2 submodel |
| `foreignFormat` | Package `_index.md` | string | Hand the package subtree to the stdio plugin `[plugins.<alias>]` in `.syscribe.toml` |
| `annotationFormat` / `marker` / `include` / `exclude` | Package `_index.md` | string / regex / globs / globs | Ingest elements from marker comment blocks in source files; mutually exclusive with `foreignFormat`/`sysmlSubmodel` (`W562`) |
| `repoImports` | Package `_index.md` | list | Multi-repo composition (§14, opt-in): each `{repo, qname, as}` mounts a peer-repo subtree. `repo` is an alias from `[repos]` in `.syscribe.toml` (E513), `qname` the element/package in that repo (E514), `as` the local mount name. Inert unless `[repos]` is configured. |

## Miscellaneous

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `metadata` | All | list | `{type: MetaDef::Name, field: value, ...}` |
| `rep` | All | string | SysML textual notation representation hint |
| `values` | EnumerationDef | list | **required** |
| `annotates` | MetadataDef | list | Restricts what types this metadata may annotate |
| `itemType` | FlowDef | string | QName of the item type flowing |
| `responsibility` | All work products | string | Accountable party/organisation (ISO 26262-8 §5 DIA/CIA split); drives W038. Opt-in. |
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
| `subConfigurations` | Configuration | string/list | **Optional** (§14.7, `ADR-SYS-HPLE-001`). One or more other `Configuration`s this one consolidates — reachable locally or via `[repos]`, at any depth. Each entry must resolve to a real, internally-valid `Configuration` (`E516` dangling, `E517` wrong-type, `E518` not internally valid). A leaf tier with no lower tiers to consolidate simply omits it. |
| `parameters` | FeatureDef | list | Typed parameters (§9.7): each `{name, type, range, enumValues, default, isFixed, isRequired, value, buildVar}`. Optional `buildVar:` maps the parameter's bound (or default) value to a named build variable emitted by `build-config`. |
| `buildExports` | FeatureDef | list | **Optional.** Build variable declarations for `build-config`: each `{var, whenSelected, whenDeselected}`. `whenSelected` (default `1`) is emitted when the feature is selected; `whenDeselected` is emitted when not selected, or the variable is omitted when absent. Multiple entries allowed per feature. See E050/W050. |
| `parameterBindings` | Configuration | map | Bind feature parameters: `<FeatureDef QName>.<param>: <value>` (dotted member; validated: E203–E206, E222, W017). A dotted key also resolves transitively through `subConfigurations:` at any depth, using the parameter's ordinary qname — no new syntax; the cross-tier legality checks are `E519`/`E523`, and the opt-in completeness warning for what's still left open anywhere in the subtree is `W513` (§14.7). |
| `buildOverrides` | Configuration | map | **Optional.** Build variable overrides applied last by `build-config`, after `buildExports` and `parameterBindings`. Use for config-specific variables (version strings, SKU names) not tied to a feature. Wins on name collision. |
| `parameterConstraints` | Package `_index.md` | list | Cross-feature constraints `{id, expression, severity, appliesWhen}` — `expression` is a comparison over dotted refs, `appliesWhen` a boolean predicate; checked by `feature-check` (E213/W014, E221/W025) |
| `groupKind` | FeatureDef | string | child grouping: `optional` · `alternative` · `or` · `mandatory` (legacy member shorthand) |
| `mandatory` | FeatureDef | bool | membership vs parent (orthogonal to `groupKind`): `true` = selected whenever parent is / always at top level |
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
| `ftti` | SafetyGoal | string | Fault-tolerant time interval, e.g. `"20ms"` |
| `hazardousEvents` | SafetyGoal | list | `HazardousEvent` id/QName refs |
| `topEvent` | FaultTree | string | `SafetyGoal` ref (the top event) |
| `missionTime` | FaultTree | string | e.g. `"1e9 h"` |
| `gateType` | FaultTreeGate | string | `AND`·`OR`·`XOR`·`NOT`·`inhibit` |
| `inputs` | FaultTreeGate | list | Input gate/event refs |
| `eventKind` | FaultTreeEvent | string | `basic`·`undeveloped`·`house` |
| `failureRate` | FaultTreeEvent | float | Failure rate /h |
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
| `confirms` | ConfirmationMeasure | list | Work products confirmed (any element) |
| `argumentType` | Argument | string | `claim`·`strategy`·`solution` |
| `supports` | Argument | string or list | SafetyGoal or parent Argument argued for |
| `evidence` | Argument | list | Requirement/TestCase/Argument/AssumptionOfUse refs (strings) |
| `appliesTo` | AssumptionOfUse | list | SafetyGoal/Argument/Requirement the SRAC constrains |

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
| `securityTestMethod` | TestCase | string | See Native TestCase fields |

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
