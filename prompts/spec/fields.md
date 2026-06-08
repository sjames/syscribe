# Syscribe Frontmatter Field Reference

All frontmatter fields. Optional unless marked **required**.
`serde(rename_all = "camelCase")` — use camelCase in YAML.

## Identity and classification

| Field | Applies to | Type | Default | Notes |
|---|---|---|---|---|
| `type` | All | string | **required** | Element type from the type inventory |
| `name` | All | string | filename stem | Overrides the filename stem as display name and QName segment |
| `shortName` | All | string | absent | Abbreviated name for display |
| `qualifiedName` | All | string | derived | Auto-derived from path; set to override |
| `visibility` | All | string | `public` | `public` or `private` |
| `id` | native Req/TC/ADR/safety | string | **required** | Stable opaque ID matching type pattern |
| `title` | native Req/TC/ADR/safety | string | **required** | Human-readable title |
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
| `implementedBy` | Part/PartDef | string or list |
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

## Product Line Engineering (PLE) fields

| Field | Applies to | Type |
|---|---|---|
| `appliesWhen` | Any element (incl. TestCase), or a Package | string/list | Boolean expression over FeatureDef QNames: `and`/`or`/`not`/parentheses; a bare QName or a list (AND) also work. Element/TestCase is included only in variants where it holds. A TestCase with no `appliesWhen` runs in every Configuration. On a Package it gates the whole subtree transitively; one declaration per path (`E228`), empty gated package `W026`. |
| `featureModel` | FeatureDef/Configuration | string | QName of the system FeatureDef model root |
| `features` | Configuration | map | Feature selections: `<FeatureDef QName>: true/false` (§9.8) |
| `parameters` | FeatureDef | list | Typed parameters (§9.7): each `{name, type, range, enumValues, default, isFixed, isRequired, value}` |
| `parameterBindings` | Configuration | map | Bind feature parameters: `<FeatureDef QName>.<param>: <value>` (dotted member; validated: E203–E206, E222, W017) |
| `parameterConstraints` | Package `_index.md` | list | Cross-feature constraints `{id, expression, severity, appliesWhen}` — `expression` is a comparison over dotted refs, `appliesWhen` a boolean predicate; checked by `feature-check` (E213/W014, E221/W025) |
| `groupKind` | FeatureDef | string | child grouping: `optional` · `alternative` · `or` · `mandatory` (legacy member shorthand) |
| `mandatory` | FeatureDef | bool | membership vs parent (orthogonal to `groupKind`): `true` = selected whenever parent is / always at top level |
| `cardinality` | FeatureDef | string | For `or` groups: `"1..*"` etc. |
| `isFixed` | FeatureDef parameter | bool | Prohibits binding override |
| `isRequired` | FeatureDef parameter | bool | W010 if unbound in Configuration |
| `contributesTo` | Component FeatureDef | string | QName of system FeatureDef |
| `parameterBindings` | Configuration | map | Feature param bindings |
| `features` (PLE) | Configuration | map | `{FeatureName: true/false}` |
