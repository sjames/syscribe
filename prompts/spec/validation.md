# Syscribe Validation Rule Codes

## Parse-time errors — core (E001–E015, E023–E025, E300–E304)

| Code | Condition |
|---|---|
| `E000` | Internal fallback for an unrecognised derive-pass finding code (should not appear in a healthy model) |
| `E001` | File does not begin with `---` (missing frontmatter delimiter) |
| `E002` | YAML frontmatter is not valid YAML 1.2 |
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
| `E310` | `Requirement` has `derivedFrom:` but no `breakdownAdr:` |
| `E311` | `breakdownAdr:` cannot be resolved or resolves to a non-`ADR` element |
| `E312` | A parent `Requirement` (has `derivedChildren`) appears in a `satisfies:` list |
| `E313` | `satisfies:` connects an architecture element and a requirement with incompatible `domain`/`reqDomain` |
| `E314` | `PartDef`/`Part` with `isDeploymentPackage: true` has no `Allocation` to a `hardware` element |
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
| `W005` | Native `Requirement` has neither `derivedFrom:` nor `derivedChildren` (possible orphan) |
| `W006` | Both `silLevel:` and `asilLevel:` set on the same element — incompatible standards |
| `W007` | Unrecognised frontmatter key (lenient mode; key preserved) |
| `W008` | Element has no `type:` field — it will be ignored by most commands |
| `W300` | Leaf `Requirement` at `approved`/`implemented` has no satisfying architecture element |
| `W301` | Leaf `Requirement` satisfied by more than one architecture element |
| `W302` | Leaf `Requirement` at `implemented`/`verified` still has `reqDomain: system` |
| `W303` | `breakdownAdr:` references an ADR with `status: proposed` |
| `W304` | `isDeploymentPackage: true` combined with `domain: hardware` |
| `W305` | Parent `Requirement` at `approved`/`implemented`/`verified` has no active `TestCase` at `testLevel: L3`–`L5` |
| `W306` | A high-integrity `Requirement` (`silLevel >= 4`/`asilLevel: D`) is not a fully integrated safety mechanism — draft, unsatisfied (leaf), or active in no `Configuration`. Gate with `--deny W306` |
| `W307` | A non-`draft` `UseCaseDef` carries no `refines:` link to a requirement (advisory, draft-suppressed; `--deny W307`) |

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

## Confirmation measures & DIA/CIA responsibility (E847–E851, W038, W039)

| Code | Condition |
|---|---|
| `E847` | `ConfirmationMeasure` missing `id`, `name`, or `status` |
| `E848` | `ConfirmationMeasure.id` does not match `CM-*` pattern |
| `E849` | `ConfirmationMeasure.measureType` not in `confirmation_review · functional_safety_audit · functional_safety_assessment · cybersecurity_assessment` |
| `E850` | `ConfirmationMeasure.independenceLevel` not in `I1 · I2 · I3` |
| `E851` | A `confirms:` ref does not resolve to any model element |
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
| `W034` | For an allocation target with ≥2 sources, a mixed-criticality source pair has no freedom-from-interference argument (`ffiRationale:` or `accepted` `breakdownAdr:`). Opt-in; gate with `--deny W034` |

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

## State machine warnings (W070–W080, §22.1)

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

## Allocation errors and structural warnings (E500–E503, W500–W503)

| Code | Condition |
|---|---|
| `E500` | A feature with `type: Allocation` has an `allocatedFrom:` that does not resolve |
| `E501` | A feature with `type: Allocation` has an `allocatedTo:` that does not resolve |
| `E502` | An `allocatedFrom:` entry (any element) does not resolve to a known element |
| `E503` | An `allocatedTo:` entry (any element) does not resolve to a known element |
| `W500` | `viewpoint:` on a View does not resolve to a `ViewpointDef` |
| `W501` | `exhibitsStates:` entry does not resolve to any known element |
| `W502` | `expose:` entry on a View does not resolve to any known element |
| `W503` | The same allocation edge is declared by both an `allocatedTo:` and an `Allocation` element (redundant) |

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

## IEC 62443 Zone/Conduit (E950–E956, W950–W953, §13)

