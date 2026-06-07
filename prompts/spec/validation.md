# Syscribe Validation Rule Codes

## Parse-time errors — core (E001–E015, E300–E304)

| Code | Condition |
|---|---|
| `E001` | File does not begin with `---` (missing frontmatter delimiter) |
| `E002` | YAML frontmatter is not valid YAML 1.2 |
| `E003` | Frontmatter contains an unrecognised key (strict mode; lenient mode emits W007) |
| `E004` | A required field is absent |
| `E005` | `type:` value is not in the element type inventory |
| `E006` | `id:` present but does not match the required pattern for the element type |
| `E007` | `status:` value is not in the allowed enum for the element type |
| `E008` | `testLevel:` is not in `L1`–`L5` |
| `E009` | `silLevel:` is not an integer in 1–4 |
| `E010` | `asilLevel:` is not in `A`–`D` |
| `E011` | Native `TestCase` body has no ` ```gherkin ` fenced block |
| `E012` | Native `Requirement` body has no normative text before the first `##` heading |
| `E013` | `verifies:` list is present but empty |
| `E014` | `Scenario Outline:` block has no `Examples:` table |
| `E015` | First Gherkin block has no `Feature:` line |
| `E300` | `ADR.id` does not match `ADR-*` pattern |
| `E301` | `ADR` missing `id`, `title`, or `status` |
| `E302` | `reqDomain:` is not `system`, `hardware`, or `software` |
| `E303` | `domain:` is not `system`, `hardware`, or `software` |
| `E304` | `ADR.status` not in `proposed · accepted · deprecated · superseded` |

## Model-time errors — core (E101–E106, E310–E315)

| Code | Condition |
|---|---|
| `E101` | Two elements have the same `id:` value |
| `E102` | `verifies:` reference cannot be resolved |
| `E103` | `derivedFrom:` reference cannot be resolved |
| `E104` | `verifies:` resolves to something that is not a native `Requirement` |
| `E105` | `derivedFrom:` resolves to something that is not a native `Requirement` |
| `E106` | `testFunctions[].scenario` does not match any Gherkin scenario title in this file |
| `E310` | `Requirement` has `derivedFrom:` but no `breakdownAdr:` |
| `E311` | `breakdownAdr:` cannot be resolved or resolves to a non-`ADR` element |
| `E312` | A parent `Requirement` (has `derivedChildren`) appears in a `satisfies:` list |
| `E313` | `satisfies:` connects an architecture element and a requirement with incompatible `domain`/`reqDomain` |
| `E314` | `PartDef`/`Part` with `isDeploymentPackage: true` has no `Allocation` to a `hardware` element |
| `E315` | `domain: software` element has `supertype:`/`typedBy:` referencing `domain: hardware`, or vice versa |

## Warnings — core (W001–W007, W300–W305)

| Code | Condition |
|---|---|
| `W001` | Native `Requirement` normative text contains no `shall` |
| `W002` | `Requirement` at `approved`/`implemented` has no active `TestCase` in `verifiedBy` |
| `W003` | `Requirement` at `verified` but `verifiedBy` is empty or all entries are `retired` |
| `W004` | `sourceFile:` path does not exist on disk relative to model root |
| `W005` | Native `Requirement` has neither `derivedFrom:` nor `derivedChildren` (possible orphan) |
| `W006` | Both `silLevel:` and `asilLevel:` set on the same element — incompatible standards |
| `W007` | Unrecognised frontmatter key (lenient mode; key preserved) |
| `W300` | Leaf `Requirement` at `approved`/`implemented` has no satisfying architecture element |
| `W301` | Leaf `Requirement` satisfied by more than one architecture element |
| `W302` | Leaf `Requirement` at `implemented`/`verified` still has `reqDomain: system` |
| `W303` | `breakdownAdr:` references an ADR with `status: proposed` |
| `W304` | `isDeploymentPackage: true` combined with `domain: hardware` |
| `W305` | Parent `Requirement` at `approved`/`implemented`/`verified` has no active `TestCase` at `testLevel: L3`–`L5` |

## Safety / ASPICE warnings (W701–W703)

