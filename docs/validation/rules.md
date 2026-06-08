# Rule Reference

`VALIDATION · RULES`

Warnings are advisory by default (exit `0`). Promote them to CI gate failures (exit `2`) with `validate --deny <CODES>` / `--max-warnings <N>` / `--warnings-as-errors`, or with a named, SIL/ASIL-scopable `validate --profile <name>` policy declared in `.syscribe.toml` — see [CI severity gating](../cli/index.md#ci-severity-gating). Errors always exit `1`.

## Parse-time errors (E001–E022)

| Code | Element | Condition |
|---|---|---|
| E004 | TestCase | `id`, `title`, `status`, or `testLevel` absent |
| E004 | Requirement | `title` or `status` absent on native Requirement |
| E006 | Requirement | `id` present but does not match `REQ-*` pattern |
| E006 | TestCase | `id` present but does not match `TC-*` pattern |
| E007 | Requirement | `status` is not one of `draft · review · approved · implemented · verified` |
| E007 | TestCase | `status` is not one of `draft · review · approved · active · retired` |
| E008 | TestCase | `testLevel` is not one of `L1 · L2 · L3 · L4 · L5` |
| E009 | Any | `silLevel` is not in range 1–4 |
| E010 | Any | `asilLevel` is not one of `A · B · C · D` |
| E011 | TestCase | Body has no ` ```gherkin ` fenced block |
| E012 | Requirement | Normative text (before first `##`) is empty |
| E013 | TestCase | `verifies:` is absent or empty |
| E014 | TestCase | `Scenario Outline:` block has no `Examples:` table |
| E015 | TestCase | First ` ```gherkin ` block has no `Feature:` line |
| E019 | Any | `dalLevel` is not one of `A · B · C · D · E` (DO-178C) |
| E020 | Any | `verificationMethod` is not one of `test · inspection · analysis · demonstration` |
| E021 | Any | `coverageTarget` is not one of `statement · branch · MCDC` |
| E022 | Any | `requirementKind` is not one of `stakeholder · system · software · hardware` |

## Parse-time warnings (W001–W008)

| Code | Condition |
|---|---|
| W001 | Requirement normative text contains no `shall` |
| W004 | `sourceFile:` path does not exist on disk |
| W006 | Both `silLevel` (IEC 61508) and `asilLevel` (ISO 26262) are set on the same element — incompatible standards; use only one |
| W007 | Type definition (e.g. `PartDef`) is never referenced as a supertype or type |
| W008 | Element has no `type:` field — will be ignored by most commands |

## Cross-reference errors (E101–E106)

| Code | Condition |
|---|---|
| E101 | Duplicate `id` across two elements |
| E102 | `verifies:` entry does not resolve |
| E103 | `derivedFrom:` entry does not resolve |
| E104 | `verifies:` target is not a native Requirement |
| E105 | `derivedFrom:` target is not a native Requirement |
| E106 | `testFunctions[].scenario` name not found in Gherkin blocks |

## Coverage warnings (W002–W005)

| Code | Condition |
|---|---|
| W002 | Requirement at `approved` or `implemented` has no active TestCase |
| W003 | Requirement at `verified` has no active TestCase covering it |
| W005 | Requirement has no `derivedFrom` and no `derivedChildren` — possible orphan |

## Cycle detection errors (E016–E018, E107)

| Code | Condition |
|---|---|
| E016 | Cycle detected in `supertype:` graph |
| E017 | Cycle detected in `derivedFrom:` graph |
| E018 | Cycle detected in `subsets:` graph |
| E107 | Cycle detected in `typedBy:` graph — **including a self-reference** (a usage typed by itself). Structural cycle error, not a name-resolution error, so it is **not** suppressed under `--config`. |

## PLE errors (E200–E209)

> The parameter-binding rules below (`E203`–`E206`, `E222`, `W017`) are enforced by **both** `validate` and `feature-check`, so a product line checked holistically also gets range/binding enforcement (GH #14).

| Code | Condition |
|---|---|
| E200 | Configuration `id` does not match `CONF-*` pattern |
| E201 | Configuration missing `id`, `title`, `status`, or `featureModel` |
| E203 | `parameterBindings` binds a parameter of a feature that is not selected |
| E204 | `parameterBindings` binds a fixed parameter (`isFixed`/`value`/`derivedFrom`) |
| E205 | A bound parameter value is outside the parameter's `range:` |
| E206 | A bound parameter value is not in the parameter's `enumValues:` |
| E209 | `appliesWhen:` is malformed, or an operand does not resolve to a FeatureDef. `appliesWhen:` accepts a bare QName, a list (AND), or a boolean expression (`and`/`or`/`not`/parentheses); every operand is checked. |
| E222 | A `parameterBindings` key does not resolve to a declared `FeatureDef` parameter (bad path — including the legacy all-`::` member form `Features::Feature::param`, which must be the dotted `Features::Feature.param` — unknown feature, or undeclared parameter) |
| E230 | A parameter declares a `bindingTime:` value other than `compile`/`load`/`runtime` (§9.7) |

## PLE warnings (W015–W017)

The variability dimension is **opt-in**: it is dormant — and these checks do not fire — unless the model has at least one `FeatureDef` and something linking to it (a `Configuration`, or any element/`TestCase` with `appliesWhen:`).

| Code | Condition |
|---|---|
| W015 | A requirement is **active** in a `Configuration` (its `appliesWhen:` holds for that configuration's selections) but no non-draft `TestCase` that runs in that `Configuration` verifies it. Draft requirements and draft tests are suppressed. Gate it in CI with `--deny W015`. |
| W016 | A `Configuration` parsed **zero** feature selections while a feature model exists — e.g. it used a legacy/unrecognized `selections:` key instead of the `features:` map (§9.8). Without this warning the block is silently ignored and every cell in `matrix` comes back N/A. Not emitted when no `FeatureDef` is present. |
| W017 | A selected feature declares a required parameter (`isRequired: true`, not fixed, no `default:`) that the `Configuration` does not bind. (§9.11 names this `W010`, which this tool already uses for test-result ingestion.) **Suppressed** for a parameter whose `bindingTime: runtime` — the running system supplies its value. |
| W027 | A `Configuration` binds a parameter whose `bindingTime: runtime` (resolved by the running system, not at configuration time). Gate with `--deny W027`. |

A `TestCase` *runs in* a `Configuration` iff its `appliesWhen:` is satisfied by that configuration's `features:` selections; a `TestCase` with no `appliesWhen:` runs in every configuration. The same relationship powers `syscribe matrix`.

## Feature-model check (`feature-check` command)

These holistic feature-model rules are **not** run by `validate` — they are emitted only by the explicit `syscribe feature-check` command (exit `0` if no errors, `1` otherwise; dormant with a notice when no `FeatureDef` is present).

| Code | Condition |
|---|---|
| E212 | A `FeatureDef.requires:`/`excludes:` entry does not resolve to a `FeatureDef` |
| E219 | In some `Configuration`, a selected feature's `requires:` target is not selected |
| E220 | In some `Configuration`, a selected feature's `excludes:` target is also selected |
| E207 | Circular `derivedFrom:` dependency among parameters of the same `FeatureDef` |
| E202 | A value propagated via `bindTo:` falls outside the component parameter's narrowing `range:` |
| E229 | A parameter's `bindingTime:` is earlier than that of a `derivedFrom`/`bindTo` source it depends on — an impossible ordering (checked only when both ends declare a `bindingTime:`) |
| E213 | A `parameterConstraints` expression references an unresolved parameter path, **or** uses the legacy `::`-member form (`Features::F::param`) instead of the canonical dotted `Features::F.param` — the constraint is flagged, never silently dropped |
| E221 | A `parameterConstraints` expression evaluates to **false** for a `Configuration` whose `appliesWhen:` holds (numeric comparison/arithmetic over dotted parameter references; default severity) |
| W011 | An `optional` `FeatureDef` is selected in zero `Configuration` files (possible dead feature) |
| W012 | An `optional` `FeatureDef` is selected in every `Configuration` (consider `mandatory`) |
| W014 | A `parameterConstraints` `appliesWhen:` references a feature selected in no `Configuration` |
| W024 | An **orphan** `FeatureDef` — referenced by no element's `appliesWhen:` and selected `true` by no `Configuration` (it gates nothing and ships in nothing). Gate with `feature-check --deny W024` |
| W025 | A `parameterConstraints` violation (as `E221`) where the constraint declares `severity: warning`. Gate with `feature-check --deny W025` |

### Deep analysis (`feature-check --deep`)

`--deep` adds SAT-backed analysis over a propositional encoding of the feature model (Boolean layer only; deterministic; engine is batsat (pure-Rust CDCL, no external process) — see `ADR-FM-002`). It guards against blow-up by skipping (with a notice) above a feature-count limit.

| Code | Condition |
|---|---|
| E223 | The feature model is **void** — no valid configuration exists. Reported once with a sound conflict-set explanation; void dominates (no per-feature dead spam) |
| E224 | A **dead feature** — selectable in no valid configuration — with an explanation of the cause |
| E225 | An authored `Configuration` is **not a valid model** of the feature model (mandatory/group/cardinality/parent-selection violation); `requires`/`excludes` violations remain `E219`/`E220` and are not duplicated |
| W018 | A **false-optional** feature — declared `optional` but forced selected whenever its parent is |

Core features (present in every valid configuration) are reported informationally in the `--json` `coreFeatures` list and the text summary. Not implemented: configuration counting and numeric/parameter (SMT) reasoning.

Not yet implemented (specified): group-cardinality rules (`E216`/`E217`/`E218`) and two-level satisfies completeness (`E210`/`E211`).

## Configuration projection (the `--config` lens, ADR-PROJ-001)

`validate --config <C>` projects the 150% model onto a configuration (or ad-hoc feature set) and re-validates that variant; `feature-check --deep` proves variability integrity across all variants; `validate --all-configs` gates every stored configuration.

| Code | Condition |
|---|---|
| E226 | (`--config`) an **active** element's **structural** reference (typedBy/supertype/subsets/redefines/allocation) escapes the variant — the target is inactive in this configuration |
| W019 | (`--config`) an **active** element's **traceability** reference (verifies/satisfies/derivedFrom/breakdownAdr) escapes the variant |
| E227 | (`feature-check --deep`) a **structural** edge is provably violable: a valid configuration activates the source without its target (`appliesWhen(X) ⇒ appliesWhen(Y)` fails); includes a witness |
| W020 | (`feature-check --deep`) a **traceability** edge is provably violable across some valid configuration |
| W021 | (`feature-check --deep`) a **dead element** — its `appliesWhen` is unsatisfiable under the feature model (active in no valid configuration) |
| W022 | (`feature-check --deep`) a requirement **active in some configuration but covered in none** (family-wide coverage gap) |

The lens is inert when the model declares no `FeatureDef`. Cross-reference-resolution codes (`E102`–`E106`) are suppressed under `--config` because escaping refs (`E226`/`W019`) are authoritative there.

## Transitive package `appliesWhen` (§9.10, REQ-TRS-VAR-006)

A `Package` may declare `appliesWhen:` to gate its whole subtree; an element's **effective condition** is its own `appliesWhen:`, else the nearest ancestor package's. To keep that unambiguous, at most one node per root-to-leaf path may declare it. Checked by `validate` (dormant without a `FeatureDef`).

| Code | Condition |
|---|---|
| E228 | Invalid `appliesWhen:` placement: nested under a package that already declares one; or on a `FeatureDef`/`Configuration`, a package whose subtree contains one, or the model-root package |
| W026 | A `Package` declares `appliesWhen:` but gates no projectable element (empty subtree); gate with `--deny W026` |

## ADR errors (E300–E304)

| Code | Condition |
|---|---|
| E300 | ADR `id` does not match `ADR-*` pattern |
| E301 | ADR missing `id`, `title`, or `status` |
| E302 | `reqDomain` value is not `system · hardware · software` |
| E303 | `domain` value is not `system · hardware · software` |
| E304 | ADR `status` is not `proposed · accepted · deprecated · superseded` |

## Traceability warnings (W300–W306)

| Code | Condition |
|---|---|
| W300 | Leaf Requirement at `approved` or `implemented` has no satisfying element |
| W301 | Leaf Requirement is satisfied by more than one element |
| W302 | Leaf Requirement at `implemented` or `verified` still has `reqDomain: system` |
| W303 | `breakdownAdr:` references a `proposed` ADR but Requirement is `approved` or higher |
| W304 | `isDeploymentPackage: true` combined with `domain: hardware` |
| W305 | Parent Requirement (has `derivedFrom` children) at `approved`, `implemented`, or `verified` has no active TestCase at `testLevel: L3`, `L4`, or `L5` — leaf-level tests on derived requirements are not sufficient to verify emergent composed behaviour |
| W306 | **Unsatisfied safety mechanism** — a high-integrity Requirement (`silLevel >= 4` or `asilLevel: D`) that is **not** a fully integrated safety mechanism: `status: draft`, **or** no element satisfies it, **or** (with a feature model) active in no `Configuration`. The message names the triggering sub-condition(s). Gate with `--deny W306`. (Threshold/sub-condition tuning rides with severity profiles, #18.) |

## §12 Traceability errors (E310–E315)

| Code | Condition |
|---|---|
| E310 | Requirement has `derivedFrom:` but no `breakdownAdr:` |
| E311 | `breakdownAdr:` does not resolve, or does not resolve to an ADR |
| E312 | Parent requirement (has `derivedChildren`) appears in a `satisfies:` list |
| E313 | `satisfies` domain mismatch: element domain ≠ requirement `reqDomain` |
| E314 | `isDeploymentPackage: true` element has no Allocation to a hardware element |
| E315 | Cross-domain `supertype:` or `typedBy:` reference — use Allocation instead |

## §12.8 Implementation trace (W023)

The optional `implementedBy:` field on a `Part`/`PartDef` records the source artifact(s) that realise an architecture element — the downstream leg of the V-model (`Requirement ─satisfies→ Architecture ─implementedBy→ Code ─verifies→ Test`).

| Code | Condition |
|---|---|
| W023 | A non-`draft` `Part`/`PartDef` has an `implementedBy:` path that does not exist on disk |

- **Opt-in** — the check runs only when `implementedBy:` is present; elements without it are never flagged.
- **Draft-suppressed** — elements with `status: draft` are skipped (the implementation may not exist yet).
- **Path resolution** is identical to `sourceFile` (`classify_source`): model-/repo-relative, `model:`/`repo:` prefixes, absolute, and `file://` paths are checked on disk; remote URIs (`scheme://`) are accepted as external pointers and not verified locally. `implementedBy:` accepts a single string or a list; each entry is checked independently.
- **Gateable** — `validate --deny W023` exits non-zero when any W023 is present.

## §3 External references (W028)

The optional common field `extRef:` (string or list) marks an element as the representation of an artifact in another tool — a requirement in DOORS Next, an element in a SysML tool, a ticket. Look up the element(s) carrying a reference with `syscribe -m <root> extref <ref>`.

| Code | Condition |
|---|---|
| W028 | The same `extRef` value is declared by two or more elements (one finding per duplicated value) |

- **Opt-in** — the check runs only when some element declares `extRef:`.
- **Allowed but flagged** — duplicates are permitted (`extref` returns all matches), but usually signal a stray copy or bad merge.
- **Not a cross-reference** — `extRef` is an external pointer; it is never a target for `supertype:`/`verifies:`/`derivedFrom:` etc.
- **Gateable** — `validate --deny W028` exits non-zero when any W028 is present.

## Diagram errors (E400–E402)

| Code | Condition |
|---|---|
| E400 | `diagramKind: Mermaid` but body has no ` ```mermaid ` block |
| E401 | `diagramKind: PlantUML` but body has no ` ```plantuml ` block |
| E402 | `svgFile:` path does not exist on disk |

## Diagram warnings (W400–W412)

| Code | Condition |
|---|---|
| W400 | Diagram has no `diagramKind` — rendering mode ambiguous |
| W401 | `subject:` does not resolve to a known element |
| W402 | Shape `ref:` does not resolve (and is not a sub-feature of a known element) |
| W403 | Edge `source` or `target` is not a defined shape id in this diagram |
| W404 | Operation `typedBy` (parameter) or `returnType` does not resolve to a known element |
| W405 | SVG companion file is referenced by both inline and companion modes simultaneously |
| W406 | Frontmatter `shapes`/`edges` id has no matching `id="..."` attribute in the inline SVG block |
| W407 | SVG element `id` has no matching entry in frontmatter `shapes`/`edges` (SVG-internal ids used via `url(#...)` are excluded) |
| W408 | Mermaid `%% ref:` annotation does not resolve to a known element |
| W409 | Mermaid diagram has no `%% ref:` annotations — add at least one to link nodes to model elements |
| W410 | Mermaid `%% link:` annotation does not resolve to a known element |
| W411 | Shape `link:` value does not resolve to a known element |
| W412 | SVG `href="..."` attribute does not resolve to any model element file |

## Allocation errors (E500–E503)

| Code | Condition |
|---|---|
| E500 | Feature with `type: Allocation` has `allocatedFrom:` that does not resolve |
| E501 | Feature with `type: Allocation` has `allocatedTo:` that does not resolve |
| E502 | `allocatedFrom:` entry (on any element) does not resolve to a known element |
| E503 | `allocatedTo:` entry (on any element) does not resolve to a known element |

## Structural warnings (W500–W502)

| Code | Condition |
|---|---|
| W500 | `viewpoint:` on View does not resolve to a ViewpointDef |
| W501 | `exhibitsStates:` entry does not resolve to any known element |
| W502 | `expose:` entry on View does not resolve to any known element |

## Documentation warnings (W600–W601)

| Code | Condition |
|---|---|
| W600 | PartDef or Part has an empty documentation body |
| W601 | ActionDef or Action has an empty documentation body |

## Safety / ASPICE warnings (W701–W703, W807, W029)

These warnings apply to requirements carrying safety integrity level fields (`asilLevel`, `silLevel`, `dalLevel`).

| Code | Condition |
|---|---|
| W701 | Requirement with `asilLevel: B`, `C`, or `D` has no `verificationMethod` — add `test`, `inspection`, `analysis`, or `demonstration` |
| W702 | Requirement with `asilLevel: D` has no active TestCase at `testLevel: L5` (HIL) — ISO 26262-6 §9 requires hardware-in-the-loop testing for ASIL D |
| W703 | Both `asilLevel` (ISO 26262) and `dalLevel` (DO-178C) are set on the same element — these are different standards; pick one or document the mapping |
| W807 | `Requirement` with `derivedFromSecurityGoal` has no `verificationMethod` — security-derived requirements must specify how they will be tested or inspected |
| W029 | Non-draft requirement with an integrity level (`silLevel`/`asilLevel`) declares a `wcet:` claim but no **active measuring** TestCase verifies it (testLevel `L5`, or tagged `timing`/`wcet`). The timing-evidence analog of `W702`. Gate with `--deny W029`. Query timing claims with `list --has-wcet`. |

## Tier 2 safety element errors (E800–E830)

Tier 2 element types support ISO 26262 HARA and ISO/SAE 21434 TARA workflows. Each type carries a stable opaque ID and required fields validated at parse time.

### HazardousEvent (E800–E804, E833–E836)

| Code | Condition |
|---|---|
| E800 | `id`, `title`, or `status` is absent |
| E801 | `severity` is not one of `S0 · S1 · S2 · S3` (ISO 26262 HARA) |
| E802 | `exposure` is not one of `E0 · E1 · E2 · E3 · E4` (ISO 26262 HARA) |
| E803 | `controllability` is not one of `C0 · C1 · C2 · C3` (ISO 26262 HARA) |
| E804 | `id` does not match `HE-*` pattern |
| E833 | `consequence` is not one of `Ca · Cb · Cc · Cd` (IEC 61508 risk graph) |
| E834 | `freqExposure` is not one of `Fa · Fb` (IEC 61508 risk graph) |
| E835 | `avoidance` is not one of `Pa · Pb` (IEC 61508 risk graph) |
| E836 | `demandRate` is not one of `W1 · W2 · W3` (IEC 61508 risk graph) |

### SafetyGoal (E805–E806, E837, W801, W806)

| Code | Condition |
|---|---|
| E805 | `id`, `title`, or `status` is absent |
| E806 | `id` does not match `SG-*` pattern |
| E837 | `plLevel` is not one of `a · b · c · d · e` (ISO 13849-1) |
| W801 | SafetyGoal has no integrity level — set `asilLevel` (ISO 26262), `silLevel` (IEC 61508), or `plLevel` (ISO 13849-1) |
| W806 | SafetyGoal has no `hazardousEvents` — it is not grounded in any hazard analysis |

### DamageScenario (E807–E810)

| Code | Condition |
|---|---|
| E807 | `id`, `title`, or `status` is absent |
| E808 | `id` does not match `DS-*` pattern |
| E809 | `damageSeverity` is not one of `severe · major · moderate · negligible` |
| E810 | `impactCategories` entry is not one of `safety · financial · operational · privacy` |
| E844 | `hazardRef` does not resolve, or resolves to an element that is not a `HazardousEvent`/`SafetyGoal` (safety↔security co-engineering) |

### ThreatScenario (E811–E814, E845)

| Code | Condition |
|---|---|
| E811 | `id`, `title`, or `status` is absent |
| E812 | `id` does not match `TS-*` pattern |
| E813 | `attackFeasibility` is not one of `high · medium · low · very_low` |
| E814 | `attackVector` is not one of `network · adjacent · local · physical` |
| E845 | `riskTreatment` is not one of `avoid · reduce · share · retain` |
| E844 | `hazardRef` does not resolve, or resolves to an element that is not a `HazardousEvent`/`SafetyGoal` (safety↔security co-engineering) |

### CybersecurityGoal (E815–E818)

| Code | Condition |
|---|---|
| E815 | `id`, `title`, or `status` is absent |
| E816 | `id` does not match `CSG-*` pattern |
| E817 | `securityProperty` is not one of `confidentiality · integrity · availability · authenticity` |
| E818 | `calLevel` is not one of `CAL1 · CAL2 · CAL3 · CAL4` |

### SecurityControl (E819–E821)

| Code | Condition |
|---|---|
| E819 | `id`, `title`, or `status` is absent |
| E820 | `id` does not match `SC-*` pattern |
| E821 | `controlType` is not one of `prevention · detection · response · recovery` |

### VulnerabilityReport (E822–E824, W803)

| Code | Condition |
|---|---|
| E822 | `id`, `title`, or `status` is absent |
| E823 | `id` does not match `VR-*` pattern |
| E824 | `cvssScore` is outside range 0.0–10.0 |

## Tier 2 cross-reference errors (E825–E832)

| Code | Condition |
|---|---|
| E825 | `SafetyGoal.hazardousEvents` entry does not resolve to a HazardousEvent |
| E826 | `ThreatScenario.damageScenarios` entry does not resolve to a DamageScenario |
| E827 | `CybersecurityGoal.threatScenarios` entry does not resolve to a ThreatScenario |
| E828 | `SecurityControl.implementsGoals` entry does not resolve to a CybersecurityGoal |
| E829 | `VulnerabilityReport.mitigatedBy` entry does not resolve to a SecurityControl |
| E830 | `VulnerabilityReport.affectedElements` entry does not resolve to any known element |
| E831 | `derivedFromSecurityGoal` does not resolve, or does not resolve to a `CybersecurityGoal` |
| E832 | `derivedFromSafetyGoal` does not resolve, or does not resolve to a `SafetyGoal` |

## Tier 2 coverage and traceability warnings (W800–W808)

| Code | Condition |
|---|---|
| W800 | HazardousEvent is not referenced by any `SafetyGoal.hazardousEvents` |
| W802 | CybersecurityGoal is not implemented by any `SecurityControl.implementsGoals` |
| W803 | VulnerabilityReport has `status: open` — ensure it is being tracked and mitigated |
| W804 | CybersecurityGoal has no `Requirement` with `derivedFromSecurityGoal` pointing to it |
| W805 | SafetyGoal has no `Requirement` with `derivedFromSafetyGoal` pointing to it |
| W806 | SafetyGoal has no `hazardousEvents` — not grounded in any hazard analysis |
| W807 | `Requirement` with `derivedFromSecurityGoal` has no `verificationMethod` |

## Safety↔security co-engineering (E844, W030)

ISO 26262 ⇄ ISO/SAE 21434 cross-domain checks. A `DamageScenario`/`ThreatScenario` may declare `hazardRef:` (string or list) pointing to the `HazardousEvent`/`SafetyGoal` it endangers. See `docs/model-guide/safety-analysis.md` and `syscribe -m <root> co-analysis`.

| Code | Severity | Condition |
|---|---|---|
| E844 | Error | A `hazardRef` value on a `DamageScenario`/`ThreatScenario` does not resolve, or resolves to an element that is not a `HazardousEvent`/`SafetyGoal` |
| W030 | Warning | A `DamageScenario` whose `impactCategories` includes `safety` has no `hazardRef` (the cross-domain gap). Opt-in (safety-tagged only); gate with `--deny W030` |

## Cybersecurity risk determination (E845, W031, W032)

ISO/SAE 21434 §15.8 / §15.9 risk determination and treatment. Per `ThreatScenario`, risk is computed from severity and feasibility:

- **severity rank** — `negligible`=0, `moderate`=1, `major`=2, `severe`=3 — the **max** `damageSeverity` over the threat's resolved `damageScenarios`.
- **feasibility rank** — `very_low`=0, `low`=1, `medium`=2, `high`=3 (from `attackFeasibility`).
- if either rank is **unknown**, the risk is **unknown** (the threat is listed but never gated).
- else `score = severity + feasibility` (0..6) → **level**: `0–1` low, `2–3` medium, `4` high, `5–6` critical.

`ThreatScenario` carries `riskTreatment:` (`avoid`/`reduce`/`share`/`retain`; invalid → E845) and a free-text `residualRisk:`. See `docs/model-guide/safety-analysis.md` and `syscribe -m <root> cyber-risk`.

| Code | Severity | Condition |
|---|---|---|
| E845 | Error | `ThreatScenario.riskTreatment` is not one of `avoid · reduce · share · retain` |
| W031 | Warning | A `ThreatScenario` whose computed risk is `high`/`critical` has no `riskTreatment` and is not addressed by any `CybersecurityGoal` (no `CybersecurityGoal.threatScenarios` lists it). Gate with `--deny W031`; promotable via `[profiles]` |
| W032 | Warning | A `CybersecurityGoal`'s `calLevel` rank is below the expected minimum CAL for the max risk over its listed threats (low→CAL1, medium→CAL2, high→CAL3, critical→CAL4). Fires only when at least one linked threat has a computable risk. Gate with `--deny W032` |

## Quantitative HW safety metrics (E846, W033)

ISO 26262-5 §8–9 hardware architectural metrics, rolled up per `SafetyGoal` from its
contributing `FaultTreeEvent`s — the events under the `FaultTree`(s) whose `topEvent`
resolves to the goal. Each event may carry `diagnosticCoverage:` (DC, 0.0–1.0) and
`latentDiagnosticCoverage:` (DCl, 0.0–1.0). Over the events that declare a `failureRate`
(λ, /h):

```
Σλ      = Σ λ_i
λ_RF    = Σ λ_i · (1 − DC_i)              DC_i defaults to 0 when absent
SPFM    = 1 − λ_RF / Σλ                    (Σλ = 0 → undefined / n/a)
λ_MPFL  = Σ λ_i · DC_i · (1 − DCl_i)       over events that DECLARE DCl
LFM     = 1 − λ_MPFL / (Σλ − λ_RF)         (n/a unless ≥1 event sets DCl)
PMHF    = λ_RF + λ_MPFL                     (/h)
```

**Opt-in.** A goal's metrics are computed and gated **only** if at least one contributing
event declares `diagnosticCoverage`; otherwise they are `n/a` and never gated. This keeps
models without coverage data silent (zero W033). Targets by ASIL: SPFM ≥ {B 0.90, C 0.97,
D 0.99}; LFM ≥ {B 0.60, C 0.80, D 0.90}; PMHF < {B/C 1e-7, D 1e-8} /h (ASIL A: not gated).
SIL-only goals gate PMHF/PFH < {SIL2 1e-6, SIL3 1e-7, SIL4 1e-8} /h; SPFM/LFM are reported
but not gated for SIL. This is a **first-order FMEDA approximation** — verify independently.
See `docs/model-guide/safety-analysis.md` and `syscribe -m <root> metrics`.

| Code | Severity | Condition |
|---|---|---|
| E846 | Error | `diagnosticCoverage` or `latentDiagnosticCoverage` is outside `0.0`–`1.0` |
| W033 | Warning | A `SafetyGoal` with diagnosticCoverage data has a computed SPFM, LFM, or PMHF below/above its ASIL/SIL target (one finding naming the metric(s) and actual vs target). Gate with `--deny W033`; promotable via `[profiles]` |

## Freedom From Interference / dependent-failure analysis (W034)

ISO 26262-9 §7 dependent-failure analysis. Two elements **share a resource** when both are
**allocated to the same target element**. The tool collects allocation edges `(source → target)`
from every form — an element's `allocatedTo: [T, …]` (source = the element), an element's
`allocatedFrom: [S, …]` (target = the element), and an `Allocation` element's
`allocatedFrom`/`allocatedTo` (source → target) — resolving every reference via the `Resolver`,
then inverts them into a `target → { sources }` map.

Each element gets an **integrity tag**: `asilLevel` if present, else `silLevel` (→ `SIL<n>`),
else `QM`. Two sources on the same target are **mixed-criticality** when their tags differ
(including classified vs `QM`). A mixed pair is **excused** when the **target** OR **at least
one** of the two sources declares a non-empty `ffiRationale:` string, OR carries a
`breakdownAdr:` that resolves to an `accepted` ADR.

**Opt-in.** The whole check is dormant unless at least one element in the model declares
`asilLevel` or `silLevel`; a non-safety model emits zero W034 and unchanged exit codes.

> Deferred: the issue's cross-domain "attack surface" bonus (reusing the shared resources for
> the cybersecurity co-analysis view) is not implemented.

