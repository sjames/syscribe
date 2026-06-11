# Syscribe Validation Rule Codes

## Parse-time errors — core (E001–E015, E023–E025, E300–E304)

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
| `E023` | A stable-ID numeric suffix is longer than the configured maximum (`[ids] max_digits`, default 8). The minimum (3) is enforced by `E006`. |
| `E024` | An **id-identified** type (`Requirement`, `TestCase`, the safety/security types, …) declares a `name:` field. Its label belongs in `title:`; remove `name:`. Every element has exactly one label field, fixed by identity class. |
| `E025` | A **name-identified** type (`PartDef`, `Package`, `FeatureDef`, all SysML structural types, …) declares a `title:` field. Its label belongs in `name:`; remove `title:`. A `FeatureDef` also carries a mandatory `FEAT-*` `id` (E201 if missing) — `id` and the label field are independent axes. |
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

## Tier 4 — Attack path analysis (E915–E921, W035–W037)

| Code | Condition |
|---|---|
| `E915` | `AttackTree` missing `id`, `title`, `status`, or `threatRef` |
| `E916` | `AttackTree.id` does not match `AT-*` |
| `E917` | `threatRef` does not resolve, or resolves to an element that is not a `ThreatScenario` |
| `E918` | `AttackTreeGate` missing `id`, `title`, or `gateType`, or `id` does not match `ATG-*` |
| `E919` | `AttackTreeGate.gateType` not in `AND · OR` |
| `E920` | `inputs` entry does not resolve to an `AttackTreeGate` or `AttackStep` |
| `E921` | `AttackStep` missing `id`/`title`, `id` not `ATS-*`, or `attackFeasibility` not in `high · medium · low · very_low` |
| `W035` | `AttackTree` computed (weakest-link) feasibility ≠ linked `ThreatScenario.attackFeasibility` (computed vs declared) |
| `W036` | `AttackTree` has no gates or steps (tree is empty) |
| `W037` | `AttackTreeGate` has no `inputs` |

## Product Line Engineering errors (E200–E221)

| Code | Condition |
|---|---|
| `E200` | `Configuration.id` does not match `CONF-*` |
| `E201` | A PLE element missing a required field: a `Configuration` missing `id`/`title`/`status`/`featureModel`, **or** a `FeatureDef` missing its mandatory `FEAT-*` `id`. (A `FeatureDef` is still name-labelled — `title:` on it is `E025`.) |
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
| `E213` | Cross-feature `parameterConstraints` references unresolved parameter path (`<FeatureDef>.<param>`) |
| `E214` | `FeatureDef.contributesTo:` does not resolve to a `FeatureDef` |
| `E215` | `Configuration.derivedFrom:` base is not `approved` or `released` |
| `E216` | `Configuration.features` omits a `mandatory` feature |
| `E217` | `Configuration.features` selects both sides of an `alternative` group |
| `E218` | `Configuration.features` violates an `or` group's `cardinality:` |
| `E219` | `FeatureDef.requires:` constraint violated by selected features |
| `E220` | `FeatureDef.excludes:` constraint violated by selected features |
| `E221` | Cross-feature `parameterConstraints` expression evaluates to `false` for a `Configuration` whose `appliesWhen:` holds (comparison/arithmetic over dotted refs; `feature-check`; default severity) |
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
| `W024` | (`feature-check`) an orphan `FeatureDef` — referenced by no `appliesWhen:` and selected by no `Configuration` (gates nothing, ships in nothing); gate with `--deny W024` |
| `W025` | (`feature-check`) a `parameterConstraints` violation (as `E221`) where the constraint declares `severity: warning`; gate with `--deny W025` |
| `E228` | (`validate`) invalid `appliesWhen:` placement (§9.10): nested under a package that already declares one; or on a `FeatureDef`/`Configuration`, a package whose subtree contains one, or the model-root package |
| `W026` | (`validate`) a `Package` declares `appliesWhen:` but gates no projectable element (empty subtree); gate with `--deny W026` |
| `W023` | (§12.8) a non-`draft` `Part`/`PartDef` has an `implementedBy:` path that does not exist on disk. Opt-in (only when `implementedBy:` is present); draft-suppressed; remote (`scheme://`) targets accepted as external and not checked. Path resolution matches `sourceFile`. Gate with `--deny W023`. |

## TestPlan (E600–E606, W610–W616)

| Code | Severity | Condition |
|---|---|---|
| `E600` | error | `TestPlan` missing `id`/`title`/`status`, or `id` does not match `^TP(-[A-Z0-9]{2,12})+-[0-9]{3,8}$` |
| `E601` | error | a `testCases:` entry does not resolve to a `TestCase` |
| `E602` | error | a `selection.testLevels` value is not one of `L1`–`L5` |
| `E603` | error | a `demonstrates:` target does not resolve to a `Requirement`/`SafetyGoal`/`CybersecurityGoal`/`Argument` |
| `E604` | error | `status` is not one of `draft · review · approved · active · retired` |
| `E605` | error | a `selection.domains` value is not one of `system`/`hardware`/`software` |
| `E606` | error | a `configurations:` entry does not resolve to a `Configuration` |
| `W610` | warning | `scope` is not in the recommended vocabulary (`unit·smoke·integration·hil·certification·security·regression`) |
| `W611` | warning | a member `TestCase` is active in **none** of the plan's bound configurations (escaping member) |
| `W612` | warning | the effective TestCase set is empty (no resolvable `testCases:` and no `selection:` match) |
| `W613` | warning | a `TestCase` named explicitly in `testCases:` has status `draft`/`retired` |
| `W614` | warning | an `approved`/`active` plan `demonstrates:` a `Requirement` that no member verifies (honours goal-closure — a member verifying a leaf of a demonstrated parent counts) |
| `W615` | warning | results-gated: an `approved` plan has a member whose ingested verdict is Fail/Missing (only when a results sidecar is loaded) |
| `W616` | warning | two plans share an identical `(configurations, scope)` pair (likely redundant) |

A duplicate `TestPlan` `id` is the generic `E101` (duplicate stable id).

## Custom fields (W041)

| Code | Severity | Condition |
|---|---|---|
| `W041` | warning | a `custom_fields` value is not a scalar or a list of scalars (e.g. a nested map); names the offending key |

## Naming (W042)

| Code | Severity | Condition |
|---|---|---|
| `W042` | warning | A qualified-name segment — an element's own name **or** a package/directory (namespace) name — is not a SysMLv2 **basic name** (`[A-Za-z_][A-Za-z0-9_]*`) and is not a stable id. Hyphens/spaces/punctuation are not allowed — rename using `_` or CamelCase. Such a name cannot be referenced in `appliesWhen`/`parameterConstraints` (where `-` is the subtraction operator). |