| Code | Condition |
|---|---|
| `W701` | `Requirement` with `asilLevel: B`, `C`, or `D` has no `verificationMethod` |
| `W702` | `Requirement` with `asilLevel: D` has no active `TestCase` at `testLevel: L5` (HIL) |
| `W703` | Both `asilLevel:` (ISO 26262) and `dalLevel:` (DO-178C) set on the same element |

## Integrity level propagation (E841–E843, W808)

Once an element in the traceability chain has `asilLevel`, `silLevel`, or `plLevel`, all
downstream elements reached via `derivedFromSafetyGoal:`, `derivedFrom:`, or `satisfies:`
must also carry the same field. A lower level is allowed only with `breakdownAdr:`.

| Code | Condition |
|---|---|
| `E841` | `derivedFromSafetyGoal:` element missing integrity level when `SafetyGoal` has one |
| `E842` | `derivedFrom:` element missing integrity level when parent `Requirement` has one |
| `E843` | `satisfies:` element missing integrity level when the satisfied `Requirement` has one |
| `W808` | Element's integrity level is lower than its source but no `breakdownAdr:` is set |

Level ranking: `asilLevel` A < B < C < D; `silLevel` 1 < 2 < 3 < 4.

## Tier 2 parse-time errors — HARA (E800–E806, E833–E837)

| Code | Condition |
|---|---|
| `E800` | `HazardousEvent` missing `id`, `title`, or `status` |
| `E801` | `severity` not in `S0 · S1 · S2 · S3` |
| `E802` | `exposure` not in `E0 · E1 · E2 · E3 · E4` |
| `E803` | `controllability` not in `C0 · C1 · C2 · C3` |
| `E804` | `HazardousEvent.id` does not match `HE-*` |
| `E805` | `SafetyGoal` missing `id`, `title`, or `status` |
| `E806` | `SafetyGoal.id` does not match `SG-*` |
| `E833` | `consequence` not in `Ca · Cb · Cc · Cd` (IEC 61508) |
| `E834` | `freqExposure` not in `Fa · Fb` |
| `E835` | `avoidance` not in `Pa · Pb` |
| `E836` | `demandRate` not in `W1 · W2 · W3` |
| `E837` | `plLevel` not in `a · b · c · d · e` |

## Tier 2 parse-time errors — TARA (E807–E824)

| Code | Condition |
|---|---|
| `E807` | `DamageScenario` missing `id`, `title`, or `status` |
| `E808` | `DamageScenario.id` does not match `DS-*` |
| `E809` | `damageSeverity` not in `severe · major · moderate · negligible` |
| `E810` | `impactCategories` entry not in `safety · financial · operational · privacy` |
| `E811` | `ThreatScenario` missing `id`, `title`, or `status` |
| `E812` | `ThreatScenario.id` does not match `TS-*` |
| `E813` | `attackFeasibility` not in `high · medium · low · very_low` |
| `E814` | `attackVector` not in `network · adjacent · local · physical` |
| `E815` | `CybersecurityGoal` missing `id`, `title`, or `status` |
| `E816` | `CybersecurityGoal.id` does not match `CSG-*` |
| `E817` | `securityProperty` not in `confidentiality · integrity · availability · authenticity` |
| `E818` | `calLevel` not in `CAL1 · CAL2 · CAL3 · CAL4` |
| `E819` | `SecurityControl` missing `id`, `title`, or `status` |
| `E820` | `SecurityControl.id` does not match `SC-*` |
| `E821` | `controlType` not in `prevention · detection · response · recovery` |
| `E822` | `VulnerabilityReport` missing `id`, `title`, or `status` |
| `E823` | `VulnerabilityReport.id` does not match `VR-*` |
| `E824` | `cvssScore` outside 0.0–10.0 |

## Tier 2 cross-reference errors (E825–E832)