| Code | Severity | Condition |
|---|---|---|
| W034 | Warning | For an allocation target with ≥2 sources, a mixed-criticality source pair has no freedom-from-interference argument. One finding per offending `(target, sourceA, sourceB)`, naming both sources and their integrity tags. Gate with `--deny W034`; promotable via `[profiles]` |

See `docs/model-guide/safety-analysis.md`.

## Integrity level propagation errors and warnings (E841–E843, W808)

Once any element in the traceability chain carries `asilLevel` or `silLevel`, all downstream elements must inherit the same field. A lower level is permitted only when accompanied by a `breakdownAdr` documenting the ASIL/SIL decomposition rationale (ISO 26262-9 / IEC 61508-2 §7.4.9).

| Code | Severity | Condition |
|---|---|---|
| E841 | Error | Element with `derivedFromSafetyGoal` is missing `asilLevel`/`silLevel` when the referenced SafetyGoal carries one |
| E842 | Error | Element with `derivedFrom` is missing `asilLevel`/`silLevel` when the parent element carries one |
| E843 | Error | Element with `satisfies` is missing `asilLevel`/`silLevel` when the satisfied requirement carries one |
| W808 | Warning | Element's integrity level is strictly lower than its source (`derivedFromSafetyGoal`, `derivedFrom`, or `satisfies`) but no `breakdownAdr` is set |

