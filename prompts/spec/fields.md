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

## `.syscribe.toml` — project configuration reference

Everything below lives in **one file**, `<model_root>/.syscribe.toml`, never in model frontmatter.
Every table is **opt-in**: absent means built-in defaults, and a model with no `.syscribe.toml` at
all behaves identically to one with an empty file. Malformed individual entries are reported (a
warning, naming the file) and excluded rather than failing the whole table — the same posture
`W046` (`[ids.prefixes]`) and `W309` (`[users]`) both follow.

| Table / Key | Field | Type | Default | Notes |
|---|---|---|---|---|
| *(top-level)* | `repo_root` (alias `repoRoot`) | string | auto-detected (walks up for `.git`) | Git repo root; `repo:`-prefixed `sourceFile:`/`implementedBy:`/evidence `path:` values resolve against it. |
| `[ids]` | `max_digits` (alias `maxDigits`) | int | `8` | Max digits in a stable-ID numeric suffix (min `3`), §11, REQ-TRS-ID-005. |
| `[ids.prefixes]` | `<TypeName> = [<prefix>, ...]` | map → list of strings | `{}` | Extra stable-ID prefixes per element type, additive to the built-in (`REQ`/`TC`/`ADR`/…). Each prefix must match `^[A-Z][A-Z0-9]{1,11}$`; a malformed prefix or unknown type key is `W046` and ignored. REQ-TRS-ID-007. |
| `[repos]` | `<alias> = { path, root, ref }` | table of tables | `{}` | Peer repos for multi-repo composition (§14). `path` **required** (relative to `.syscribe.toml`); `root` default `"model/"`; `ref` (tag/branch/SHA) optional — absent pins nothing (`W510`). Drives `repoImports:`, `subConfigurations:` (§14.7), and cross-repo `verifies:`/`derivedFrom:`/etc. resolution. |
| `[links]` | `base_url` (alias `baseUrl`) | string | unset | Hosted-source URL template, simple form: `<base_url>/<path>`. REQ-TRS-LINK-001. |
| `[links]` | `url_template` (alias `urlTemplate`) | string | unset | Escape-hatch template: `{path}`/`{qname}`/`{id}`/`{ref}` placeholders. |
| `[links]` | `ref` | string | `""` | Substituted for `{ref}` in `url_template`. |
| `[scripts]` | `path` | string | `.syscribe/scripts/` | Rhai extension-scripts directory, relative to the model root. REQ-TRS-SCRIPT-001. |
| `[plantuml]` | `theme` | string | unset | `!theme <name>` emitted into generated `.puml` files. |
| `[plantuml]` | `style_file` | string | unset | `!include <path>`; takes precedence over `theme`. REQ-TRS-PUML-040. |
| `[plantuml]` | `base_url` | string | `http://localhost:3000` | Base URL for clickable element links; `""` suppresses links. |
| `[plantuml]` | `jar` | string | unset | Path to a PlantUML `.jar` for `plantuml render`. REQ-TRS-PUML-051. |
| `[baselines]` | `element_dir` (alias `elementDir`) | string | `model/Baselines` | Output dir for the sealed `type: Baseline` element. REQ-TRS-BL-010. |
| `[baselines]` | `manifest_dir` (alias `manifestDir`) | string | `<git-root>/baselines` | Output dir for the JSON manifest. |
| `[users]` | `<username> = "<display name>"` | map | `{}` | Roster for `PlanningItem.assignedTo:` (§23.7). A key not matching the Unix-style username shape `^[a-z_][a-z0-9_-]{0,31}$` is `W309` and excluded from the roster. REQ-TRS-PLANITEM-008. |
| `[profiles]` | `<name> = { promote, sil, status, tag, magicgrid }` | table of tables | `{}` | Named `--profile <name>` gate presets: `promote` (warning codes to gate on), `sil`/`status`/`tag` (optional scope filters — unscoped promotes everywhere), `magicgrid` (bool, runs the gated MagicGrid pass). |
| `[matchers]` | `<extension> = [<regex>, ...]` | map → list of strings | built-ins for Rust/Java/C/C++/Kotlin/shell | Per-extension function-definition patterns for `W009`; an override **replaces** the built-in list for that extension, not merges with it. |
| `[remote]` | `download` | string | unset | `sh -c` command template (`{url}`/`{dest}` placeholders) to fetch a remote `sourceFile:`. Only runs under the explicit `validate --fetch-remote` flag — configuring it alone never executes anything. |
| `[remote]` | `cache_dir` (alias `cacheDir`) | string | `.syscribe/cache` | Where fetched remote sources are cached, relative to the model root. |

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
| `mitigatedBy` | VulnerabilityReport | list | `SecurityControl` id/QName refs |
| `derivedFromSecurityGoal` | Requirement | string | `CSG-*` that generated this requirement |
| `derivedFromSafetyGoal` | Requirement | string | `SG-*` that generated this requirement |