| Code | Condition |
|---|---|
| `E950` | `Zone` missing `id`/`name`/`status`/`targetSL` |
| `E951` | `Zone.id` not a `ZN-*` id |
| `E952` | `Conduit` missing `id`/`name`/`status`/`fromZone`/`toZone` |
| `E953` | `Conduit.id` not a `CD-*` id |
| `E954` | `Conduit.fromZone`/`toZone` unresolved or not a `Zone` |
| `E955` | `Zone.members:` entry unresolved or not a `PartDef`/`Part` |
| `E956` | `PartDef`/`Part.inZone:` unresolved or not a `Zone` |
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
| `E512` | A cross-repo `verifies`/`derivedFrom`/`allocatedTo` reference resolves in neither the local model nor any loaded repo |
| `E513` | `repoImports[].repo` names an alias not present in `[repos]` |
| `E514` | `repoImports[].qname` does not resolve to any element in the named repo |
| `E515` | Two repos export the same stable ID (the id namespace is global) |
| `W510` | A repo in `[repos]` has no `ref:` — composition is not pinned (opt-in; `--deny W510`) |
| `W511` | A peer repo's git `HEAD` has drifted from its configured `ref:` (opt-in; `--deny W511`) |
| `W512` | A peer submodule's gitlink disagrees with its configured `ref:` (opt-in; `--deny W512`) |

## Documentation linting (W099–W102, `lint-docs`)

The `lint-docs` command scans external `.md`/`.svg` docs for references that no longer resolve.

| Code | Condition |
|---|---|
| `W099` | An unresolvable stable-ID token (`REQ-*`/`TC-*`/…) in prose |
| `W100` | A qualified name inside a ` ```mermaid ` block that does not resolve |
| `W101` | An SVG `sysml:ref="…"` that does not resolve |
| `W102` | A local image/diagram embed path that does not exist (remote URIs accepted) |

## §12.8 Implementation trace (W029)

| Code | Condition |
|---|---|
| `W029` | A non-`draft` requirement with an integrity level declares a `wcet:` claim but no active measuring `TestCase` verifies it (timing analog of `W702`; `--deny W029`) |

## Tier 4 — Fault Tree Analysis (E900–E909, W900–W901, W926, W927)

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
| `W900` | `FaultTree` has no gates or events (tree is empty) |
| `W901` | `FaultTreeGate` has no `inputs` |
| `W926` | `FaultTreeEvent.fmeaRef` does not resolve to a known `FMEAEntry` (FTA↔FMEA cross-link) |
| `W927` | `FMEAEntry.ftaRef` does not resolve to a known `FaultTreeEvent` (FMEA↔FTA cross-link) |

## Tier 4 — FMEA (E911–E914, E922, W902–W904)

| Code | Condition |
|---|---|
| `E911` | `FMEASheet` missing `id`, `name`, or `status` |
| `E912` | `FMEASheet.id` does not match `FMEA-*` |
| `E913` | FMEAEntry `id` does not match `FM-*` |
| `E914` | `fmeaSeverity`, `occurrence`, or `detection` outside 1–10 |
| `E922` | An `entries:` row contains an unrecognised key (silent data loss in a safety analysis — error) |
| `W902` | `FMEASheet` has no `entries` |
| `W903` | Computed RPN > 100 and no `recommendedAction` set. RPN is auto-computed as `fmeaSeverity × occurrence × detection` when `rpn:` is absent |
| `W904` | Entry `ref` does not resolve to a known model element |

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

## Product Line Engineering errors (E200–E230)

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

## Product Line Engineering warnings (W010–W017)

| Code | Condition |
|---|---|
| `W010` | `Configuration` does not bind a parameter declared `isRequired: true` on a selected feature |
| `W011` | `FeatureDef` with `groupKind: optional` selected in zero `Configuration` files |
| `W012` | `FeatureDef` with `groupKind: optional` selected in every `Configuration` |
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
| `W027` | (`validate`) a `Configuration` binds a parameter whose `bindingTime: runtime` (resolved by the running system, not at configuration time); gate with `--deny W027` |
| `W023` | (§12.8) a non-`draft` `Part`/`PartDef`/`Interface`/`InterfaceDef` has an `implementedBy:` path that does not exist on disk. Opt-in (only when `implementedBy:` is present); draft-suppressed; remote (`scheme://`) targets accepted as external and not checked. Path resolution matches `sourceFile`. Gate with `--deny W023`. |

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