## Tier 4 — Fault Tree Analysis (E900–E910, W900–W901)

### FaultTree (E900–E902, W900)

| Code | Condition |
|---|---|
| E900 | `id`, `title`, `status`, or `topEvent` is absent |
| E901 | `id` does not match `FT-*` pattern |
| E902 | `topEvent` does not resolve, or resolves to an element that is not a `SafetyGoal` |
| W900 | FaultTree has no gates or events — the tree is empty |

### FaultTreeGate (E903–E906, W901)

| Code | Condition |
|---|---|
| E903 | `id`, `title`, or `gateType` is absent |
| E904 | `id` does not match `FTG-*` pattern |
| E905 | `gateType` is not one of `AND · OR · XOR · NOT · inhibit` |
| E906 | An entry in `inputs` does not resolve, or resolves to an element that is not a `FaultTreeGate` or `FaultTreeEvent` |
| W901 | FaultTreeGate has no `inputs` — it contributes nothing to the fault tree |

### FaultTreeEvent (E907–E909)

| Code | Condition |
|---|---|
| E907 | `id`, `title`, or `eventKind` is absent |
| E908 | `id` does not match `FTE-*` pattern |
| E909 | `eventKind` is not one of `basic · undeveloped · house` |

## Tier 4 — FMEA (E911–E914, W902–W904)

