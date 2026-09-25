# Syscribe Validation Rule Codes

## Parse-time errors — core (E001–E015, E023–E025, E300–E304)

| Code | Condition |
|---|---|
| `E000` | Internal fallback for an unrecognised walker-pass finding code (derive `E504`–`E506`, SysML v2 ingestion, plugins, …; should not appear in a healthy model) |
| `E001` | File does not begin with `---` (missing frontmatter delimiter) |
| `E002` | YAML frontmatter is not valid YAML 1.2 |
| `E003` | **RETIRED** — never emitted. There is no strict mode; an unrecognised top-level frontmatter key is the warning `W047`. |
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
| `E019` | `dalLevel:` is not in `A`–`E` (DO-178C) |
| `E020` | `verificationMethod:` is not `test`/`inspection`/`analysis`/`demonstration` |
| `E021` | `coverageTarget:` is not `statement`/`branch`/`MCDC` |
| `E022` | `requirementKind:` is not `stakeholder`/`system`/`software`/`hardware` |
| `E050` | Two selected features export the same `buildExports` variable name, unresolved by `buildOverrides:` (opt-in; §9.9) |
| `E023` | A stable-ID numeric suffix is longer than the configured maximum (`[ids] max_digits`, default 8). The minimum (3) is enforced by `E006`. |
| `E024` | **RETIRED** — formerly flagged a `name:` field on an id-identified type. `name` is now the single, required label on every element, so this code is no longer emitted. |
| `E025` | The removed `title:` field is declared on an element (any type, id-identified or name-identified). The `title` field is removed — rename it to `name`. |
| `E300` | `ADR.id` does not match `ADR-*` pattern |
| `E301` | `ADR` missing `id`, `name`, or `status` |
| `E302` | `reqDomain:` is not `system`, `hardware`, or `software` |
| `E303` | `domain:` is not `system`, `hardware`, or `software` |
| `E304` | `ADR.status` not in `proposed · accepted · deprecated · superseded` |

## Model-time errors — core (E101–E106, E310–E315)

| Code | Condition |
|---|---|
| `E016` | Cycle detected in the `supertype:` graph |
| `E017` | Cycle detected in the `derivedFrom:` graph |
| `E018` | Cycle detected in the `subsets:` graph |
| `E101` | Two elements have the same `id:` value |
| `E102` | `verifies:` reference cannot be resolved |
| `E103` | `derivedFrom:` reference cannot be resolved |
| `E104` | `verifies:` resolves to something that is not a native `Requirement` |
| `E105` | `derivedFrom:` resolves to something that is not a native `Requirement` |
| `E106` | `testFunctions[].scenario` does not match any Gherkin scenario title in this file |
| `E107` | Cycle detected in the `typedBy:` graph (including a usage typed by itself) |
| `E108` | Two elements — any origin (hand-authored, FMEA/TARA row explosion, SysMLv2 ingestion, stdio plugin, annotated source) — share a qualified name; names both files |
| `E110` | `supertype:` reference cannot be resolved |
| `E111` | `typedBy:` reference cannot be resolved (element-level or an inline `features:` entry) |
| `E112` | `subsets:` reference cannot be resolved |
| `E113` | `redefines:` reference cannot be resolved |
| `E114` | `satisfies:` reference cannot be resolved |
| `E310` | `Requirement` has `derivedFrom:` but no `breakdownAdr:` |
| `E311` | `breakdownAdr:` cannot be resolved or resolves to a non-`ADR` element |
| `E312` | A parent `Requirement` (has `derivedChildren`) appears in a `satisfies:` list |
| `E313` | `satisfies:` connects an architecture element and a requirement with incompatible `domain`/`reqDomain` |
| `E314` | `PartDef`/`Part` with `isDeploymentPackage: true` has no allocation to a `hardware` element in any §12.9 form (`Allocation` element top-level or per `features:` entry, `allocatedTo:` on the part, legacy authored `allocatedFrom:` on the target) |
| `E315` | `domain: software` element has `supertype:`/`typedBy:` referencing `domain: hardware`, or vice versa |
| `E316` | A `refines:` operand on a `UseCaseDef`/`UseCase` or behavioral `ActionDef`/`Action`/`StateDef`/`State` does not resolve, or resolves to a non-`Requirement`/`RequirementDef` |

## Warnings — core (W001–W007, W300–W305)