| Code | Condition |
|---|---|
| `E825` | `SafetyGoal.hazardousEvents` entry does not resolve to a `HazardousEvent` |
| `E826` | `ThreatScenario.damageScenarios` entry does not resolve to a `DamageScenario` |
| `E827` | `CybersecurityGoal.threatScenarios` entry does not resolve to a `ThreatScenario` |
| `E828` | `SecurityControl.implementsGoals` entry does not resolve to a `CybersecurityGoal` |
| `E829` | `VulnerabilityReport.mitigatedBy` entry does not resolve to a `SecurityControl` |
| `E830` | `VulnerabilityReport.affectedElements` entry does not resolve to any known element |
| `E831` | `derivedFromSecurityGoal` does not resolve or resolves to a non-`CybersecurityGoal` |
| `E832` | `derivedFromSafetyGoal` does not resolve or resolves to a non-`SafetyGoal` |

## Tier 2 coverage warnings (W800–W808)

| Code | Condition |
|---|---|
| `W800` | `HazardousEvent` not referenced by any `SafetyGoal.hazardousEvents` |
| `W801` | `SafetyGoal` has no integrity level (`asilLevel`, `silLevel`, or `plLevel`) |
| `W802` | `CybersecurityGoal` not implemented by any `SecurityControl.implementsGoals` |
| `W803` | `VulnerabilityReport` has `status: open` |
| `W804` | `CybersecurityGoal` has no `Requirement` with `derivedFromSecurityGoal` pointing to it |
| `W805` | `SafetyGoal` has no `Requirement` with `derivedFromSafetyGoal` pointing to it |
| `W806` | `SafetyGoal` has no `hazardousEvents:` — not grounded in any hazard analysis |
| `W807` | `Requirement` with `derivedFromSecurityGoal` has no `verificationMethod` |

## Tier 4 — Fault Tree Analysis (E900–E910, W900–W901)

| Code | Condition |
|---|---|
| `E900` | `FaultTree` missing `id`, `title`, `status`, or `topEvent` |
| `E901` | `FaultTree.id` does not match `FT-*` |
| `E902` | `topEvent` does not resolve or resolves to a non-`SafetyGoal` |
| `E903` | `FaultTreeGate` missing `id`, `title`, or `gateType` |
| `E904` | `FaultTreeGate.id` does not match `FTG-*` |
| `E905` | `gateType` not in `AND · OR · XOR · NOT · inhibit` |
| `E906` | `inputs` entry does not resolve to a `FaultTreeGate` or `FaultTreeEvent` |
| `E907` | `FaultTreeEvent` missing `id`, `title`, or `eventKind` |
| `E908` | `FaultTreeEvent.id` does not match `FTE-*` |
| `E909` | `eventKind` not in `basic · undeveloped · house` |
| `W900` | `FaultTree` has no gates or events (tree is empty) |
| `W901` | `FaultTreeGate` has no `inputs` |

## Tier 4 — FMEA (E911–E914, W902–W904)

| Code | Condition |
|---|---|
| `E911` | `FMEASheet` missing `id`, `title`, or `status` |
| `E912` | `FMEASheet.id` does not match `FMEA-*` |
| `E913` | FMEAEntry `id` does not match `FM-*` |
| `E914` | `fmeaSeverity`, `occurrence`, or `detection` outside 1–10 |
| `W902` | `FMEASheet` has no `entries` |
| `W903` | Computed RPN > 100 and no `recommendedAction` set |
| `W904` | Entry `ref` does not resolve to a known model element |

## Tier 4 — TARA container (E940–E941, W905)

| Code | Condition |
|---|---|
| `E940` | `TARASheet` missing `id`, `title`, or `status` |
| `E941` | `TARASheet.id` does not match `TARA-*` |
| `W905` | `TARASheet` has no rows in any section table |

## Product Line Engineering errors (E200–E221)