### FMEASheet (E911–E912, W902)

| Code | Condition |
|---|---|
| E911 | `id`, `title`, or `status` is absent |
| E912 | `id` does not match `FMEA-*` pattern |
| W902 | FMEASheet has no `entries` — add at least one failure mode row |

### FMEAEntry (E913–E914, W903–W904)

FMEAEntry elements are synthesised at parse time from each row in a `FMEASheet.entries:` list. They are not authored as standalone files.

| Code | Condition |
|---|---|
| E913 | Entry `id` does not match `FM-*` pattern |
| E914 | `fmeaSeverity`, `occurrence`, or `detection` is outside the range 1–10 |
| W903 | Computed RPN (severity × occurrence × detection) exceeds 100 and no `recommendedAction` is set |
| W904 | Entry `ref` field does not resolve to a known model element |

RPN is computed automatically when all three of `fmeaSeverity`, `occurrence`, and `detection` are present. An explicit `rpn:` field is accepted and used as-is.

## Tier 4 — TARA container (E940–E941, W905)

`TARASheet` is an Option-B container whose four section tables (`damageTable`, `threatTable`, `goalTable`, `controlTable`) are each exploded at parse time into the corresponding Tier 2 element types.

| Code | Condition |
|---|---|
| E940 | `id`, `title`, or `status` is absent |
| E941 | `id` does not match `TARA-*` pattern |
| W905 | TARASheet has no rows in any section table |