| Code | Condition |
|---|---|
| `W001` | Native `Requirement` normative text contains no `shall` |
| `W002` | `Requirement` at `approved`/`implemented` has no active `TestCase` in `verifiedBy` |
| `W003` | `Requirement` at `verified` but `verifiedBy` is empty or all entries are `retired` |
| `W004` | `sourceFile:` path does not exist on disk relative to model root |
| `W009` | A TestCase `testFunctions[].function` is not found in its `sourceFile` (live source-drift; planned/draft TestCases report `I010` instead) |
| `W005` | Native `Requirement` has no upstream link (`derivedFrom:`, `derivedFromSafetyGoal:` or `derivedFromCybersecurityGoal:`) and no `derivedChildren` (possible orphan) |
| `W006` | Both `silLevel:` and `asilLevel:` set on the same element — incompatible standards |
| `W007` | A type definition (e.g. `PartDef`, `PortDef`, `ItemDef`) is defined but never used as a `supertype:` or `typedBy:` type by any element. (An unrecognised frontmatter key is `W047`.) |
| `W008` | Element has no `type:` field — it will be ignored by most commands |
| `W010` | An `active` `TestCase`'s `testFunctions[].function` last failed, was ignored/skipped, or was absent in the ingested test results (`ingest-results` sidecar or `validate --results`). Inert unless results have been ingested; gate with `--deny W010`. (The product-line unbound-required-parameter warning is `W017`.) |
| `W300` | Leaf `Requirement` at `approved`/`implemented` has no satisfying architecture element |
| `W301` | **Retired** (GH #121) — no longer emitted; a leaf may be satisfied by several elements |
| `W302` | Leaf `Requirement` at `implemented`/`verified` still has `reqDomain: system` |
| `W303` | `breakdownAdr:` references an ADR with `status: proposed` |
| `W304` | `isDeploymentPackage: true` combined with `domain: hardware` |
| `W305` | Parent `Requirement` at `approved`/`implemented`/`verified` has no active `TestCase` at `testLevel: L3`–`L5` |
| `W306` | A high-integrity `Requirement` (`silLevel >= 4`/`asilLevel: D`) is not a fully integrated safety mechanism — draft, unsatisfied (leaf), or active in no `Configuration`. Gate with `--deny W306` |
| `W307` | A non-`draft` `UseCaseDef` carries no `refines:` link to a requirement (advisory, draft-suppressed; `--deny W307`) |
| `I010` | Informational: a **planned** `TestCase` (`status: draft`/`review`/`approved`) has a `sourceFile:` or `testFunctions[].function` that is not present yet — the planned-verification counterpart of `W004`/`W009`; never affects the exit status |

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

## Confirmation measures & DIA/CIA responsibility (E847–E851, E924, W038, W039)

| Code | Condition |
|---|---|
| `E847` | `ConfirmationMeasure` missing `id`, `name`, or `status` |
| `E848` | `ConfirmationMeasure.id` does not match `CM-*` pattern |
| `E849` | `ConfirmationMeasure.measureType` not in `confirmation_review · functional_safety_audit · functional_safety_assessment · cybersecurity_assessment` |
| `E850` | `ConfirmationMeasure.independenceLevel` not in `I1 · I2 · I3` |
| `E851` | A `confirms:` ref does not resolve to any model element |
| `E924` | `ConfirmationMeasure.status` not in `planned · in_progress · completed` |
| `W038` | A non-draft work product (`Requirement`, `PartDef`, `Part`, `SafetyGoal`, `CybersecurityGoal`) has no `responsibility:` field. **Opt-in:** dormant unless some element declares `responsibility:`. Gate with `--deny W038` |
| `W039` | A high-integrity item lacks its required independent assessment: an `asilLevel: D` **or** `silLevel: 3`/`silLevel: 4` `SafetyGoal`/native `Requirement` not confirmed by an I3 `functional_safety_assessment`; or a `calLevel: CAL4` `CybersecurityGoal` not confirmed by an I3 `cybersecurity_assessment`. **Opt-in:** dormant unless at least one `ConfirmationMeasure` exists. Gate with `--deny W039` |

## Tier 2 parse-time errors — HARA (E800–E806, E833–E837)

| Code | Condition |
|---|---|
| `E800` | `HazardousEvent` missing `id`, `name`, or `status` |
| `E801` | `severity` not in `S0 · S1 · S2 · S3` |
| `E802` | `exposure` not in `E0 · E1 · E2 · E3 · E4` |
| `E803` | `controllability` not in `C0 · C1 · C2 · C3` |
| `E804` | `HazardousEvent.id` does not match `HE-*` |
| `E805` | `SafetyGoal` missing `id`, `name`, or `status` |
| `E806` | `SafetyGoal.id` does not match `SG-*` |
| `E833` | `consequence` not in `Ca · Cb · Cc · Cd` (IEC 61508) |
| `E834` | `freqExposure` not in `Fa · Fb` |
| `E835` | `avoidance` not in `Pa · Pb` |
| `E836` | `demandRate` not in `W1 · W2 · W3` |
| `E837` | `plLevel` not in `a · b · c · d · e` |

## Tier 2 parse-time errors — TARA (E807–E824)

| Code | Condition |
|---|---|
| `E807` | `DamageScenario` missing `id`, `name`, or `status` |
| `E808` | `DamageScenario.id` does not match `DS-*` |
| `E809` | `damageSeverity` not in `severe · major · moderate · negligible` |
| `E810` | `impactCategories` entry not in `safety · financial · operational · privacy` |
| `E844` | `DamageScenario`/`ThreatScenario` `hazardRef` does not resolve, or resolves to a non-`HazardousEvent`/`SafetyGoal` |
| `E811` | `ThreatScenario` missing `id`, `name`, or `status` |
| `E812` | `ThreatScenario.id` does not match `TS-*` |
| `E813` | `attackFeasibility` not in `high · medium · low · very_low` |
| `E814` | `attackVector` not in `network · adjacent · local · physical` |
| `E845` | `ThreatScenario.riskTreatment` not in `avoid · reduce · share · retain` |
| `E815` | `CybersecurityGoal` missing `id`, `name`, or `status` |
| `E816` | `CybersecurityGoal.id` does not match `CSG-*` |
| `E817` | `securityProperty` not in `confidentiality · integrity · availability · authenticity` |
| `E818` | `calLevel` not in `CAL1 · CAL2 · CAL3 · CAL4` |
| `E819` | `SecurityControl` missing `id`, `name`, or `status` |
| `E820` | `SecurityControl.id` does not match `SC-*` |
| `E821` | `controlType` not in `prevention · detection · response · recovery` |
| `E822` | `VulnerabilityReport` missing `id`, `name`, or `status` |
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
| `E831` | `derivedFromCybersecurityGoal` does not resolve or resolves to a non-`CybersecurityGoal` |
| `E832` | `derivedFromSafetyGoal` does not resolve or resolves to a non-`SafetyGoal` |

## Tier 2 coverage warnings (W800–W808)

| Code | Condition |
|---|---|
| `W800` | `HazardousEvent` not referenced by any `SafetyGoal.hazardousEvents` |
| `W801` | `SafetyGoal` has no integrity level (`asilLevel`, `silLevel`, or `plLevel`) |
| `W802` | `CybersecurityGoal` not implemented by any `SecurityControl.implementsGoals` |
| `W803` | `VulnerabilityReport` has `status: open` |
| `W804` | `CybersecurityGoal` has no `Requirement` with `derivedFromCybersecurityGoal` pointing to it |
| `W805` | `SafetyGoal` has no `Requirement` with `derivedFromSafetyGoal` pointing to it |
| `W806` | `SafetyGoal` has no `hazardousEvents:` — not grounded in any hazard analysis |
| `W807` | `Requirement` with `derivedFromCybersecurityGoal` has no `verificationMethod` |

## Quantitative HW safety metrics (E846, W033)

| Code | Condition |
|---|---|
| `E846` | `diagnosticCoverage` or `latentDiagnosticCoverage` is outside `0.0`–`1.0` |
| `W033` | A `SafetyGoal` with diagnostic-coverage data has a computed SPFM/LFM/PMHF below/above its ASIL/SIL target. Opt-in; gate with `--deny W033` |

## Safety↔security co-engineering & cyber-risk (W028, W030, W031, W032)

| Code | Condition |
|---|---|
| `W028` | The same `extRef` value is declared by two or more elements (opt-in; §3) |
| `W030` | A `DamageScenario` whose `impactCategories` includes `safety` has no `hazardRef` (cross-domain gap; opt-in) |
| `W031` | A `ThreatScenario` whose computed risk is `high`/`critical` has no `riskTreatment` and is addressed by no `CybersecurityGoal`. Gate with `--deny W031` |
| `W032` | A `CybersecurityGoal.calLevel` is below the expected minimum CAL for the max risk over its listed threats. Gate with `--deny W032` |

## Freedom From Interference (W034)

| Code | Condition |
|---|---|
| `W034` | For an allocation target with ≥2 sources (edges from the §12.9 unified allocation-edge set), a mixed-criticality source pair has no freedom-from-interference argument (`ffiRationale:` or `accepted` `breakdownAdr:`). Opt-in; gate with `--deny W034` |

## Integrity-level propagation — ASIL/SIL decomposition (E865, W860)

| Code | Condition |
|---|---|
| `E865` | ASIL D / SIL 4 decomposition siblings (uniformly-lower children) share a `satisfies:` target — channels must be architecturally independent (§22.3) |
| `W860` | An ASIL D / SIL 4 requirement has a single uniformly-lower child — a decomposition needs ≥2 independent channels (§22.3) |

## GSN safety-argument layer (E852–E860, W040)

| Code | Condition |
|---|---|
| `E852` | `Argument` missing `id`, `name`, or `status` |
| `E853` | `Argument.id` does not match `ARG-*` |
| `E854` | `Argument.argumentType` not in `claim · strategy · solution` (absent → `claim`) |
| `E855` | An `Argument.supports`/`evidence` ref does not resolve to any model element |
| `E856` | `AssumptionOfUse` missing `id`, `name`, or `status` |
| `E857` | `AssumptionOfUse.id` does not match `AOU-*` |
| `E858` | An `AssumptionOfUse.appliesTo` ref does not resolve to any model element |
| `E859` | `AssumptionOfUse.appliesTo` resolves to a non-`SafetyGoal`/`CybersecurityGoal`/`Argument`/`Requirement` (REQ-TRS-SEC-004) |
| `E860` | `ConfirmationMeasure.confirms` resolves to a non-`SafetyGoal`/`CybersecurityGoal`/`HazardousEvent`/`Requirement` (REQ-TRS-SEC-005) |
| `E718` | An `Argument.evidence` entry is not a scalar reference (expected a string id/qname — e.g. a `PlanningItem`-style `{ref:, path:}` mapping on an `Argument`) |
| `W040` | A `claim`/`strategy` `Argument` has neither `supports` nor `evidence` (orphan GSN node) |

## Budget expression validation (E866–E868, W060)

| Code | Condition |
|---|---|
| `E866` | A budget `CalculationDef`'s `evaluate:` does not resolve to a `ConstraintDef` |
| `E867` | The budget `body:` expression has a syntax error |
| `E868` | A `feature_ref` operand resolves to no numeric attribute in scope |
| `W060` | The budget value violates the `evaluate:` constraint (best-effort; draft-suppressed; `--deny W060`) |

## Trade studies (E869–E877, W061–W064, §15)

| Code | Condition |
|---|---|
| `E869` | `TradeStudy` missing `id`, `name`, `status`, `criteria`, `alternatives`, or `scores` |
| `E870` | `TradeStudy.id` does not match `TRD-*` |
| `E871` | A `criteria:` entry is missing `name`, `weight`, or `direction` |
| `E872` | A `criteria[].weight` is not in `[0.0, 1.0]`, or all weights are zero |
| `E873` | A `criteria[].direction` is not `maximize`/`minimize` |
| `E874` | `alternatives:` is empty |
| `E875` | An `alternatives:` entry is missing `name` |
| `E876` | A `scores:` entry references an unknown alternative or criterion |
| `E877` | A `scores[].score` is not a number |
| `W061` | A `status: complete` study has no `decision:` ADR |
| `W062` | `objective:` is present but unresolved (draft-suppressed) |
| `W063` | The score matrix is incomplete (draft-suppressed) |
| `W064` | An `alternatives[].element` is present but unresolved (draft-suppressed) |

## State machine warnings (W070–W080, W929, §22.1)

| Code | Condition |
|---|---|
| `W070` | Dead state — a substate has no incoming transition and is not `isInitial: true` |
| `W071` | Trap state — a substate has no outgoing transition and is not `isFinal: true` |
| `W072` | Non-determinism — two+ transitions from one source with the same `accept` payload, none guarded |
| `W073` | Missing initial — a single-region `StateDef` with substates has no `isInitial: true` substate |
| `W074` | Multiple initial — more than one substate is `isInitial: true` |
| `W075` | A transition uses the deprecated `from:`/`to:`/`trigger:` keys instead of `source:`/`target:`/`accept:` |
| `W076` | Unresolved endpoint — a transition `source`/`target` names no state and resolves to no element |
| `W077` | Cross-region transition between two regions of an `isParallel` state |
| `W078` | Parallel arity — an `isParallel: true` state declares fewer than two regions |
| `W079` | Unresolved behavior — a state `entry`/`do`/`exit` action or transition `effect` resolves to no element |
| `W929` | Incomplete transition — a top-level transition has no `source:`, or any transition has no `target:` (§8.8.3); it would otherwise contribute no edge. Draft-suppressed; `--deny W929` |
| `W080` | A `Sequence` diagram's subject `ActionDef` has a `SendAction`/`AcceptAction` not referenced by any `edges:` entry |

## Diagram errors and warnings (E400–E404, W400–W415)

| Code | Condition |
|---|---|
| `E400` | `diagramKind: Mermaid` but body has no ` ```mermaid ` block |
| `E401` | `diagramKind: PlantUML` but body has no ` ```plantuml ` block |
| `E402` | `svgFile:`/companion SVG path does not exist on disk |
| `E403` | `pumlMode:` declares an unrecognised value (only `companion`) |
| `E404` | `pumlMode: companion` set but the element has no `diagramKind:` |
| `W400` | Diagram has no `diagramKind` — rendering mode ambiguous |
| `W401` | `subject:` does not resolve to a known element |
| `W402` | Shape `ref:` does not resolve (and is not a sub-feature of a known element) |
| `W403` | Edge `source`/`target` is not a defined shape id in this diagram |
| `W404` | An operation parameter's `typedBy` or an operation's `returnType` does not resolve to a known element (a warning, since standard-library types may be unregistered) |
| `W405` | SVG body is inconsistent with `svgMode` |
| `W406` | Frontmatter `shapes`/`edges` id has no matching `id="..."` in the inline SVG |
| `W407` | SVG element `id` has no matching frontmatter `shapes`/`edges` entry |
| `W408` | Mermaid `%% ref:` annotation does not resolve to a known element |
| `W409` | Mermaid diagram has no `%% ref:` annotations |
| `W410` | Mermaid `%% link:` annotation does not resolve to a known element |
| `W411` | Shape `link:` value does not resolve to a known element |
| `W412` | SVG `href="..."` attribute does not resolve to any model element file |
| `W413` | `pumlMode: companion` body contains no image reference to its companion (REQ-TRS-PUML-030) |
| `W414` | `pumlMode: companion` `.puml` file not yet generated (REQ-TRS-PUML-031) |
| `W415` | `[plantuml] style_file` path in `.syscribe.toml` does not exist (REQ-TRS-PUML-042) |

## Build-system integration (E050, W050, §9.9)

| Code | Condition |
|---|---|
| `W050` | A selected feature contributes no build variable (no `buildExports:`/`buildVar:`). Opt-in; gate with `--deny W050` (`E050` is in the parse-time table) |

## Allocation and derive errors, structural warnings (E500–E506, W500–W503, W930)

| Code | Condition |
|---|---|
| `E500` | A feature with `type: Allocation` has an `allocatedFrom:` that does not resolve |
| `E501` | A feature with `type: Allocation` has an `allocatedTo:` that does not resolve |
| `E502` | An `allocatedFrom:` entry (any element) does not resolve to a known element |
| `E503` | An `allocatedTo:` entry (any element) does not resolve to a known element |
| `E504` | Cyclic dependency between `derive:` fields (a field reads itself directly or via other derived fields); reported on each participating element naming the cycle; the cyclic fields are not evaluated |
| `E505` | A `derive:` formula does not parse, the `derive:` value is not a mapping, or a formula is not a string |
| `E506` | A `derive:` formula's `elements["QName"]` names no element |
| `W500` | `viewpoint:` on a View does not resolve to a `ViewpointDef` |
| `W501` | `exhibitsStates:` entry does not resolve to any known element |
| `W502` | `expose:` entry on a View does not resolve to any known element |
| `W930` | A `features:` entry on a non-`Allocation` element declares an allocation (`type: Allocation`, or `allocatedFrom:`/`allocatedTo:`) — only a `type: Allocation` element carries features-form allocations (§12.9), so it contributes no allocation edge; use `allocatedTo:` on the source or a standalone `Allocation` element |
| `W503` | The same allocation edge is declared by more than one form — `allocatedTo:` on the source, an `Allocation` element, a legacy authored `allocatedFrom:` on the target (redundant) |

## Documentation warnings (W600, W601)

| Code | Condition |
|---|---|
| `W600` | `PartDef`/`Part` has an empty documentation body |
| `W601` | `ActionDef`/`Action` has an empty documentation body |

## Review records (E700–E705, W700, W704, §19)

| Code | Condition |
|---|---|
| `E700` | `ReviewRecord` missing `id`, `name`, `status`, `reviewType`, or `reviews` |
| `E701` | `ReviewRecord.id` does not match `RR-*` |
| `E702` | `ReviewRecord.status` not in `open · closed · waived` |
| `E703` | `ReviewRecord.reviewType` not in the allowed enum |
| `E704` | A `reviews:` entry does not resolve |
| `E705` | An `items[].disposition` not in `open · closed · not_applicable` |
| `W700` | A `status: closed` review has an `items[]` with `disposition: open` |
| `W704` | A non-`draft` native Requirement appears in no `ReviewRecord.reviews:` list (opt-in; `--deny W704`) |

## Native PlanningItem (E706–E717, E719–E723, W308–W311, §23, ADR-SYS-PLANITEM-001)

| Code | Condition |
|---|---|
| `E706` | `PlanningItem.id` does not match `PI-*` |
| `E707` | `PlanningItem` missing `id`, `name`, or `status` |
| `E708` | `PlanningItem.status` not in `todo · in_progress · blocked · done` |
| `E709` | `PlanningItem.itemType` (if present) not in `bug · task · feature` |
| `E710` | `parent:` reference does not resolve |
| `E711` | `parent:` resolves to something that is not a `PlanningItem` |
| `E712` | A `parent:` chain forms a cycle |
| `E713` | A top-level item (no `parent:`) has no `achieves:` entry |
| `E714` | An `achieves:` entry does not resolve |
| `E715` | An `achieves:` entry does not resolve to a native Requirement |
| `E716` | An `evidence[].ref` does not resolve (and is not waived by that entry's own `rationale:`) |
| `E717` | An `evidence[].path` does not exist on disk (and is not waived) |
| `E719` | A leaf item (empty computed `children`) at `status: done` has no non-waived, resolving `evidence:` entry — graded harder than the analogous `W300` (a warning), since claiming done with no proof is a correctness defect |
| `E720` | A `blockedBy:` entry does not resolve to any model element |
| `E721` | A `blockedBy:` chain forms a cycle (directly or through other PlanningItems) |
| `W308` | A non-empty `blockedBy:` while `status` is not `blocked` — likely stale. The converse (`status: blocked` with empty `blockedBy:`) raises nothing — being blocked needs no proof, unlike claiming done. |
| `E722` | `assignedTo:` names a username not present in the declared `[users]` roster (checked only when non-empty) |
| `E723` | `assignedTo:` is not a valid Unix-style username `^[a-z_][a-z0-9_-]{0,31}$` (checked unconditionally) |
| `W309` | A `[users]` key in `.syscribe.toml` is not a valid username — entry ignored, excluded from the roster |
| `W310` | A `done` `PlanningItem`'s `achieves:` Requirement hasn't met the verification bar `validate` already applies to it directly — an active TestCase for a leaf, an active integration-level (`L3`/`L4`/`L5`) TestCase for a parent (mirrors `W002`/`W305`, scoped to the specific PlanningItem, issue #114) |
| `W311` | Two `PlanningItem`s that are both active (`status: in_progress`, or explicitly claimed via `claimedBy:`) overlap by a shared `achieves:` Requirement or an `evidence[].path` resolving to the same repo-relative path — likely duplicate concurrent work (issue #115) |

`claimedBy:`/`claimedAt:` are advisory ownership markers (issue #115), written/cleared by the
dedicated `claim <PI-id> --by <agent-id>`/`release <PI-id>` commands — not the generic MCP
write path. Every other PlanningItem field is still queried via `list`/`show`/`ls`/`find`/`refs`
and written via the generic MCP element tools; no dedicated CLI subcommand or MCP tool for
those yet.

## IEC 62443 Zone/Conduit (E950–E956, E925, E926, W950–W953, §13)

| Code | Condition |
|---|---|
| `E950` | `Zone` missing `id`/`name`/`status`/`targetSL` |
| `E951` | `Zone.id` not a `ZN-*` id |
| `E952` | `Conduit` missing `id`/`name`/`status`/`fromZone`/`toZone` |
| `E953` | `Conduit.id` not a `CD-*` id |
| `E954` | `Conduit.fromZone`/`toZone` unresolved or not a `Zone` |
| `E955` | `Zone.members:` entry unresolved or not a `PartDef`/`Part` |
| `E956` | `PartDef`/`Part.inZone:` unresolved or not a `Zone` |
| `E925` | `targetSL:`/`achievedSL:` on a `Zone`/`Conduit`/`PartDef`/`Part` outside the Security Level range `1`–`4` |
| `E926` | `Zone`/`Conduit` `status:` not in `draft · review · approved · deprecated` |
| `W950` | `Zone.achievedSL < targetSL` (SL gap) |
| `W951` | `Conduit.achievedSL` below a connected zone's `targetSL` (opt-in) |
| `W952` | A part declares `targetSL` but belongs to no zone (opt-in) |
| `W953` | An `approved` `Zone` (`targetSL >= 2`) referenced by no `Conduit` |

## Multi-repository composition (E510–E515, W510–W512, §14)

Active only when `[repos]` is configured in `.syscribe.toml`.

| Code | Condition |
|---|---|
| `E510` | Circular repo import — a repo transitively imports back into this model |
| `E511` | `repos.<alias>.path` is absent on disk and no `ref:` is configured |
| `E512` | A cross-repo `verifies`/`derivedFrom`/`satisfies`/`allocatedTo`/`supertype`/`typedBy`/`subsets`/`redefines` reference resolves in neither the local model nor any loaded repo (reported instead of `E102`/`E103`/`E110`–`E114`/`E503` when `[repos]` is configured) |
| `E513` | `repoImports[].repo` names an alias not present in `[repos]` |
| `E514` | `repoImports[].qname` does not resolve to any element in the named repo |
| `E515` | Two repos export the same stable ID — the local model and a peer, or two different peer repos (the id namespace is global) |
| `W510` | A repo in `[repos]` has no `ref:` — composition is not pinned (opt-in; `--deny W510`) |
| `W511` | A peer repo's git `HEAD` has drifted from its configured `ref:` (opt-in; `--deny W511`) |
| `W512` | A peer submodule's gitlink disagrees with its configured `ref:` (opt-in; `--deny W512`) |

## Hierarchical product-line composition (E516–E519, E523, W513, §14, ADR-SYS-HPLE-001)

A `Configuration` may declare `subConfigurations:` naming one or more other `Configuration`s it consolidates — reachable locally or via `[repos]`, at any depth. Dormant unless some `Configuration` declares `subConfigurations:`.

| Code | Condition |
|---|---|
| `E516` | A `subConfigurations:` entry does not resolve to any element, locally or in a loaded peer repo |
| `E517` | A `subConfigurations:` entry resolves to a real element that is not a `Configuration` |
| `E518` | A `subConfigurations:` entry resolves to a `Configuration` that is not itself internally valid (a validation error, a void feature model, or `feature-check --deep`'s `E225`) — or the chain exceeds the bounded consolidation depth |
| `E519` | A `parameterBindings:` entry resolved transitively through `subConfigurations:` targets a `FeatureDef` the owning peer `Configuration` does not itself select — the cross-tier extension of `E203` |
| `E523` | A transitively-resolved `parameterBindings:` entry double-binds a parameter some nearer tier on the path — local or peer, the owner itself or an intermediate consolidator — already supplies |
| `W513` | Opt-in, `--deny`-gateable: a selected, required, no-default parameter anywhere in a consolidated `subConfigurations:` subtree remains unbound after every tier's own `parameterBindings:` — never a hard error, since deferral to a still-higher tier is the deliberate mechanism this feature exists for |

`parameterBindings:` itself is reused unchanged, extended to resolve transitively through `subConfigurations:` at any depth using a parameter's ordinary, already-mounted qname (no new addressing syntax); its existing intrinsic checks (`E204` fixed, `E205` range, `E206` enum, `E222` unresolved, `W027` runtime `bindingTime:`) apply identically whether the target is local or reached transitively. `E203` (feature not selected) and `W017` (required-and-unbound) stay scoped to a `Configuration`'s own local selection — the cross-tier equivalents are `E519` and `W513` respectively. A lower tier carries zero awareness of, or reference to, whoever consolidates it: `bindTo:` (component→system propagation) is explicitly not the mechanism here and continues to resolve purely within its own model.

## Documentation linting (W099–W103, `lint-docs`)

The `lint-docs` command scans external `.md`/`.svg` docs for references that no longer resolve.

| Code | Condition |
|---|---|
| `W099` | An unresolvable stable-ID token (`REQ-*`/`TC-*`/…) in prose |
| `W100` | A qualified name inside a ` ```mermaid ` block that does not resolve |
| `W101` | An SVG `sysml:ref="…"` that does not resolve |
| `W102` | A local image/diagram embed path that does not exist (remote URIs accepted) |
| `W103` | Advisory: a package `_index.md` body enumerates three or more of the package's own direct members by stable id — membership is generated (`show <package>`); describe purpose instead. Does not affect the exit status |

## §12.8 Implementation trace (W029)

| Code | Condition |
|---|---|
| `W029` | A non-`draft` requirement with an integrity level declares a `wcet:` claim but no active measuring `TestCase` verifies it (timing analog of `W702`; `--deny W029`) |

## Tier 4 — Fault Tree Analysis (E900–E909, E927, W900–W901, W926, W927)

| Code | Condition |
|---|---|
| `E900` | `FaultTree` missing `id`, `name`, `status`, or `topEvent` |
| `E901` | `FaultTree.id` does not match `FT-*` |
| `E902` | `topEvent` does not resolve or resolves to a non-`SafetyGoal` |
| `E903` | `FaultTreeGate` missing `id`, `name`, or `gateType` |
| `E904` | `FaultTreeGate.id` does not match `FTG-*` |
| `E905` | `gateType` not in `AND · OR · XOR · NOT · inhibit` |
| `E906` | `inputs` entry does not resolve to a `FaultTreeGate` or `FaultTreeEvent` |
| `E907` | `FaultTreeEvent` missing `id`, `name`, or `eventKind` |
| `E908` | `FaultTreeEvent.id` does not match `FTE-*` |
| `E909` | `eventKind` not in `basic · undeveloped · house` |
| `E927` | `FaultTreeEvent.ref` (the modelled architecture element) does not resolve to a known element |
| `W900` | `FaultTree` has no gates or events (tree is empty) |
| `W901` | `FaultTreeGate` has no `inputs` |
| `W926` | `FaultTreeEvent.fmeaRef` does not resolve to a known `FMEAEntry` (FTA↔FMEA cross-link) |
| `W927` | `FMEAEntry.ftaRef` does not resolve to a known `FaultTreeEvent` (FMEA↔FTA cross-link) |

## Tier 4 — FMEA (E911–E914, E922, E923, W902–W904, W928)

| Code | Condition |
|---|---|
| `E911` | `FMEASheet` missing `id`, `name`, or `status` |
| `E912` | `FMEASheet.id` does not match `FMEA-*` |
| `E913` | FMEAEntry `id` does not match `FM-*` |
| `E914` | `fmeaSeverity`, `occurrence`, or `detection` outside 1–10 |
| `E922` | An `entries:` row contains an unrecognised key (silent data loss in a safety analysis — error) |
| `E923` | An `FMEASheet` `entries:` row has no string `id:` (or is not a mapping) — it cannot become an `FMEAEntry` and is dropped from validation and `fmea report`; reported on the sheet, naming the row's 1-based position and its `failureMode:`/`name:` |
| `W902` | `FMEASheet` has no `entries` |
| `W903` | Computed RPN > 100 and no `recommendedAction` set. RPN is `fmeaSeverity × occurrence × detection` when all three are present (an explicit `rpn:` is used only when a factor is missing — see `W928`) |
| `W904` | Entry `ref` does not resolve to a known model element |
| `W928` | An `entries:` row declares `fmeaSeverity` (or `severity`), `occurrence`, `detection` **and** an explicit `rpn:` that differs from their product — the computed `S × O × D` is kept; the message names the row, the explicit and the computed value. Silent when `rpn:` equals the product or any factor is absent |

## Tier 4 — TARA container (E940–E941, W905)

| Code | Condition |
|---|---|
| `E940` | `TARASheet` missing `id`, `name`, or `status` |
| `E941` | `TARASheet.id` does not match `TARA-*` |
| `W905` | `TARASheet` has no rows in any section table |

## Tier 4 — Attack path analysis (E915–E921, W035–W037)

| Code | Condition |
|---|---|
| `E915` | `AttackTree` missing `id`, `name`, `status`, or `threatRef` |
| `E916` | `AttackTree.id` does not match `AT-*` |
| `E917` | `threatRef` does not resolve, or resolves to an element that is not a `ThreatScenario` |
| `E918` | `AttackTreeGate` missing `id`, `name`, or `gateType`, or `id` does not match `ATG-*` |
| `E919` | `AttackTreeGate.gateType` not in `AND · OR` |
| `E920` | `inputs` entry does not resolve to an `AttackTreeGate` or `AttackStep` |
| `E921` | `AttackStep` missing `id`/`name`, `id` not `ATS-*`, or `attackFeasibility` not in `high · medium · low · very_low` |
| `W035` | `AttackTree` computed (weakest-link) feasibility ≠ linked `ThreatScenario.attackFeasibility` (computed vs declared) |
| `W036` | `AttackTree` has no gates or steps (tree is empty) |
| `W037` | `AttackTreeGate` has no `inputs` |

## Asset identification — ISO/SAE 21434 §15.3 (E861–E864, W810)

| Code | Condition |
|---|---|
| `E861` | `Asset` missing `id`, `name`, or `status` |
| `E862` | `Asset.id` does not match the `ASSET-*` pattern |
| `E863` | `Asset.cybersecurityProperties` entry not in `confidentiality · integrity · availability · authenticity` |
| `E864` | `DamageScenario.assets` entry does not resolve to an `Asset` element |
| `W810` | `Asset` not referenced by any `DamageScenario.assets` (asset-identification gap) |

## Security test method — ISO/SAE 21434 §13.3 (W809)

| Code | Condition |
|---|---|
| `W809` | `TestCase.securityTestMethod` not in `fuzz · penetration_test · security_regression · vulnerability_scan · threat_modeling` |

## Product Line Engineering errors (E200–E237)

| Code | Condition |
|---|---|
| `E200` | `Configuration.id` does not match `CONF-*` |
| `E201` | A PLE element missing a required field: a `Configuration` missing `id`/`name`/`status`/`featureModel`, **or** a `FeatureDef` missing its mandatory `FEAT-*` `id`. (A `FeatureDef` carries `name` as its label; a stray `title:` on it is `E025`.) |
| `E202` | Propagated parameter value outside component parameter `range:` |
| `E203` | `Configuration.parameterBindings` binds a parameter for a feature not selected |
| `E204` | `Configuration.parameterBindings` binds a parameter declared `isFixed: true` |
| `E205` | Bound parameter value violates `range:` constraint |
| `E206` | Bound parameter value not in `enumValues:` |
| `E207` | Circular `derivedFrom:` dependency between parameters of the same `FeatureDef` |
| `E209` | `appliesWhen:` is malformed, or an operand does not resolve to a `FeatureDef` (operands of `and`/`or`/`not` expressions are each checked) |
| `E212` | `FeatureDef.requires:` or `excludes:` does not resolve to a `FeatureDef` |
| `E215` | A `Configuration`'s `derivedFrom:` base (§9.8 inheritance) is not `approved` or `released` |
| `E213` | Cross-feature `parameterConstraints` references unresolved parameter path (`<FeatureDef>.<param>`) |
| `E219` | `FeatureDef.requires:` constraint violated by selected features |
| `E220` | `FeatureDef.excludes:` constraint violated by selected features |
| `E221` | Cross-feature `parameterConstraints` expression evaluates to `false` for a `Configuration` whose `appliesWhen:` holds (comparison/arithmetic over dotted refs; `feature-check`; default severity) |
| `E222` | `parameterBindings` key does not resolve to a declared `FeatureDef` parameter (bad path / unknown feature / undeclared parameter) |
| `E229` | A parameter's `bindingTime:` is earlier than that of a `derivedFrom`/`bindTo` source it depends on (impossible ordering; checked when both ends declare a `bindingTime:`) |
| `E230` | A parameter's `bindingTime:` is not `compile`/`load`/`runtime` (§9.7) |
| `E223` | (`feature-check --deep`) feature model is **void** — no valid configuration exists |
| `E224` | (`feature-check --deep`) a **dead feature** — selectable in no valid configuration |
| `E225` | (`feature-check --deep`) a `Configuration` is not a valid model of the feature model (mandatory/group/cardinality/parent violation) |
| `E226` | (`validate --config`) an active element's structural reference escapes the configuration (target inactive in this variant) |
| `E227` | (`feature-check --deep`) a structural reference is provably violable: a valid configuration activates the source without the target |
| `E231` | (§9.6a single-file feature model) a `featureTree:` entry is not a mapping, has no `name:`, or its dotted `name:` path is malformed — the entry is skipped |
| `E232` | (§9.6a) a `featureTree:` entry's resolved qualified name collides with an existing element (or another entry) of the same qname |
| `E233` | (§9.6a) a `crossTreeConstraints:` entry is not a mapping, has no `feature:`, has an empty path segment in `feature:`/`requires:`/`excludes:`, or its `feature:` does not resolve to a `FeatureDef` synthesized from the same sheet's `featureTree:` |
| `E234` | (§9.8) a `Configuration`'s `derivedFrom:` base does not resolve to any element of the model — the base must be local (consolidate a peer product line with `subConfigurations:`); the configuration inherits nothing |
| `E235` | (§9.8) a `Configuration`'s `derivedFrom:` base resolves to an element that is not a `Configuration`; the configuration inherits nothing |
| `E236` | (§9.8) a `Configuration` is on a `derivedFrom:` inheritance cycle — reported on every member; none of them inherits (`E017` is not raised for Configuration cycles) |
| `E237` | (§9.8) a `Configuration`'s `derivedFrom:` names more than one base — a Configuration inherits from at most one; it inherits nothing |

## Product Line Engineering warnings (W011–W027, W048)

| Code | Condition |
|---|---|
| `W011` | `FeatureDef` with `groupKind: optional` selected in zero `Configuration` files |
| `W012` | `FeatureDef` with `groupKind: optional` selected in every `Configuration` |
| `W014` | `parameterConstraint` has `appliesWhen:` referencing a feature not in any `Configuration` |
| `W015` | A requirement is active in a `Configuration` (its `appliesWhen` holds) but no non-draft `TestCase` that runs in that `Configuration` verifies it. Only emitted when the variability dimension is active; honours draft suppression; gate with `--deny W015`. |
| `W016` | A `Configuration` parsed **zero** feature selections while a feature model exists — e.g. it used an unrecognized `selections:` key instead of the `features:` map. Surfaces the otherwise-silent all-N/A footgun. Not emitted when no `FeatureDef` is present. |
| `W017` | A selected feature declares a required parameter (`isRequired: true`, not fixed, no `default:`) that the `Configuration` does not bind. (`W010` is test-result ingestion — see the core warnings.) |
| `W018` | (`feature-check --deep`) a **false-optional** feature — declared `optional` but forced selected whenever its parent is |
| `W019` | (`validate --config`) an active element's traceability reference escapes the configuration (target inactive in this variant) |
| `W020` | (`feature-check --deep`) a traceability reference is provably violable across some valid configuration |
| `W021` | (`feature-check --deep`) a dead element — its `appliesWhen` is unsatisfiable under the feature model |
| `W022` | (`feature-check --deep`) a requirement active in some configuration but covered in none |
| `W024` | (`feature-check`) an orphan `FeatureDef` — referenced by no `appliesWhen:` and selected by no `Configuration` (gates nothing, ships in nothing); gate with `--deny W024` |
| `W025` | (`feature-check`) a `parameterConstraints` violation (as `E221`) where the constraint declares `severity: warning`; gate with `--deny W025` |
| `E228` | (`validate`) invalid `appliesWhen:` placement (§9.10): nested under a package that already declares one; or on a `FeatureDef`/`Configuration`, a package whose subtree contains one, or the model-root package |
| `W026` | (`validate`) a `Package` declares `appliesWhen:` but gates no projectable element (empty subtree); gate with `--deny W026` |
| `W027` | (`validate`) a `Configuration` binds a parameter whose `bindingTime: runtime` (resolved by the running system, not at configuration time); gate with `--deny W027` |
| `W048` | (§9.6a) `featureTree:`/`crossTreeConstraints:` is declared on an element whose `type:` is not `FeatureModel`, or `parameterConstraints:` on anything other than `Package`/`LibraryPackage`/`Namespace`/`FeatureModel` — the field is inert and ignored |
| `W023` | (§12.8) a non-`draft` `Part`/`PartDef`/`Interface`/`InterfaceDef` has an `implementedBy:` path that does not exist on disk. Opt-in (only when `implementedBy:` is present); draft-suppressed; remote (`scheme://`) targets and package-registry references (`crates.io:tokio@1.38.0`, `npm:…`, `pypi:…`, `maven:…`, `nuget:…`, `github:org/repo@v1`) accepted as external and not checked. Path resolution matches `sourceFile`. Gate with `--deny W023`. |

## TestPlan (E600–E606, W610–W616)

| Code | Severity | Condition |
|---|---|---|
| `E600` | error | `TestPlan` missing `id`/`name`/`status`, or `id` does not match `^TP(-[A-Z0-9]{2,12})+-[0-9]{3,8}$` |
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

## User-defined link types (E630–E636, W630, W631, ADR-SYS-LINKTYPE-001)

Dormant unless `.syscribe.toml` declares `[linkTypes]` or an element carries `links:`.

| Code | Condition |
|---|---|
| `W630` | A `[linkTypes.<name>]` entry in `.syscribe.toml` is malformed (bad/colliding name or inverse, unparseable `cardinality`, non-zero lower bound without `sourceTypes`, unknown element type, unsupported `extends`, `relax`/`coverage` without `extends`, a code not relaxable for the base) — the entry is ignored as a whole, so its uses raise `E630`; or an entry has an unknown key (key ignored, entry kept) |
| `E630` | A `links:` key is not a declared link type — the message lists the declared types, or says none are declared and how to add a `[linkTypes.<name>]` table to `.syscribe.toml` |
| `E631` | `links:` is not a mapping, or a key's value is not a reference or a list of references |
| `E632` | A `links:` target does not resolve (by id or qualified name, like `satisfies:`) |
| `E633` | The element's `type:` is not in the link type's declared `sourceTypes` |
| `E634` | A resolved target's `type:` is not in the link type's declared `targetTypes` (names the target) |
| `E635` | An element holds more targets of a link type than its `cardinality` upper bound |
| `W631` | A non-`draft` element whose type is in a link type's `sourceTypes` holds fewer targets than the `cardinality` lower bound |
| `E636` | A cycle — including a self-link — formed solely by links of an `acyclic = true` type; reported once per cycle, naming its members |

A type that `extends` `satisfies`/`verifies`/`derivedFrom`/`refines` is treated as that base link by
every base rule and reverse index (`E104`, `E105`, `E310`, `E312`, `E313`, `E316`, `W002`, `W300`,
`W303`, `W305`, …) except the codes it lists in `relax` (`E310`/`W303` are relaxed only when
every derivedFrom-like link on the requirement relaxes them); `coverage = false` keeps the checks but
gives no coverage credit. The built-in fields themselves are never relaxed.

## Custom fields (W041)

| Code | Severity | Condition |
|---|---|---|
| `W041` | warning | a `custom_fields` value is not a scalar or a list of scalars (e.g. a nested map); names the offending key |

## Naming (W042)

| Code | Severity | Condition |
|---|---|---|
| `W042` | warning | A qualified-name segment — an element's own name **or** a package/directory (namespace) name — is not a SysMLv2 **basic name** (`[A-Za-z_][A-Za-z0-9_]*`) and is not a stable id. Hyphens/spaces/punctuation are not allowed — rename using `_` or CamelCase. Such a name cannot be referenced in `appliesWhen`/`parameterConstraints` (where `-` is the subtraction operator). |

## Built-in types (W043, W044)

| Code | Severity | Condition |
|---|---|---|
| `W043` | warning | A type reference names a member of a **closed** auto-imported package (`ScalarValues`, `Base`) that the package does not declare (e.g. `ScalarValues::Flota`) — a likely typo; the message lists the known members. Recognised members resolve with no `W404`/`W043`. The **open** packages `ISQ`/`SI` are curated-recognised (clean) but lenient — an unrecognised `ISQ`/`SI` member is never flagged. |
| `W044` | warning | An element/feature declares both a recognised `ISQ` quantity type and a recognised `SI` unit whose physical **dimensions differ** (e.g. `ISQ::MassValue` + `unit: SI::metre`); names both dimensions. Lenient when either side is unrecognised. |

## Stereotypes — metadata applications (E317, E318, W045)

A stereotype is a `MetadataDef` applied via an element's `metadata:` field (SysMLv2 metadata, not UML).

| Code | Severity | Condition |
|---|---|---|
| `E317` | error | A `metadata:` application does not resolve to a `MetadataDef`. |
| `E318` | error | A `metadata:` application's `MetadataDef` declares `annotates:` that excludes the annotated element's type (abstract `Element`/`Definition`/`Usage` match; stdlib metadata recognised). |
| `W045` | warning | A tagged-value key in a `metadata:` application is not a declared feature of the `MetadataDef`. |

## Stable-ID prefixes and unknown fields (W046, W047)

| Code | Condition |
|---|---|
| `W046` | An `[ids.prefixes]` entry in `.syscribe.toml` is malformed: the key is not an id-identified element type, or a prefix does not match `^[A-Z][A-Z0-9]{1,11}$`. The offending entry/prefix is ignored; well-formed siblings still apply |
| `W047` | A top-level frontmatter key is not a recognised schema field (and is not `custom_fields:`) — likely a typo (`reqDomian`, `verifis`). One finding per key; move author-defined data under `custom_fields:` (§3.15). Gate with `--deny W047` |

## Suspect links (W090, ADR-SYS-SUSLINK-001)

| Code | Condition |
|---|---|
| `W090` | Suspect link: a trace-link target's normative content (body + normative frontmatter) changed since the source's `traceBaselines:` entry for it was captured. Review, then re-baseline with `suspect accept <src> <tgt>`. Unbaselined links are silent. Gate with `--deny W090` |

## Release baselines (E520–E522, W520, ADR-SYS-BASELINE-001)

| Code | Condition |
|---|---|
| `E520` | A `status: released` `Baseline`'s frozen scope has drifted — the recomputed aggregate content hash no longer matches its `seal` |
| `W520` | A `status: approved` `Baseline`'s frozen scope has drifted (the `approved` grade of `E520`; `draft` is silent, `superseded` is skipped) |
| `E521` | A `Baseline`'s `seal.aggregateHash` disagrees with its JSON manifest under `baselines/` — the seal was tampered with or the manifest is stale |
| `E522` | A `Baseline`'s `supersedes:` names a baseline that resolves to no element |

## Native SysML v2 submodel ingestion (W540–W542, ADR-SYS-SYSMLV2-001)

| Code | Condition |
|---|---|
| `W540` | A nested `_index.md` (or other stray `.md`) inside a `sysmlSubmodel:` subtree is ignored — nested files carry no namespace meaning there |
| `W541` | A `.sysml`/`.kerml` file in a `sysmlSubmodel:` subtree could not be read, or failed to parse as SysML v2/KerML; its content is skipped |
| `W542` | A `connect` endpoint's two-segment feature chain was truncated to a head-only edge because the tail is not a locally redeclared feature (REQ-TRS-SYSMLV2-015) |

## Foreign-format stdio plugins (E550, E551, W550–W553, ADR-SYS-PLUGIN-002)

| Code | Condition |
|---|---|
| `E550` | A `foreignFormat:` plugin's command was not found (the `[plugins.<alias>]` executable cannot be spawned) |
| `E551` | A package declares `foreignFormat: <alias>` but `.syscribe.toml` has no matching `[plugins.<alias>]` entry |
| `W550` | A plugin process failed to spawn, timed out (`timeout_ms`), exited non-zero, or hit an I/O error; the package contributes no elements |
| `W551` | A plugin's stdout is not a valid `{elements, diagnostics}` envelope, or the plugin reported one or more `diagnostics` in its envelope (relayed, previewed in the message) |
| `W552` | A plugin-emitted element was dropped because its frontmatter is not valid (bad qname or does not deserialize); sibling elements are kept |
| `W553` | A plugin-emitted element was dropped because its `type` is not a recognised element type |

## Annotated-source ingestion (E560, E561, W560–W563, ADR-SYS-ANNOTATE-001)

| Code | Condition |
|---|---|
| `E560` | A package declares `annotationFormat:` without a non-empty `marker:` regex and a non-empty `include:` glob list (or the `marker:` regex does not compile); no scanning happens |
| `E561` | A marker comment block is not valid YAML |
| `W560` | A marker block is valid YAML but not a legal element (does not deserialize into the frontmatter schema); the block is skipped |
| `W561` | A marker block has no content, no `type:` (or an unrecognised one), or no identity (`id:` or `name:`); the block is skipped |
| `W562` | `annotationFormat:` is set alongside `foreignFormat:`/`sysmlSubmodel:` on the same package; annotation scanning is skipped for that package |
| `W563` | Informational warning: an annotated element's `implementedBy:` was auto-filled from the marker's own source location (set `implementedBy:` explicitly to silence) |