| Code | Condition |
|---|---|
| `E200` | `Configuration.id` does not match `CONF-*` |
| `E201` | `FeatureDef` or `Configuration` missing `id`, `title`, `status`, or `featureModel` |
| `E202` | Propagated parameter value outside component parameter `range:` |
| `E203` | `Configuration.parameterBindings` binds a parameter for a feature not selected |
| `E204` | `Configuration.parameterBindings` binds a parameter declared `isFixed: true` |
| `E205` | Bound parameter value violates `range:` constraint |
| `E206` | Bound parameter value not in `enumValues:` |
| `E207` | Circular `derivedFrom:` dependency between parameters of the same `FeatureDef` |
| `E208` | Duplicate `Configuration.id` |
| `E209` | `appliesWhen:` is malformed, or an operand does not resolve to a `FeatureDef` (operands of `and`/`or`/`not` expressions are each checked) |
| `E210` | Selected system feature has no component `Configuration` satisfying it |
| `E211` | Selected system feature satisfied by more than one component `Configuration` in same package |
| `E212` | `FeatureDef.requires:` or `excludes:` does not resolve to a `FeatureDef` |
| `E213` | Cross-feature `parameterConstraints` references unresolved parameter path |
| `E214` | `FeatureDef.contributesTo:` does not resolve to a `FeatureDef` |
| `E215` | `Configuration.derivedFrom:` base is not `approved` or `released` |
| `E216` | `Configuration.features` omits a `mandatory` feature |
| `E217` | `Configuration.features` selects both sides of an `alternative` group |
| `E218` | `Configuration.features` violates an `or` group's `cardinality:` |
| `E219` | `FeatureDef.requires:` constraint violated by selected features |
| `E220` | `FeatureDef.excludes:` constraint violated by selected features |
| `E221` | Cross-feature `parameterConstraint` evaluates to `false` |
| `E222` | `parameterBindings` key does not resolve to a declared `FeatureDef` parameter (bad path / unknown feature / undeclared parameter) |
| `E223` | (`feature-check --deep`) feature model is **void** — no valid configuration exists |
| `E224` | (`feature-check --deep`) a **dead feature** — selectable in no valid configuration |
| `E225` | (`feature-check --deep`) a `Configuration` is not a valid model of the feature model (mandatory/group/cardinality/parent violation) |
| `E226` | (`validate --config`) an active element's structural reference escapes the configuration (target inactive in this variant) |
| `E227` | (`feature-check --deep`) a structural reference is provably violable: a valid configuration activates the source without the target |

## Product Line Engineering warnings (W010–W017)

| Code | Condition |
|---|---|
| `W010` | `Configuration` does not bind a parameter declared `isRequired: true` on a selected feature |
| `W011` | `FeatureDef` with `groupKind: optional` selected in zero `Configuration` files |
| `W012` | `FeatureDef` with `groupKind: optional` selected in every `Configuration` |
| `W013` | Component `FeatureDef` has no `contributesTo:` or `excludes:` visible from system level |
| `W014` | `parameterConstraint` has `appliesWhen:` referencing a feature not in any `Configuration` |
| `W015` | A requirement is active in a `Configuration` (its `appliesWhen` holds) but no non-draft `TestCase` that runs in that `Configuration` verifies it. Only emitted when the variability dimension is active; honours draft suppression; gate with `--deny W015`. |
| `W016` | A `Configuration` parsed **zero** feature selections while a feature model exists — e.g. it used an unrecognized `selections:` key instead of the `features:` map. Surfaces the otherwise-silent all-N/A footgun. Not emitted when no `FeatureDef` is present. |
| `W017` | A selected feature declares a required parameter (`isRequired: true`, not fixed, no `default:`) that the `Configuration` does not bind. (Spec §9.11 nominally calls this `W010`, which this tool already uses for test-result ingestion.) |
| `W018` | (`feature-check --deep`) a **false-optional** feature — declared `optional` but forced selected whenever its parent is |
| `W019` | (`validate --config`) an active element's traceability reference escapes the configuration (target inactive in this variant) |
| `W020` | (`feature-check --deep`) a traceability reference is provably violable across some valid configuration |
| `W021` | (`feature-check --deep`) a dead element — its `appliesWhen` is unsatisfiable under the feature model |
| `W022` | (`feature-check --deep`) a requirement active in some configuration but covered in none |
| `W023` | (§12.8) a non-`draft` `Part`/`PartDef` has an `implementedBy:` path that does not exist on disk. Opt-in (only when `implementedBy:` is present); draft-suppressed; remote (`scheme://`) targets accepted as external and not checked. Path resolution matches `sourceFile`. Gate with `--deny W023`. |