Once rows are exploded, all existing Tier 2 validation rules (E807–E821, E825–E830, W800–W803) apply to the synthesised elements exactly as they would to hand-authored files.

## Tier 4 — Attack path analysis (E915–E921, W035–W037)

ISO/SAE 21434 §15.7 attack trees, mirroring the Fault Tree Analysis family. An
`AttackTree` (`AT-*`) substantiates a `ThreatScenario` via `threatRef` and
decomposes it into `AttackTreeGate`s (`ATG-*`, `gateType` `AND`/`OR`) and
`AttackStep`s (`ATS-*`, leaf with `attackFeasibility`), with the gates/steps
nested in a subdirectory named after the tree file. Feasibility is rolled up with
the **weakest-link** rule (rank `very_low`=0 … `high`=3): an `AttackStep` is its
`attackFeasibility` rank; an `AND` gate (a sequential path) is the **MIN** of its
children; an `OR` gate (alternatives) is the **MAX** of its children; the tree's
feasibility is the value of its single root child, mapped back to a label.

### AttackTree (E915–E917, W035–W036)

| Code | Condition |
|---|---|
| E915 | `id`, `title`, `status`, or `threatRef` is absent |
| E916 | `id` does not match `AT-*` pattern |
| E917 | `threatRef` does not resolve, or resolves to an element that is not a `ThreatScenario` |
| W035 | The tree's computed (weakest-link) feasibility does not match the linked `ThreatScenario.attackFeasibility` — message names computed vs declared. Gateable via `--deny W035`; promotable via `[profiles]` |
| W036 | AttackTree has no gates or steps — the tree is empty |

### AttackTreeGate (E918–E920, W037)

| Code | Condition |
|---|---|
| E918 | `id`, `title`, or `gateType` is absent, or `id` does not match `ATG-*` pattern |
| E919 | `gateType` is not one of `AND` (sequential path) · `OR` (alternatives) |
| E920 | An entry in `inputs` does not resolve, or resolves to an element that is not an `AttackTreeGate` or `AttackStep` |
| W037 | AttackTreeGate has no `inputs` — it contributes nothing to the attack tree |

### AttackStep (E921)

| Code | Condition |
|---|---|
| E921 | `id` or `title` is absent; `id` does not match `ATS-*` pattern; or `attackFeasibility` is not one of `high · medium · low · very_low` |
