# Syscribe Frontmatter Field Reference

All frontmatter fields. Optional unless marked **required**.
`serde(rename_all = "camelCase")` — use camelCase in YAML.

## Identity and classification

| Field | Applies to | Type | Default | Notes |
|---|---|---|---|---|
| `type` | All | string | **required** | Element type from the type inventory |
| `name` | **All** | string | filename stem (name-identified) | The single human-readable label on **every** element type. For name-identified types (SysML structural, `Package`, `Diagram`, `FeatureDef`) it is also the QName/identity segment and must be a basic name (`W042`). For id-identified types (native Req/TC/TP/Config/ADR/safety/security) it is **required** free prose — spaces/punctuation allowed, `W042` does not apply. |
| `shortName` | All | string | absent | Abbreviated name for display |
| `qualifiedName` | All | string | derived | Auto-derived from path; set to override |
| `visibility` | All | string | `public` | `public` or `private` |
| `id` | id-identified types + `FeatureDef` | string | **required** | Stable opaque ID matching the type's pattern. **Mandatory `FEAT-*` id on `FeatureDef`** too (E201 if missing) — a feature stays name-labelled but must carry a stable id. |
| `title` | — | — | — | **REMOVED.** No longer a label field on any element; use `name`. A stray `title:` on any element is error `E025`. |
| `status` | native Req/TC/ADR/safety | string | **required** | Lifecycle status |
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
| `ports` | Port | list | Nested sub-ports |
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

| Field | Applies to | Type |
|---|---|---|
| `entryAction` | StateDef/State | string or map |
| `doAction` | StateDef/State | string or map |
| `exitAction` | StateDef/State | string or map |
| `subStates` | StateDef/State | list |
| `transitions` | StateDef/State | list |

## Constraints and expressions

| Field | Applies to | Type | Default |
|---|---|---|---|
| `expression` | ConstraintDef | string | absent |
| `expressionLanguage` | ConstraintDef | string | `"ocl"` |
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
| `verifiedBy` | Requirement | list |
| `verifies` | VerificationCase | list |
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
| `derivedFromSecurityGoal` | string | CybersecurityGoal ID/QName |
| `tags` | list | Free-form tags |

## Native TestCase extra fields

| Field | Type | Notes |
|---|---|---|
| `testLevel` | string | **required** — `L1` (doc review) · `L2` (analysis) · `L3` (unit/integration) · `L4` (system) · `L5` (HIL/physical) |
| `securityTestMethod` | string | optional (ISO/SAE 21434 §13.3) — `fuzz` · `penetration_test` · `security_regression` · `vulnerability_scan` · `threat_modeling` (W809 if other). Orthogonal to `testLevel`; lets `verification-depth`/`matrix` distinguish security-method tests from functional ones |
| `sourceFile` | string | Path relative to model root (W004 if not found) |
| `testFunctions` | list | `{function: name, scenario: "title"}` mappings |
| `tags` | list | Free-form tags |

## Allocation

| Field | Applies to | Type |
|---|---|---|
| `allocateFrom` | Allocation element | string |
| `allocateTo` | Allocation element | string |
| `allocations` | AllocationDef/Package/PartDef | list |
| `allocatedFrom` | Any element | string or list |
| `allocatedTo` | Any element | string or list |

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

## Packaging and imports

| Field | Applies to | Type |
|---|---|---|
| `imports` | Package | list |
| `aliases` | All | list |
| `filterCondition` | Package | string |
| `dependsOn` | All | list |

## Miscellaneous

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `metadata` | All | list | `{type: MetaDef::Name, field: value, ...}` |
| `rep` | All | string | SysML textual notation representation hint |
| `values` | EnumerationDef | list | **required** |
| `annotates` | MetadataDef | list | Restricts what types this metadata may annotate |
| `itemType` | FlowDef | string | QName of the item type flowing |

## Custom fields

| Field | Applies to | Type | Notes |
|---|---|---|---|
| `custom_fields` | All | map | Freeform user metadata: `string -> scalar \| list-of-scalars`. Keys are not validated. Values must be scalars or lists of scalars (nested map → `W041`). Serialised in sorted order. Read-only in UI/`show`. Queryable via `--where custom.<key>[=,=~,~=]<val>`. |

```yaml
custom_fields:
  supplier: Bosch
  partNumbers: [A-1001, A-1002]
```

## Product Line Engineering (PLE) fields

| Field | Applies to | Type |
|---|---|---|
| `appliesWhen` | Any element (incl. TestCase), or a Package | string/list | Boolean expression over FeatureDef QNames: `and`/`or`/`not`/parentheses; a bare QName or a list (AND) also work. Element/TestCase is included only in variants where it holds. A TestCase with no `appliesWhen` runs in every Configuration. On a Package it gates the whole subtree transitively; one declaration per path (`E228`), empty gated package `W026`. |
| `featureModel` | FeatureDef/Configuration | string | QName of the system FeatureDef model root |
| `features` | Configuration | map | Feature selections: `<FeatureDef QName>: true/false` (§9.8) |
| `parameters` | FeatureDef | list | Typed parameters (§9.7): each `{name, type, range, enumValues, default, isFixed, isRequired, value, buildVar}`. Optional `buildVar:` maps the parameter's bound (or default) value to a named build variable emitted by `build-config`. |
| `buildExports` | FeatureDef | list | **Optional.** Build variable declarations for `build-config`: each `{var, whenSelected, whenDeselected}`. `whenSelected` (default `1`) is emitted when the feature is selected; `whenDeselected` is emitted when not selected, or the variable is omitted when absent. Multiple entries allowed per feature. See E050/W050. |
| `parameterBindings` | Configuration | map | Bind feature parameters: `<FeatureDef QName>.<param>: <value>` (dotted member; validated: E203–E206, E222, W017) |
| `buildOverrides` | Configuration | map | **Optional.** Build variable overrides applied last by `build-config`, after `buildExports` and `parameterBindings`. Use for config-specific variables (version strings, SKU names) not tied to a feature. Wins on name collision. |
| `parameterConstraints` | Package `_index.md` | list | Cross-feature constraints `{id, expression, severity, appliesWhen}` — `expression` is a comparison over dotted refs, `appliesWhen` a boolean predicate; checked by `feature-check` (E213/W014, E221/W025) |
| `groupKind` | FeatureDef | string | child grouping: `optional` · `alternative` · `or` · `mandatory` (legacy member shorthand) |
| `mandatory` | FeatureDef | bool | membership vs parent (orthogonal to `groupKind`): `true` = selected whenever parent is / always at top level |
| `cardinality` | FeatureDef | string | For `or` groups: `"1..*"` etc. |
| `isFixed` | FeatureDef parameter | bool | Prohibits binding override |
| `isRequired` | FeatureDef parameter | bool | W010 if unbound in Configuration |
| `contributesTo` | Component FeatureDef | string | QName of system FeatureDef |
| `parameterBindings` | Configuration | map | Feature param bindings |
| `features` (PLE) | Configuration | map | `{FeatureName: true/false}` |

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
| `derivedFromSecurityGoal` | Requirement | string | `CSG-*` that generated this requirement |
| `derivedFromSafetyGoal` | Requirement | string | `SG-*` that generated this requirement |
