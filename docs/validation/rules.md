# Rule Reference

`VALIDATION · RULES`

Warnings are advisory by default (exit `0`). Promote them to CI gate failures (exit `2`) with `validate --deny <CODES>` / `--max-warnings <N>` / `--warnings-as-errors`, or with a named, SIL/ASIL-scopable `validate --profile <name>` policy declared in `.syscribe.toml` — see [CI severity gating](../cli/index.md#ci-severity-gating). Errors always exit `1`.

## Parse-time errors (E001–E025)

| Code | Element | Condition |
|---|---|---|
| E000 | — | Internal fallback code for a derive-pass finding whose original code is not one of the recognised derive codes (`E500`/`E501`/`E502`). Should not appear in a healthy model |
| E002 | Any | Frontmatter is not valid YAML 1.2 (parse error) |
| E004 | TestCase | `id`, `name`, `status`, or `testLevel` absent |
| E005 | Any | `type:` value is present but is not in the element type inventory (unrecognised type) |
| E004 | Requirement | `name` or `status` absent on native Requirement |
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
| E023 | Any (stable id) | The numeric suffix is longer than the configured maximum (`[ids] max_digits` in `.syscribe.toml`, default 8; minimum 3 enforced by E006) |
| E024 | — | **RETIRED.** Formerly flagged a `name:` field on an id-identified type. `name` is now the single, required label on every element, so this code is **no longer emitted** — a `Requirement` carrying `id` + `name` validates clean. |
| E025 | Any element | The removed `title:` field is declared on an element (id-identified or name-identified alike) — the `title` field is removed; rename it to `name`. (A `FeatureDef` carries `name` as its label and a mandatory `FEAT-*` `id` — see `E201` — the `id` and label axes are independent.) |

## Parse-time warnings (W001–W008)

| Code | Condition |
|---|---|
| W001 | Requirement normative text contains no `shall` |
| W004 | `sourceFile:` path does not exist on disk |
| W006 | Both `silLevel` (IEC 61508) and `asilLevel` (ISO 26262) are set on the same element — incompatible standards; use only one |
| W007 | Type definition (e.g. `PartDef`) is never referenced as a supertype or type |
| W008 | Element has no `type:` field — will be ignored by most commands |
| W009 | A TestCase `testFunctions[].function` is not found in its `sourceFile` (live source-drift; a planned/draft TestCase reports the informational `I010` instead) |

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
| E201 | Configuration missing `id`, `name`, `status`, or `featureModel`; **or** a `FeatureDef` missing its mandatory `FEAT-*` `id` |
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
| E301 | ADR missing `id`, `name`, or `status` |
| E302 | `reqDomain` value is not `system · hardware · software` |
| E303 | `domain` value is not `system · hardware · software` |
| E304 | ADR `status` is not `proposed · accepted · deprecated · superseded` |

## Traceability warnings (W300–W307)

| Code | Condition |
|---|---|
| W300 | Leaf Requirement at `approved` or `implemented` has no satisfying element |
| W301 | Leaf Requirement is satisfied by more than one element |
| W302 | Leaf Requirement at `implemented` or `verified` still has `reqDomain: system` |
| W303 | `breakdownAdr:` references a `proposed` ADR but Requirement is `approved` or higher |
| W304 | `isDeploymentPackage: true` combined with `domain: hardware` |
| W305 | Parent Requirement (has `derivedFrom` children) at `approved`, `implemented`, or `verified` has no active TestCase at `testLevel: L3`, `L4`, or `L5` — leaf-level tests on derived requirements are not sufficient to verify emergent composed behaviour |
| W306 | **Unsatisfied safety mechanism** — a high-integrity Requirement (`silLevel >= 4` or `asilLevel: D`) that is **not** a fully integrated safety mechanism: `status: draft`, **or** (for a **leaf**) no element satisfies it, **or** (with a feature model) active in no `Configuration`. The "unsatisfied" sub-condition applies to leaves only — a **parent** (has `derivedChildren`) is satisfied transitively and can't be satisfied directly (`E312`), so it is never flagged unsatisfied. Message names the triggering sub-condition(s). Gate with `--deny W306`. (Threshold/sub-condition tuning rides with severity profiles, #18.) |
| W307 | A non-`draft` `UseCaseDef` carries no `refines:` link to a requirement (absent or empty). Advisory and draft-suppressed; gate with `--deny W307` and promote it to a gate failure via the `[profiles.magicgrid]` profile. See the [MagicGrid](#magicgrid-overlay-e316-w307-mg010mg070) section. |

## §12 Traceability errors (E310–E316)

| Code | Condition |
|---|---|
| E310 | Requirement has `derivedFrom:` but no `breakdownAdr:` |
| E311 | `breakdownAdr:` does not resolve, or does not resolve to an ADR |
| E312 | Parent requirement (has `derivedChildren`) appears in a `satisfies:` list |
| E313 | `satisfies` domain mismatch: element domain ≠ requirement `reqDomain` |
| E314 | `isDeploymentPackage: true` element has no Allocation to a hardware element |
| E315 | Cross-domain `supertype:` or `typedBy:` reference — use Allocation instead |
| E316 | A `refines:` operand on a `UseCaseDef`/`UseCase` — or on a behavioral definition `ActionDef`/`Action`/`StateDef`/`State` — does not resolve, or resolves to an element that is not a `Requirement`/`RequirementDef`. **Base-format check** — runs regardless of the MagicGrid profile. The `refinedBy` reverse index includes refining behavioral elements alongside refining use cases. |

## §12.8 Implementation trace (W023)

The optional `implementedBy:` field on a `Part`, `PartDef`, `Interface`, or `InterfaceDef` records the source artifact(s) that realise an architecture element — the downstream leg of the V-model (`Requirement ─satisfies→ Architecture ─implementedBy→ Code ─verifies→ Test`). For interface-typed elements this typically points to a header file, IDL file, or protocol spec document.

| Code | Condition |
|---|---|
| W023 | A non-`draft` `Part`/`PartDef`/`Interface`/`InterfaceDef` has an `implementedBy:` path that does not exist on disk |

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

## Diagram errors (E400–E404)

| Code | Condition |
|---|---|
| E400 | `diagramKind: Mermaid` but body has no ` ```mermaid ` block |
| E401 | `diagramKind: PlantUML` but body has no ` ```plantuml ` block |
| E402 | `svgFile:`/companion SVG path does not exist on disk (`svgMode: companion`, or `svgFile:` set without `svgMode`) |
| E403 | `pumlMode:` declares an unrecognised value (only `companion` is supported) |
| E404 | `pumlMode: companion` is set but the element has no `diagramKind:` to derive the PlantUML companion from |

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
| W413 | `pumlMode: companion` element's body contains no image reference to its rendered PlantUML companion (REQ-TRS-PUML-030) |
| W414 | `pumlMode: companion` element's `.puml` companion file has not been generated yet — run `plantuml` (REQ-TRS-PUML-031) |
| W415 | The `[plantuml] style_file` path configured in `.syscribe.toml` does not exist on disk (REQ-TRS-PUML-042) |
| W080 | `Sequence` diagram's subject `ActionDef` has a `SendAction`/`AcceptAction` in its sub-action tree not referenced by any `edges:` entry (draft-suppressed; `--deny W080`) |

## State machine warnings (W070–W079, §22.1)

SysMLv2-faithful state-machine checks on `StateDef`/`State`. All draft-suppressed and gateable with `--deny W07x`.

| Code | Condition |
|---|---|
| W070 | Dead state — a substate has no incoming transition and is not `isInitial: true` (single-region machines). |
| W071 | Trap state — a substate has no outgoing transition and is not `isFinal: true`. |
| W072 | Non-determinism — two+ transitions from one source with the same `accept` payload, none guarded. |
| W073 | Missing initial — a single-region `StateDef` with substates has no `isInitial: true` substate. |
| W074 | Multiple initial — more than one substate is `isInitial: true`. |
| W075 | A transition uses the deprecated keys `from:`/`to:`/`trigger:` instead of the canonical `source:`/`target:`/`accept:` (§8.8.3). Legacy keys are still accepted as aliases. |
| W076 | Unresolved endpoint — a transition `source`/`target` names no state anywhere in the machine and resolves to no model element. |
| W077 | Cross-region transition — a transition connects substates in two different regions of an `isParallel` state (illegal in SysMLv2). |
| W078 | Parallel arity — an `isParallel: true` state declares fewer than two regions. |
| W079 | Unresolved behavior — a state `entryAction`/`doAction`/`exitAction` or a transition `effect` references an action that resolves to no model element. |

The checks apply **recursively** over the state hierarchy: each level's substates are checked by `W070`–`W074` (composite substates as nodes), inline-`subStates:` substates are recursed into, and parallel (`isParallel`) levels are checked per region plus `W077`/`W078`. `W076` covers transition endpoints that resolve to no state.

## Budget expression validation (E866–E868, W060, §22.2)

For a `CalculationDef` with `bodyLanguage: budget`, the `body:` is a restricted arithmetic expression (`+ - * /`, parentheses, number literals, and `feature_ref` operands resolving to inline attribute `value:`/`default:`). An optional `evaluate:` names a `ConstraintDef` bounding the result.

| Code | Condition |
|---|---|
| E866 | `evaluate:` does not resolve to a `ConstraintDef`. |
| E867 | The budget `body:` expression has a syntax error. |
| E868 | A `feature_ref` operand resolves to no numeric attribute in scope. |
| W060 | The budget value violates the `evaluate:` constraint (best-effort, for `<lhs> <op> <number>` bounds; draft-suppressed; `--deny W060`). |

## Trade studies (E869–E877, W061–W064, §15)

A `TradeStudy` (`TRD-*`) is a weighted-criteria evaluation; the tool computes normalised/weighted scores and rankings (never written to disk). Codes drafted as `E400`–`E408`/`W400`–`W403` were reassigned (they collide with the Diagram codes).

| Code | Condition |
|---|---|
| E869 | Missing `id`, `name`, `status`, `criteria`, `alternatives`, or `scores`. |
| E870 | `id` does not match the `TRD-*` pattern. |
| E871 | A `criteria:` entry is missing `name`, `weight`, or `direction`. |
| E872 | A `criteria[].weight` is not in [0.0, 1.0], or all weights are zero. |
| E873 | A `criteria[].direction` is not `maximize` or `minimize`. |
| E874 | `alternatives:` is empty. |
| E875 | An `alternatives:` entry is missing `name`. |
| E876 | A `scores:` entry references an unknown alternative or criterion. |
| E877 | A `scores[].score` is not a number. |
| W061 | A `status: complete` study has no `decision:` ADR. |
| W062 | `objective:` is present but unresolved (draft-suppressed). |
| W063 | The score matrix is incomplete (draft-suppressed). |
| W064 | An `alternatives[].element` is present but unresolved (draft-suppressed). |

## IEC 62443 Zone/Conduit (E950–E956, W950–W953, §13)

| Code | Condition |
|---|---|
| E950 | `Zone` missing `id`/`name`/`status`/`targetSL`. |
| E951 | `Zone.id` not a `ZN-*` id. |
| E952 | `Conduit` missing `id`/`name`/`status`/`fromZone`/`toZone`. |
| E953 | `Conduit.id` not a `CD-*` id. |
| E954 | `Conduit.fromZone`/`toZone` unresolved or not a `Zone`. |
| E955 | `Zone.members:` entry unresolved or not a `PartDef`/`Part`. |
| E956 | `PartDef`/`Part.inZone:` unresolved or not a `Zone`. |
| W950 | `Zone.achievedSL < targetSL` (SL gap). |
| W951 | `Conduit.achievedSL` below a connected zone's `targetSL` (opt-in). |
| W952 | A part declares `targetSL` but belongs to no zone (opt-in). |
| W953 | An `approved` `Zone` (`targetSL >= 2`) referenced by no `Conduit`. |

## Documentation linting (W099–W102, `lint-docs`)

The `lint-docs` command scans external `.md` and `.svg` docs for references to model elements that no longer resolve (gateable, e.g. `--deny W100`).

| Code | Condition |
|---|---|
| W099 | An unresolvable stable-ID token (`REQ-*`/`TC-*`/…) in prose. |
| W100 | A qualified name (`A::B::C`) inside a ` ```mermaid ` block that does not resolve (prose qnames are not checked). |
| W101 | An SVG `sysml:ref="…"` that does not resolve (SVGs with no `sysml:ref` are opaque). |
| W102 | A local image/diagram embed path (`![](…)`, `<img src>`) that does not exist (remote URIs accepted). |

## Review records (E700–E705, W700, W704, §19)

A `ReviewRecord` (`RR-*`) is a baselined, thin traceability anchor for a formal review; the discussion lives in the external tool named by `recordedAt:`.

| Code | Condition |
|---|---|
| E700 | Missing `id`, `name`, `status`, `reviewType`, or `reviews` (≥1). |
| E701 | `id` does not match the `RR-*` pattern. |
| E702 | `status` not in `open \| closed \| waived`. |
| E703 | `reviewType` not in the allowed enum. |
| E704 | A `reviews:` entry does not resolve. |
| E705 | An `items[].disposition` is not `open \| closed \| not_applicable`. |
| W700 | A `status: closed` review has an `items[]` with `disposition: open`. |
| W704 | A non-`draft` native Requirement appears in no `ReviewRecord.reviews:` list (dormant unless ReviewRecords exist; `--deny W704`). |

## Multi-repository composition (E510–E515, W510–W512, §14)

A model composes peer repositories declared in the `[repos]` table of the model-root `.syscribe.toml` and imports their namespaces via `repoImports:` on a Package `_index.md`. Cross-repo `verifies:`/`derivedFrom:`/`satisfies:`/`allocatedTo:` references resolve against the local model first, then each loaded repo in declaration order (by global stable ID or qualified name). **Active only when `[repos]` is configured** — single-repo models are unaffected.

| Code | Condition |
|---|---|
| E510 | Circular repo import — a repo transitively imports back into this model. |
| E511 | `repos.<alias>.path` is absent on disk and no `ref:` is configured. |
| E512 | A cross-repo `verifies`/`derivedFrom`/`satisfies`/`allocatedTo` reference resolves in neither the local model nor any loaded repo. |
| E513 | `repoImports[].repo` names an alias not present in `[repos]`. |
| E514 | `repoImports[].qname` does not resolve to any element in the named repo. |
| E515 | Two repos export the same stable ID (the id namespace is global across the composition). |
| W510 | A repo in `[repos]` has no `ref:` — composition is not pinned to a reproducible snapshot (opt-in; `--deny W510`). |
| W511 | A peer repo's git `HEAD` has drifted from its configured `ref:` — checkout is not at the pinned snapshot. Never raised when drift cannot be determined (no git, not a work tree, ref unresolved). Opt-in; `--deny W511` for a CI reproducibility gate. |
| W512 | A peer repo's `path` is a **git submodule** of the composing model's repo, and its `ref:` resolves to a different commit than the gitlink the parent repo records — `.syscribe.toml` disagrees with `.gitmodules`. Independent of `W511` (gitlink pin vs ref, not checkout vs ref). Never raised when `path` is not a submodule. Opt-in; `--deny W512`. |

## Build-system integration (E050, W050, §9.9)

A `FeatureDef` or `Configuration` may declare `buildExports:` mapping selected features to build-system variables, with `buildOverrides:` resolving conflicts. **Opt-in** — the pass runs only when at least one element declares `buildExports:` or `buildOverrides:`.

| Code | Condition |
|---|---|
| E050 | Two selected features export the same `buildExports` variable name and the conflict is not resolved by `buildOverrides:` |
| W050 | A selected feature contributes no build variable (no `buildExports:` and no parameter with `buildVar:`). Opt-in; gate with `--deny W050` |

## Allocation errors (E500–E503)

| Code | Condition |
|---|---|
| E500 | Feature with `type: Allocation` has `allocatedFrom:` that does not resolve |
| E501 | Feature with `type: Allocation` has `allocatedTo:` that does not resolve |
| E502 | `allocatedFrom:` entry (on any element) does not resolve to a known element |
| E503 | `allocatedTo:` entry (on any element) does not resolve to a known element |

## Structural warnings (W500–W503)

| Code | Condition |
|---|---|
| W500 | `viewpoint:` on View does not resolve to a ViewpointDef |
| W501 | `exhibitsStates:` entry does not resolve to any known element |
| W502 | `expose:` entry on View does not resolve to any known element |
| W503 | The **same** allocation edge `source → target` is declared by **both** an `allocatedTo:` on the source **and** a standalone `Allocation` element — redundant; use one form (§12.9, `REQ-TRS-ALLOC-001`) |

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
| E800 | `id`, `name`, or `status` is absent |
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
| E805 | `id`, `name`, or `status` is absent |
| E806 | `id` does not match `SG-*` pattern |
| E837 | `plLevel` is not one of `a · b · c · d · e` (ISO 13849-1) |
| W801 | SafetyGoal has no integrity level — set `asilLevel` (ISO 26262), `silLevel` (IEC 61508), or `plLevel` (ISO 13849-1) |
| W806 | SafetyGoal has no `hazardousEvents` — it is not grounded in any hazard analysis |

### DamageScenario (E807–E810)

| Code | Condition |
|---|---|
| E807 | `id`, `name`, or `status` is absent |
| E808 | `id` does not match `DS-*` pattern |
| E809 | `damageSeverity` is not one of `severe · major · moderate · negligible` |
| E810 | `impactCategories` entry is not one of `safety · financial · operational · privacy` |
| E844 | `hazardRef` does not resolve, or resolves to an element that is not a `HazardousEvent`/`SafetyGoal` (safety↔security co-engineering) |

### ThreatScenario (E811–E814, E845)

| Code | Condition |
|---|---|
| E811 | `id`, `name`, or `status` is absent |
| E812 | `id` does not match `TS-*` pattern |
| E813 | `attackFeasibility` is not one of `high · medium · low · very_low` |
| E814 | `attackVector` is not one of `network · adjacent · local · physical` |
| E845 | `riskTreatment` is not one of `avoid · reduce · share · retain` |
| E844 | `hazardRef` does not resolve, or resolves to an element that is not a `HazardousEvent`/`SafetyGoal` (safety↔security co-engineering) |

### CybersecurityGoal (E815–E818)

| Code | Condition |
|---|---|
| E815 | `id`, `name`, or `status` is absent |
| E816 | `id` does not match `CSG-*` pattern |
| E817 | `securityProperty` is not one of `confidentiality · integrity · availability · authenticity` |
| E818 | `calLevel` is not one of `CAL1 · CAL2 · CAL3 · CAL4` |

### SecurityControl (E819–E821)

| Code | Condition |
|---|---|
| E819 | `id`, `name`, or `status` is absent |
| E820 | `id` does not match `SC-*` pattern |
| E821 | `controlType` is not one of `prevention · detection · response · recovery` |

### VulnerabilityReport (E822–E824, W803)

| Code | Condition |
|---|---|
| E822 | `id`, `name`, or `status` is absent |
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

## Asset identification — ISO/SAE 21434 §15.3 (E861–E864, W810)

An `Asset` (`ASSET-*`) is a model element worth protecting; `DamageScenario.assets:` links a damage scenario to the assets it endangers. See `docs/model-guide/safety-analysis.md`.

| Code | Severity | Condition |
|---|---|---|
| E861 | Error | `Asset` is missing `id`, `name`, or `status` |
| E862 | Error | `Asset.id` does not match the `ASSET-*` pattern (`^ASSET(-[A-Z0-9]{2,12})+-[0-9]{3,}$`) |
| E863 | Error | `Asset.cybersecurityProperties` entry is not one of `confidentiality · integrity · availability · authenticity` |
| E864 | Error | a `DamageScenario.assets` entry does not resolve to an `Asset` element (REQ-TRS-TYPE-017) |
| W810 | Warning | An `Asset` is not referenced by any `DamageScenario.assets` (asset-identification gap; REQ-TRS-TYPE-017) |

## Security test methods — ISO/SAE 21434 §13 (W809)

| Code | Severity | Condition |
|---|---|---|
| W809 | Warning | A `TestCase.securityTestMethod` is not a recognised ISO/SAE 21434 §13 test method (REQ-TRS-SEC-008) |

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

## Confirmation measures & DIA/CIA responsibility (E847–E851, E860, W038, W039)

ISO 26262-2 §6 confirmation measures, ISO 26262-8 §5 Development Interface Agreement (DIA),
and ISO/SAE 21434 §7 Cybersecurity Interface Agreement (CIA). Both checks are **opt-in**.

**`responsibility:`** (common field, any element) — the accountable party/organisation for a
work product (the DIA/CIA split, e.g. `OEM` / `Supplier-X`).

**ConfirmationMeasure** (`type: ConfirmationMeasure`, `CM-*` id) — a confirmation review, FS
audit, FS assessment, or cybersecurity assessment, with `measureType:`, `independenceLevel:`
(`I1`/`I2`/`I3`), `status:`, and `confirms:` (work-product ref(s) resolved via the `Resolver`).

The ASIL/CAL → independence mapping is intentionally minimal: only `asilLevel: D → I3
functional_safety_assessment` and `calLevel: CAL4 → I3 cybersecurity_assessment` are gated.
Lower integrity levels are documented as future tightening and are not gated.

| Code | Severity | Condition |
|---|---|---|
| E847 | Error | `ConfirmationMeasure` is missing `id`, `name`, or `status` |
| E848 | Error | `ConfirmationMeasure.id` does not match the `CM-*` pattern |
| E849 | Error | `measureType` is not one of `confirmation_review · functional_safety_audit · functional_safety_assessment · cybersecurity_assessment` |
| E850 | Error | `independenceLevel` is not one of `I1 · I2 · I3` |
| E851 | Error | a `confirms:` ref does not resolve to any model element |
| E860 | Error | a `ConfirmationMeasure.confirms` ref resolves to an element that is not a `SafetyGoal`, `CybersecurityGoal`, `HazardousEvent`, or native `Requirement` (REQ-TRS-SEC-005) |
| W038 | Warning | A non-draft work product (`Requirement`, `PartDef`, `Part`, `SafetyGoal`, `CybersecurityGoal`) declares no `responsibility:`. **Opt-in:** dormant unless some element declares `responsibility:`. Gate with `--deny W038`; promotable via `[profiles]` |
| W039 | Warning | A high-integrity item lacks its required independent assessment: an `asilLevel: D` **or `silLevel: 3`/`silLevel: 4`** `SafetyGoal`/native `Requirement` not confirmed by an I3 `functional_safety_assessment` (ISO 26262-2 §6 / IEC 61508-1 §8); or a `calLevel: CAL4` `CybersecurityGoal` not confirmed by an I3 `cybersecurity_assessment`. **Opt-in:** dormant unless at least one `ConfirmationMeasure` exists. Gate with `--deny W039`; promotable via `[profiles]` |

See `docs/model-guide/safety-analysis.md`.

## GSN safety-argument layer (E852–E859, W040)

The Goal Structuring Notation (GSN) argument layer (issue #20). `Argument` (`ARG-*`)
nodes argue for a `SafetyGoal` or a parent `Argument`, discharged by `evidence`
(`Requirement` / `TestCase` / sub-`Argument` / `AssumptionOfUse`). `AssumptionOfUse`
(`AOU-*`) records a safety-related application condition (SRAC). Render the tree with
`syscribe safety-case`.

### Argument (E852–E855, W040)

| Code | Severity | Condition |
|---|---|---|
| E852 | Error | `Argument` is missing `id`, `name`, or `status` |
| E853 | Error | `Argument.id` does not match the `ARG-*` pattern |
| E854 | Error | `Argument.argumentType` is not one of `claim · strategy · solution` (absent → treated as `claim`) |
| E855 | Error | an `Argument.supports` or `Argument.evidence` ref does not resolve to any model element |
| W040 | Warning | a `claim`/`strategy` `Argument` has **both** an empty `supports` and an empty `evidence` (an orphan GSN node arguing nothing) |

### AssumptionOfUse (E856–E859)

| Code | Severity | Condition |
|---|---|---|
| E856 | Error | `AssumptionOfUse` is missing `id`, `name`, or `status` |
| E857 | Error | `AssumptionOfUse.id` does not match the `AOU-*` pattern |
| E858 | Error | an `AssumptionOfUse.appliesTo` ref does not resolve to any model element |
| E859 | Error | an `AssumptionOfUse.appliesTo` ref resolves to an element that is not a `SafetyGoal`, `CybersecurityGoal`, `Argument`, or `Requirement` (REQ-TRS-SEC-004) |

See `docs/model-guide/safety-analysis.md`.

## Integrity level propagation errors and warnings (E841–E843, W808)

Once any element in the traceability chain carries `asilLevel` or `silLevel`, all downstream elements must inherit the same field. A lower level is permitted only when accompanied by a `breakdownAdr` documenting the ASIL/SIL decomposition rationale (ISO 26262-9 / IEC 61508-2 §7.4.9).

| Code | Severity | Condition |
|---|---|---|
| E841 | Error | Element with `derivedFromSafetyGoal` is missing `asilLevel`/`silLevel` when the referenced SafetyGoal carries one |
| E842 | Error | Element with `derivedFrom` is missing `asilLevel`/`silLevel` when the parent element carries one |
| E843 | Error | Element with `satisfies` is missing `asilLevel`/`silLevel` when the satisfied requirement carries one |
| W808 | Warning | Element's integrity level is strictly lower than its source (`derivedFromSafetyGoal`, `derivedFrom`, or `satisfies`) but no `breakdownAdr` is set |
| E865 | Error | ASIL D / SIL 4 decomposition siblings (uniformly-lower children) share a `satisfies:` target — channels must be architecturally independent (§22.3) |
| W860 | Warning | An ASIL D / SIL 4 requirement has a single uniformly-lower child — a decomposition needs ≥2 independent channels (§22.3) |

## Tier 4 — Fault Tree Analysis (E900–E909, W900–W901)

### FaultTree (E900–E902, W900)

| Code | Condition |
|---|---|
| E900 | `id`, `name`, `status`, or `topEvent` is absent |
| E901 | `id` does not match `FT-*` pattern |
| E902 | `topEvent` does not resolve, or resolves to an element that is not a `SafetyGoal` |
| W900 | FaultTree has no gates or events — the tree is empty |

### FaultTreeGate (E903–E906, W901)

| Code | Condition |
|---|---|
| E903 | `id`, `name`, or `gateType` is absent |
| E904 | `id` does not match `FTG-*` pattern |
| E905 | `gateType` is not one of `AND · OR · XOR · NOT · inhibit` |
| E906 | An entry in `inputs` does not resolve, or resolves to an element that is not a `FaultTreeGate` or `FaultTreeEvent` |
| W901 | FaultTreeGate has no `inputs` — it contributes nothing to the fault tree |

### FaultTreeEvent (E907–E909)

| Code | Condition |
|---|---|
| E907 | `id`, `name`, or `eventKind` is absent |
| E908 | `id` does not match `FTE-*` pattern |
| E909 | `eventKind` is not one of `basic · undeveloped · house` |

## Tier 4 — FMEA (E911–E914, W902–W904)

### FMEASheet (E911–E912, W902)

| Code | Condition |
|---|---|
| E911 | `id`, `name`, or `status` is absent |
| E912 | `id` does not match `FMEA-*` pattern |
| W902 | FMEASheet has no `entries` — add at least one failure mode row |

### FMEAEntry (E913–E914, E922, W903–W904)

FMEAEntry elements are synthesised at parse time from each row in a `FMEASheet.entries:` list. They are not authored as standalone files.

| Code | Condition |
|---|---|
| E913 | Entry `id` does not match `FM-*` pattern |
| E914 | `fmeaSeverity`, `occurrence`, or `detection` is outside the range 1–10 |
| E922 | An `entries:` row contains an unrecognised key — the field is silently ignored, which constitutes silent data loss in a safety analysis (error-level) |
| W903 | Computed RPN (fmeaSeverity × occurrence × detection) exceeds 100 and no `recommendedAction` is set |
| W904 | Entry `ref` field does not resolve to a known model element |
| W926 | A `FaultTreeEvent.fmeaRef` does not resolve to a known `FMEAEntry` (FTA↔FMEA cross-link) |
| W927 | An `FMEAEntry.ftaRef` does not resolve to a known `FaultTreeEvent` (FMEA↔FTA cross-link) |

The canonical severity key is **`fmeaSeverity:`** (camelCase, integer 1–10). The deprecated alias `severity:` is accepted and silently mapped for backward compatibility. RPN is computed automatically as `fmeaSeverity × occurrence × detection` when `rpn:` is absent; an explicit `rpn:` overrides the computed value.

## Tier 4 — TARA container (E940–E941, W905)

`TARASheet` is an Option-B container whose four section tables (`damageTable`, `threatTable`, `goalTable`, `controlTable`) are each exploded at parse time into the corresponding Tier 2 element types.

| Code | Condition |
|---|---|
| E940 | `id`, `name`, or `status` is absent |
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
| E915 | `id`, `name`, `status`, or `threatRef` is absent |
| E916 | `id` does not match `AT-*` pattern |
| E917 | `threatRef` does not resolve, or resolves to an element that is not a `ThreatScenario` |
| W035 | The tree's computed (weakest-link) feasibility does not match the linked `ThreatScenario.attackFeasibility` — message names computed vs declared. Gateable via `--deny W035`; promotable via `[profiles]` |
| W036 | AttackTree has no gates or steps — the tree is empty |

### AttackTreeGate (E918–E920, W037)

| Code | Condition |
|---|---|
| E918 | `id`, `name`, or `gateType` is absent, or `id` does not match `ATG-*` pattern |
| E919 | `gateType` is not one of `AND` (sequential path) · `OR` (alternatives) |
| E920 | An entry in `inputs` does not resolve, or resolves to an element that is not an `AttackTreeGate` or `AttackStep` |
| W037 | AttackTreeGate has no `inputs` — it contributes nothing to the attack tree |

### AttackStep (E921)

| Code | Condition |
|---|---|
| E921 | `id` or `name` is absent; `id` does not match `ATS-*` pattern; or `attackFeasibility` is not one of `high · medium · low · very_low` |

## TestPlan (E600–E606, W610–W616)

| Code | Condition |
|---|---|
| E600 | `TestPlan` missing `id`/`name`/`status`, or `id` does not match `TP(-[A-Z0-9]{2,12})+-[0-9]{3,8}` |
| E601 | a `testCases:` entry does not resolve to a `TestCase` |
| E602 | a `selection.testLevels` value is not one of `L1`–`L5` |
| E603 | a `demonstrates:` target does not resolve to a Requirement/SafetyGoal/CybersecurityGoal/Argument |
| E604 | `status` is not one of `draft · review · approved · active · retired` |
| E605 | a `selection.domains` value is not one of `system`/`hardware`/`software` |
| E606 | a `configurations:` entry does not resolve to a `Configuration` |
| W610 | `scope` is not in the recommended vocabulary (`unit·smoke·integration·hil·certification·security·regression`) |
| W611 | a member `TestCase` is active in none of the plan's bound configurations (escaping member) |
| W612 | the effective TestCase set is empty |
| W613 | a `TestCase` named explicitly in `testCases:` is `draft`/`retired` |
| W614 | an `approved`/`active` plan demonstrates a `Requirement` that no member verifies (honours goal-closure) |
| W615 | results-gated: an `approved` plan has a member whose ingested verdict is Fail/Missing |
| W616 | two plans share an identical `(configurations, scope)` pair |

A duplicate `TestPlan` `id` is the generic `E101`.

## Custom fields (W041)

| Code | Condition |
|---|---|
| W041 | a `custom_fields` value is not a scalar or a list of scalars (e.g. a nested map); names the offending key |

## MagicGrid overlay (E316, W307, MG010–MG070)

The MagicGrid method is supported as a **`custom_fields:` overlay** — see the
[MagicGrid guide](../model-guide/magicgrid.md). The `MG###` namespace is **opt-in**:
these checks fire only under the MagicGrid profile (`[profiles.<name>] magicgrid = true`,
e.g. `validate --profile magicgrid`). They validate `mg_`-prefixed `custom_fields:`
and the base `actors:` field, all of which stay inert in the base format. All
`MG###` findings are **Error** severity. `E316` (above) is a base-format check that
always runs; `W307` (above) is advisory until promoted.

| Code | Condition |
|---|---|
| MG010 | An `actors:` entry (on a `UseCaseDef`/`UseCase`/use-case-style `RequirementDef`/`Requirement`) resolves to no model element |
| MG011 | An `actors:` entry resolves to an element that is not a `Part`/`PartDef` |
| MG012 | A referenced actor `Part`/`PartDef` is not marked `custom_fields: { mg_external: true }` — a non-external actor is a B3 modelling error |
| MG013 | A non-`draft` `UseCaseDef` declares an empty or absent `actors:` list |
| MG020 | `custom_fields.mg_cell` is not one of the recognised coordinates `B1`–`B4`, `W1`–`W4`, `S1`–`S4` |
| MG021 | An element's `type` is incompatible with the pillar implied by its `mg_cell` column (col1→Requirement; col2→UseCase/Action/State; col3→Part/Port/Interface/Connection; col4→Constraint/Calculation/AnalysisCase) |
| MG030 | `custom_fields.mg_moe: true` on an element that is not a `CalculationDef` or `AnalysisCase` |
| MG031 | `mg_moe_measures` is absent, or does not resolve to a `Requirement`/`RequirementDef` |
| MG032 | `mg_moe_direction` is absent or not `maximize`/`minimize` |
| MG033 | `mg_moe_threshold`/`mg_moe_objective` not numeric or inconsistent with the direction (`maximize` ⇒ objective ≥ threshold; `minimize` ⇒ objective ≤ threshold); or `mg_moe_weight` present and not numeric in `[0, 1]` |
| MG040 | `custom_fields.mg_layer` is present on a `Part`/`PartDef` and is not `logical` or `physical` |
| MG041 | A `Part`/`PartDef` with `mg_layer: logical` has no `Allocation` to a `physical` element |
| MG042 | A `logical` and a `physical` `Part`/`PartDef` share a direct `supertype:`/`typedBy:` link — relate the layers only through an explicit `Allocation` |
| MG050 | `custom_fields.mg_mop: true` (a Measurement of Performance) on an element that is not a `CalculationDef`, `ConstraintDef`, or `AnalysisCase` |
| MG051 | `mg_mop_refines` is absent, or does not resolve (by qname/id) to a model element |
| MG052 | `mg_mop_refines` resolves to an element that is not marked `custom_fields: { mg_moe: true }` — a MoP must refine an MoE |
| MG060 | `custom_fields.mg_soi: true` (the System of Interest) on an element that is not a `Part`/`PartDef` |
| MG061 | More than one element in the model is marked `mg_soi: true` — a MagicGrid model has a single system of interest |
| MG062 | An element is marked **both** `mg_soi: true` and `mg_external: true` — the SoI cannot also be external to itself |
| MG070 | `custom_fields.mg_variant: true` on an element that is not a `Configuration` |
| MG080 | **Warning** (coverage) — a non-`draft` B1 `Requirement` (`mg_cell: B1`) neither refined by a use case nor derived into a system requirement (orphan need) |
| MG081 | **Warning** (coverage) — a W2 functional-analysis element (`ActionDef`/`StateDef`, `mg_cell: W2`) allocated to no logical (W3) `Part`/`PartDef` |
| MG082 | **Warning** (coverage) — the model declares an external actor (`mg_external: true`) but no element is marked `mg_soi: true` (missing System of Interest) |
| MG083 | **Warning** (coverage) — an `mg_moe` element that no Measurement of Performance refines (empty `mopRefinedBy`) |

The `MG080`–`MG083` checks are the **completeness / gap-analysis** half of MagicGrid
validation: advisory warnings (draft-suppressed where applicable, gateable with `--deny`,
promotable via the profile). They are rolled up, with a readiness summary and a PASS/FAIL
verdict, by **`magicgrid --audit`** (`--json` for CI).

**Root-name cross-reference hint (`REQ-TRS-XREF-006`).** Qualified names are derived
relative to the model root, so the root package (`_index.md`) contributes **no**
segment — a reference starts at the first sub-namespace
(e.g. `ProblemDomain::WhiteBox::…`), never `<RootName>::ProblemDomain::…`. When an
unresolved cross-reference begins with the root package's `name:` followed by `::`
and the *stripped* remainder resolves, the tool appends a diagnostic **hint** naming
the corrected reference. The hint augments the existing unresolved-reference finding
(`E102`/`E103`/`E311`/`E316`/`E502`/`E503` and the structural
supertype/typedBy/subsets/redefines/connection resolution errors); it is advisory
only — it never changes resolution and never rewrites the model. It does not fire
when the root package has no `name:`, nor when stripping the prefix still does not
resolve.

## Naming (W042)

| Code | Condition |
|---|---|
| W042 | A qualified-name segment — an element's own name or a package/directory name — is not a SysMLv2 basic name (`[A-Za-z_][A-Za-z0-9_]*`) and is not a stable id; rename using `_` or CamelCase. Hyphenated names cannot be referenced in `appliesWhen`/`parameterConstraints` (`-` is the subtraction operator). |

## Built-in standard-library types (W043, W044)

| Code | Condition |
|---|---|
| W043 | A type reference (`supertype`/`typedBy`/`returnType`/parameter `type`) names a member of a **closed** auto-imported package (`ScalarValues`, `Base`) that the package does not declare — e.g. `ScalarValues::Flota` — a likely typo; the message lists the package's known members. Recognised members (`ScalarValues::{Integer,Real,Natural,Boolean,String}`, `Base::{Anything,DataValue}`) resolve cleanly with no `W404`/`W043`. The **open** packages `ISQ`/`SI` are curated-recognised (clean, no `W404`) but **lenient** — an unrecognised `ISQ`/`SI` member is never flagged `W043`. |
| W044 | An element/feature declares **both** a recognised `ISQ` quantity type (`typedBy:`/parameter `type`) **and** a recognised `SI` unit (`unit:`) whose **physical dimensions differ** — e.g. `typedBy: ISQ::MassValue` with `unit: SI::metre`. The message names both and their dimensions (over the seven SI base quantities). Lenient when either side is unrecognised (e.g. a domain unit like `USD`). |

## Stereotypes — metadata applications (E317, E318, W045)

SysMLv2 has no UML stereotypes; a stereotype is a `MetadataDef` applied to an element via its `metadata:` field (bare reference, or a map whose `apply`/`def` names the def and whose other keys are tagged values).

| Code | Condition |
|---|---|
| E317 | A `metadata:` application does not resolve to a `MetadataDef` (unresolved stereotype). The root-name hint applies. |
| E318 | A `metadata:` application names a `MetadataDef` whose `annotates:` does not include the annotated element's type (directly or via the abstract `Element`/`Definition`/`Usage` metaclasses) — stereotype not applicable to this element kind. Standard-library metadata (`ModelingMetadata`, `RiskMetadata`) is recognised (no `E317`). |
| W045 | A tagged-value key in a `metadata:` application is not a declared `features:` attribute of the referenced `MetadataDef` (likely typo / undeclared attribute). |
