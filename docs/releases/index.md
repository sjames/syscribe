# Releases

`RELEASES`

## 0.18.0 — 2026-06-11

### SysMLv2 basic-name validation (GH #42)

Element names now follow the SysMLv2 **basic-name** grammar `[A-Za-z_][A-Za-z0-9_]*`. Previously a hyphenated name (e.g. a `FeatureDef` `Anti-Lock`) parsed as a file/qname but failed `E209` when referenced in `appliesWhen:` — because a hyphen is the subtraction operator in the expression grammar.

- New warning **`W042`**: an element's own name that is not a basic name (and not a stable id, which legitimately contains `-`) — with a rename hint (`Anti-Lock` → `Anti_Lock` / `AntiLock`). Advisory and gateable, so existing models have a migration path.
- The `appliesWhen` parse error for `-` now points at the basic-name convention.

(SysMLv2 *unrestricted* quoted names are not supported; the convention is basic names.) Requirement-first: `REQ-TRS-NAME-001` + `TC-TRS-NAME-001`. Suite at **146** test cases, all passing.

### Internal

- The release workflow now uploads assets via the `gh` CLI with retry (replacing a Node20 action that intermittently failed the Windows upload under matrix contention).

## 0.17.0 — 2026-06-10

### Configurable stable-ID suffix width (GH #41)

Stable IDs (REQ-*, TC-*, TP-*, ADR-*, and all safety/security ids) previously ended in **exactly 3 digits**. They now accept a **3–8 digit** numeric suffix by default, and the maximum is **configurable**:

```toml
# .syscribe.toml
[ids]
max_digits = 8   # default 8; raise (e.g. 12) or lower (e.g. 4). Minimum stays 3.
```

- Existing 3-digit IDs are unchanged; up to 8 digits works out of the box.
- A suffix longer than the cap is the new error **`E023`**; shorter than 3 is still `E006`.
- A reference to an over-long ID still resolves — the defect surfaces on the ID-bearing element, not as a dangling reference.

Requirement-first: `REQ-TRS-ID-005` + `TC-TRS-ID-005`. Suite at **145** test cases, all passing.

## 0.16.2 — 2026-06-10

### Docs site

- Aligned the §9 table-of-contents anchors in the published format specification so all TOC links resolve on the site; the fix (em-dash → colon in three headings) keeps the same anchors working on GitHub's renderer too. `mkdocs build --strict` is clean.

## 0.16.1 — 2026-06-10

### Docs site

- The canonical format specification (`spec/markdown-sysml-format.md`) is now published on the documentation site under **Format ▸ Full Specification** (a symlink — single source of truth).
- Fixed two broken links on the CLI reference page that pointed outside the docs tree; `mkdocs build --strict` is clean again.

## 0.16.0 — 2026-06-10

### TestPlan documentation + `template TestPlan`

- **`syscribe template TestPlan`** — emits a ready-to-fill TestPlan frontmatter skeleton (TP-* id, title, status, scope, a `testCases` member, with `configurations`/`demonstrates`/`selection` shown as commented optional fields); `TestPlan` is now listed among the `template` command's known native types.
- **Documentation** — the `TestPlan` element (0.13.0/0.15.0) and `custom_fields` (0.14.0) are now fully documented across the format spec (§8.12.6), the embedded `prompts/spec` and `create-model` guides, the site format/validation references, and a new *Test Plans* model-guide page.

Suite at **144** test cases, all passing.

## 0.15.0 — 2026-06-10

### `audit --plan` — scoped readiness verdict (GH #40)

Completes the `--plan` lens (deferred from #38): `audit --plan TP-X` scopes the readiness verdict to a TestPlan. Validation runs over the **whole** model — so a reference escaping the plan subset (a requirement's `breakdownAdr` ADR, a member's out-of-scope `verifies:` target) resolves and is never mistaken for a defect — and only findings on the plan's in-scope elements (in-scope requirements ∪ member TestCases ∪ their satisfying architecture) count toward the verdict. The dashboard sections are scoped to the plan but resolve references against the full model. Composes with `--config`; exits `1` on an unknown plan id; dormant-safe.

`--plan` is now available on `matrix`, `verification-depth` **and** `audit`. Suite at **144** test cases, all passing.

## 0.14.0 — 2026-06-10

### User-defined custom fields (GH #39)

A dedicated `custom_fields:` frontmatter map lets you attach arbitrary user data to any element — making it intentional and addressable instead of being silently swallowed by the unknown-key catch-all.

- **Schema** — `custom_fields:` is a flat `string → scalar | list-of-scalars` map, accepted on every element type, serialised in stable sorted order.
- **Validation** — `W041` shape-check: a value that is not a scalar or a list of scalars (e.g. a nested map) is flagged, naming the key. Freeform keys, no registration.
- **Query** — `--where custom.<key>` on `ls` / `find` / `list`: exact (`=`), regex/substring (`=~`), list-membership (`~=`), and bare presence; composes (AND) with the type/tag/status filters.
- **Rendering** — a `## Custom Fields` section in CLI `show`, and a read-only key/value table in the web detail panel (never editable via the element editor).

Requirement-first: `REQ-TRS-CFLD-001..003`, `TC-TRS-CFLD-001..003` with fixtures. Suite at **144** test cases, all passing.

## 0.13.0 — 2026-06-10

### Native `TestPlan` element (GH #38)

A first-class `TestPlan` (`type: TestPlan`, stable `TP-*` id) groups TestCases into the unit a team executes and reports against. TestCases stay reusable atoms; a plan is a curated, per-product artifact.

- **Schema & validation** — `scope` (recommended vocab + free-form), `configurations:` (scalar/list of `Configuration`s, or absent = config-agnostic), `demonstrates:` (optional safety-case evidence link), `testCases:` + an additive `selection:` query (`testLevels`/`domains`/`tags`). New rules `E600`–`E606` and `W610`–`W616`: malformed id, unresolvable member/config/demonstrates target, bad selection levels/domains, non-recommended scope, escaping member, empty plan, pinned-draft member, demonstration gap, results-gated failing member, duplicate `(configurations, scope)`. `W614` honours **goal-closure** — demonstrating a parent goal whose leaves are tested does not false-fire (cf. GH #37).
- **`testplan` command** — `testplan` lists every plan (scope, configs, effective-TC count, coverage %, verdict); `testplan TP-X` shows members (with escaping flags), in-scope requirements, a per-config coverage grid and a rolled-up verdict (`pass|fail|incomplete|empty`); `--json` on both. Coverage and verdict reuse the existing `matrix`/results machinery.
- **`--plan TP-X` lens** — on `matrix` and `verification-depth`: restricts rows to the plan's in-scope requirements and the TestCase universe to its members; composes with `--config`. (`audit --plan` is deferred — GH #40.)
- **`matrix --config`** now reduces the grid to the selected Configuration's column.

Requirement-first: `REQ-TRS-PLAN-001..006`, `TC-TRS-PLAN-001..006` with fixtures covering every code. Suite at **141** test cases, all passing.

## 0.12.1 — 2026-06-09

### Fixes

- **`audit` no longer mis-lists a parent requirement as unsatisfied/unverified (GH #37).** A parent (a `Requirement` with `derivedChildren`) is satisfied and verified only *transitively* through its leaves and can never be satisfied directly (§12.4 / `E312` forbid it appearing in any `satisfies:` list). The orphan section nonetheless flagged every such parent under `unsatisfiedRequirements` (and structurally under `unverifiedRequirements`), disagreeing with `validate`, which already suppresses parents in W002/W300/W306. The orphan loop now skips parents from both sets; genuine gaps still surface on the leaf requirements. Covered by a new parent-rollup fixture under `TC-TRS-OUT-013`.

## 0.12.0 — 2026-06-09

### Configuration lens on the analysis & audit commands

- **`audit` now honours `--config` (GH #35).** The dashboard was always computed over the 150% superset, so a requirement gated out of a variant still tripped the verdict. `audit --config <CONF|features>` now projects the entire dashboard — verdict, `W306`, orphan sets and coverage — onto the elements **active** in that Configuration, exactly like `validate --config`. New **`audit --all-configs`** audits every stored `Configuration`'s variant and exits non-zero if any fails (the product-line CI gate).
- **`audit --config` and `validate --config` now agree (GH #36).** A `TestCase` that survives the projection but whose `verifies:` target was projected out was mis-counted as a dangling *error-severity* finding. The verdict now uses the projection-aware `validate_projected` path, and dangling detection considers only the active `TestCase`s while resolving their references against the **full** model — so the two commands report the same error count for a variant.
- **The `--config` lens extends to the other read-only safety/security commands.** `metrics`, `cyber-risk`, `co-analysis`, `verification-depth` and `safety-case` each accept `--config <C>` and compute their report over the projected active subset (dormant when no feature model; unresolvable `--config` exits `1`).

Requirement-first: `REQ-TRS-OUT-013` (audit lens + #36 agreement) and new `REQ-TRS-PROJ-006` (lens on the analysis commands), with `TC-TRS-OUT-013` / `TC-TRS-PROJ-006` harnesses and fixtures. Suite at **135** test cases, all passing.

## 0.11.1 — 2026-06-09

### Fixes

- **`W306` no longer flags a satisfied-via-leaf parent requirement (GH #34).** The "unsatisfied" sub-condition demanded a *direct* satisfier, contradicting `E312` (a parent requirement may not appear in a `satisfies:` list — it is satisfied transitively through its leaves). High-integrity parents were therefore permanently flagged and `audit` could never PASS on a hierarchical model. The sub-condition now applies to **leaf** requirements only (mirroring the existing W002 parent suppression); genuine gaps still surface on the unsatisfied leaf, and `status: draft` / active-in-no-config still apply to parents. Covered by a new satisfied-via-leaf parent fixture in `TC-TRS-TRACE-010`.

## 0.11.0 — 2026-06-09

A large safety/security + tooling release. Every feature is requirement-first (a `REQ-TRS-*` + `TC-TRS-*` with fixtures); the tool-qualification suite grew to **134 test cases**, all passing.

### Safety & security analysis (ISO 26262 / IEC 61508 / ISO/SAE 21434)
- **Safety ↔ security co-engineering** — `hazardRef` links `DamageScenario`/`ThreatScenario` to the `HazardousEvent`/`SafetyGoal` they endanger (`E844`); `W030` flags a safety-tagged damage scenario with no link; new **`co-analysis`** command shows which cyber threats can violate each safety goal.
- **Cybersecurity risk determination** — computed risk (severity × feasibility) per `ThreatScenario`, `riskTreatment`/`residualRisk` fields, `W031` (untreated high/critical threat) and `W032` (CAL below risk), new **`cyber-risk`** command.
- **Quantitative HW metrics** — `diagnosticCoverage`/`latentDiagnosticCoverage`, SPFM/LFM/PMHF computed per `SafetyGoal` vs ASIL/SIL target (`W033`), new **`metrics`** command. First-order FMEDA approximation.
- **Freedom from interference** — `W034` flags differing-integrity elements sharing an allocation target without an `ffiRationale`.
- **Attack trees** — new `AttackTree`/`AttackTreeGate`/`AttackStep` types with weakest-link feasibility roll-up (AND=min, OR=max) reconciled against the linked threat (`W035`).
- **Confirmation measures & DIA** — `responsibility` field (`W038`), `ConfirmationMeasure` type + `W039` (missing independent assessment for ASIL-D/CAL4).
- **GSN safety case** — `Argument`/`AssumptionOfUse` types and a **`safety-case`** command rendering the goal → argument → evidence tree.
- **Unsatisfied safety mechanism** `W306`; **WCET evidence** `W029` + `list --has-wcet`.

### Reports, queries & CLI
- New **`audit`** (safety-readiness dashboard with PASS/FAIL), **`verification-depth`** (independence report + `--min-levels` gate), **`connectivity`** (element-rooted subgraph as text/DOT/JSON), **`extref`** (lookup by external reference).
- `list`/`matrix` gain `--status`/`--sil`/`--has-wcet`/`--gaps-only` filters, coverage-% footers, JSON, and executed-evidence glyphs/annotations in `matrix`/`trace` from ingested results.
- **Named severity profiles** — `.syscribe.toml` `[profiles.*]` (SIL/ASIL-scopable code promotion) via `validate --profile` / `audit --profile`.
- **`extRef`** common field (external-tool references) + `W028` duplicate check.
- **Model-root auto-discovery** — `.syscribe.toml` walk-up; `-m` stays primary (backward-compatible).
- **Detailed per-command help** — `syscribe help <command>` and `syscribe <command> --help`/`-h` (man-page style for every command), plus `syscribe help` index.

### Format & correctness
- **`bindingTime`** on feature parameters (`compile`/`load`/`runtime`) with ordering (`E229`) and value (`E230`) checks, `W027`, and `W017` suppression.
- **`typedBy` cycle/self-reference** now detected (`E107`) — previously silently accepted.
- **Ports & Interfaces modeling guide** (SysML v2) added to the spec, `syscribe spec types`, and the LLM prompt.
- Discoverability: the full safety/security field & type set is now documented in `syscribe spec fields`/`types`/`safety`, and all new commands/types are in the `--agent-instructions` LLM prompt.

## 0.10.0 — 2026-06-08

### Model-root auto-discovery (REQ-TRS-CLI-004)

- With **no** `-m`/`--model` flag and **no** `SYSCRIBE_MODEL`, the tool now walks up from the current working directory to the nearest ancestor containing a **`.syscribe.toml`** and uses that directory as the model root — run any command from anywhere inside the model tree (the `git`/`cargo` ergonomics). Full resolution order: `--model` → `SYSCRIBE_MODEL` → walk-up to `.syscribe.toml` → the literal `model/` default.
- The marker is the existing config file (`repo_root` / `[matchers]` / `[remote]`); an **empty** `.syscribe.toml` is a valid root marker. It is a tooling locator only — it never affects qualified-name resolution or the implicit root namespace.
- **Fully backward-compatible**: the flag, env var, and `model/` default behave exactly as before; a tree with no marker falls straight through to the default.

### Tests

- `REQ-TRS-CLI-004` + `TC-TRS-CLI-004` (discovery from a subdirectory, explicit `-m` override, explicit `-m` on a model with no marker, and fallback-with-miss). Tool-qualification suite 115 → **116** test cases, all passing.

## 0.9.0 — 2026-06-08

### Feature-parameter binding time (REQ-TRS-PARAM-004)

- A `FeatureDef` parameter may declare an optional **`bindingTime:`** — *when* its value is resolved, from the product-line-engineering triad ordered earliest→latest: `compile` (build / codegen) · `load` (deployment / startup) · `runtime` (live). Orthogonal to `isFixed:`/`value:` (a value fixed in the model); an absent binding time is unspecified and opts out of the checks.
- **`E230`** — an unrecognised `bindingTime:` value (`validate`). **`E229`** — a parameter computed from a `derivedFrom`/`bindTo` source it depends on cannot bind *earlier* than that source (`feature-check`; checked only when both ends declare a binding time). **`W027`** — a `Configuration` that binds a `runtime` parameter (resolved by the running system, not at configuration time); for the same reason `W017` is suppressed for an unbound `runtime` parameter. The `feature`/`features` cards show `param [bindingTime]`.

### External references on all elements (REQ-TRS-EXTREF-001/002)

- New optional **common** field **`extRef`** (string or list) on **every** element type — marks an element as the representation of an artifact in another tool (a requirement in DOORS Next, an element in a SysML tool, a ticket). Opaque values (a URI or a `tool:id` token); syntax unconstrained; never a model cross-reference target.
- New **`extref <ref> [--json]`** command looks up the element(s) carrying a reference (exact match, returns all matches, exits non-zero on a miss). **`W028`** warns when the same `extRef` is declared by two or more elements. `show` surfaces the field and `spec fields` lists it.

### Tests

- Requirement-first: `REQ-TRS-PARAM-004` + `REQ-TRS-EXTREF-001/002` with `TC-TRS-PARAM-004` and `TC-TRS-EXTREF-001/002` and fixtures. Tool-qualification suite 113 → **115** test cases, all passing.

## 0.8.1 — 2026-06-08

### Fixes (GH #14 re-open)

- **Parameter `range:` is now enforced by `feature-check`, not only `validate`.** The binding rules (`E203`–`E206`/`E222`/`W017`) were extracted into a shared check so a product line validated holistically (`feature-check`) gets the same range/binding enforcement — an out-of-range binding (e.g. `99` against `range: "1..=8"`) now fires `E205` under `feature-check`.
- **A `parameterConstraints` expression that used the legacy `::`-member form (`Features::F::param`) was silently dropped** (no error, no evaluation). It now raises `E213` with a hint to use the canonical dotted form `Features::F.param` — a declared constraint can never silently no-op.
- **Hardened the constraint arithmetic tokenizer** to reject unexpected characters (previously swallowed, so a stray operator could yield a spurious value).

### Tests

- New `feature_model` unit tests for the constraint evaluator: all comparison operators, arithmetic precedence/parentheses/unary-minus, unresolved/malformed → no-match, and `range:` parsing (incl. `..=`). Model unit tests 33 → 40.

## 0.8.0 — 2026-06-07

### Typed feature-parameter constraints enforced (closes #14)

- **`E221`** — `feature-check` now evaluates `parameterConstraints` expressions (numeric comparison `== != >= <= > <` over `+ - * /` arithmetic of literals and parameter references) against every `Configuration` whose `appliesWhen:` predicate holds; a violation is an error. **`W025`** — the same violation when the constraint declares `severity: warning`. Both gateable with `--deny`.
- Compound `appliesWhen:` on `parameterConstraints` is now boolean-parsed (`and`/`or`/`not`), fixing a spurious `W014`.
- `range:` now accepts the inclusive form `"min..=max"` as well as `"min..max"`, so `E205`/`E202` actually fire (a `1..=8` range was previously dropped silently).
- **Schema:** a feature-parameter reference is now the canonical **dotted** form `Features::Feature.param` (a single `.` before the parameter member) everywhere — `parameterBindings:` keys, `parameterConstraints` expressions, and `bindTo:` targets. The legacy all-`::` member form is rejected (`E222`). Existing fixtures and the demo model were migrated.

### Transitive package `appliesWhen` (REQ-TRS-VAR-006)

- A **`Package`** (`_index.md`) may declare `appliesWhen:` to gate its **whole subtree** — enabling/disabling a cohesive variant of requirements + architecture + tests with one declaration. An element's *effective condition* is its own `appliesWhen:`, else the nearest ancestor package's, else always-active; conditions are never combined.
- **`E228`** — invalid placement (at most one declaration per root-to-leaf path): a nested declaration, or `appliesWhen:` on a `FeatureDef`/`Configuration`, a package whose subtree contains one, or the model root. **`W026`** — a gated package with an empty subtree.
- All consumers honour the effective condition: `--config` projection, escaping refs (`E226`/`W019`), `matrix`, `why-active` (now shows "inherited from package"), feature-card gates, `list --feature`, and `feature-check --deep` edges.

### Tests

- The `appliesWhen` boolean grammar is now covered by an exhaustive oracle (3000 random expression ASTs evaluated across all assignments), precedence-vs-parentheses checks, and operator-substring/whitespace/double-negation edge cases.

## 0.7.0 — 2026-06-07

### Feature discoverability commands

Five read-only commands for navigating a product line, plus an orphan-feature check:

- **`features [--json]`** — the feature model as a tree: each `FeatureDef`'s `groupKind`, `requires`/`excludes`, parameters, and a "selected in N/M configs" rollup.
- **`feature <qname> [--json]`** — one feature's card: doc, group, constraints, parameters, the `Configuration`s that select it, and every element it **gates** (whose `appliesWhen:` names it).
- **`matrix --features`** — a Feature × Configuration grid (which feature ships in which product), complementing the Requirement × Configuration view.
- **`list <type> --feature <F>`** — restrict a listing to elements whose `appliesWhen:` names `F` (orthogonal to `--tag`/`--config`).
- **`why-active <element> --config <C>`** — explain a projection: the element's `appliesWhen:`, the config's relevant selections, and a `Verdict:` of `active` / `inactive` / `always active`.
- **`W024`** — `feature-check` now flags an **orphan feature** (referenced by no `appliesWhen:` and selected by no `Configuration`); gate with `feature-check --deny W024`. Never emitted by base `validate`.

### Feature-model schema: `mandatory:` membership (ADR-FM-003)

A new optional boolean **`mandatory:`** on `FeatureDef` separates *membership* (relative to the parent) from *grouping* (`groupKind`, which now governs child layout only). A node can be both `mandatory: true` and `groupKind: alternative` — a **mandatory XOR group** (every product selects exactly one child). Backward compatible: the legacy `groupKind: mandatory` remains a shorthand for `mandatory: true` on a leaf.

### UAV model is now a full product line

The bundled `model/` is a runnable 150% UAV product line: a feature model (`Features/`) with three mandatory XOR groups (Propulsion/Payload/DataLink), an optional `DualFlightController`, cross-tree `requires`, and a typed parameter; three products (`Configurations/CONF-UAV-*`); variant-conditioned architecture, requirements, and tests under `ADR-SYS-PLE-001`; and `implementedBy:` traces into `firmware/`. Every variability command runs against it cleanly (`feature-check --deep`, `matrix`, `validate --all-configs`). With the new `mandatory:` field the earlier synthetic `Base` workaround feature was removed.

Documented in the [variability guide](../model-guide/variability.md), [modeling guide](../model-guide/index.md), [CLI reference](../cli/index.md), format spec §9, and `syscribe spec` prompts.

## 0.6.0 — 2026-06-07

### Implementation trace (`implementedBy:`, closes #13)

Closes the downstream leg of the V-model: `Requirement ─satisfies→ Architecture ─implementedBy→ Code ─verifies→ Test`.

- **`implementedBy:`** — a new optional field on `Part`/`PartDef` (string or list) linking an architecture element to the source artifact(s) that realise it. Paths resolve with the same rules as a TestCase's `sourceFile` (model-/repo-relative, `model:`/`repo:` prefixes, absolute, `file://`, remote `scheme://`).
- **W023** — a non-`draft` `Part`/`PartDef` whose `implementedBy:` path is missing on disk (one finding per missing path). Opt-in (only when `implementedBy:` is present), draft-suppressed, remote-tolerant, and gateable with `validate --deny W023`.
- **Discoverability** — `links <element>` lists `implementedBy` paths; `refs <path-or-dir>` reverse-maps a source path (or directory prefix) back to the declaring architecture element(s).

Documented in format spec §12.8, `syscribe spec validation`/`spec fields`, the validation-rule reference, the traceability guide, and the LLM authoring prompt.

## 0.5.0 — 2026-06-06

### Configuration selections (fixes #12)

- `template Configuration` now emits the canonical `features:` map (was `selections:`, which the parser silently ignored — every cell came back N/A).
- **W016** — a `Configuration` that parses zero feature selections while a feature model exists is now flagged instead of silently ignored.
- `show <Configuration>` displays parsed feature selections (and `featureModel`/`appliesWhen`), so a parse failure is visible locally.

### Feature parameters (§9.7, single-level)

- `FeatureDef.parameters:` are now validated against a `Configuration`'s `parameterBindings:`:
  - **E203** bind for an unselected feature · **E204** bind a fixed parameter · **E205** value out of `range:` · **E206** value not in `enumValues:` · **E222** binding path resolves to no declared parameter · **W017** selected feature's required, default-less parameter left unbound.
- Two-level `bindTo:` propagation, derived-expression cycles, and cross-feature `parameterConstraints` remain unimplemented (documented).

## 0.4.0 — 2026-06-06

### Product-line variability (opt-in)

The variability dimension stays dormant — and changes nothing — unless the model declares a `FeatureDef` and links something to it.

- **Boolean `appliesWhen:`** — conditions any element (including a `TestCase`) on an expression over `FeatureDef` qualified names: `and` / `or` / `not` / parentheses. Bare QName and list (AND) forms remain valid. `E209` now checks every operand.
- **`TestCase` variant membership** is derived from `appliesWhen:` — a test runs in a `Configuration` iff its condition holds for that configuration's selections (no `runsIn` field).
- **`syscribe matrix [--json] [--tag]`** — Requirement × Configuration coverage grid (covered / gap / N-A); falls back to a flat view when no feature model is present.
- **W015** — per-`Configuration` coverage rule: a requirement active in a configuration with no covering in-config `TestCase`. Honours draft suppression; gate with `--deny W015`.
- **`list --tag` / generic tags** — free-text `tags:` filtering across `list` and `matrix`, orthogonal to the feature model.
- `Configuration.features:` selection maps now parse; `refs <CONF>` lists the TestCases that run in a configuration.

Documented in format spec §9.10–9.11, `syscribe spec validation`/`spec fields`, the CLI help, and the LLM authoring prompt (Part 9b).

## 0.3.0 — 2026-06-06

- CI severity gating for `validate` (`--deny`, `--max-warnings`, `--warnings-as-errors`; exit codes 0/1/2)
- Function-level traceability (`W009`) and structured model-graph `export` (JSON / NDJSON)
- Gherkin scaffolding (`scaffold-gherkin`) and test-result ingestion (`ingest-results`, `W010`)
- Atomic `move` with reference rewriting
- `sourceFile` location semantics (model/repo-relative, absolute, `file://`, remote) with an opt-in `--fetch-remote` download hook
- Active-only source-drift scoping with informational `I010`

## 0.2.0 — 2026-05-28

### Demo models

- **Engine ECU** (`model_auto/`) — full ISO 26262 / ISO/SAE 21434 reference model: ASIL A–D safety goals, HARA, FTA, FMEA, TARA, 14 test cases
- **SIL 4 Computer-Based Interlocking** (`model_sil/`) — full IEC 61508 / EN 50128 / EN 50159 reference model: SIL 4 2oo2D architecture, formal B-Method specification obligation, quantitative FTA (< 10⁻⁸ /h), 11 test cases
- Separate documentation pages for each demo model in the docs

### CLI

- Model root is now set with `-m <path>` / `--model <path>` or the `SYSCRIBE_MODEL` environment variable — positional argument removed
- New `spec` subcommand: `syscribe spec`, `syscribe spec types`, `syscribe spec fields`, `syscribe spec validation`, `syscribe spec traceability`, `syscribe spec safety` — in-terminal format spec browser
- New `trace`, `why`, `who-verifies` commands for traceability queries
- New `next-id`, `template`, `check-ref`, `path-for` commands for model authoring
- New `validate --json` flag for machine-readable output
- New `types` and `untyped` commands for model inspection

### Validation

- **Safety/security integrity level rules**: E841/E842/E843 (ASIL/SIL consistency across inheritance and deployment), W808 (integrity level not set on safety-critical element)
- **W806**: SafetyGoal not grounded in any HazardousEvent or HARA element
- **W305**: parent Requirement must have at least one integration-level (L3/L4/L5) TestCase
- **W410/W411/W412**: cross-reference target existence checks
- **W408/W409**: `%% ref:` annotation validation in Mermaid diagram blocks
- Various false-positive fixes (W006, W007, W406/W407 SVG-internal IDs)

### Format

- `derivedFromSafetyGoal:` link from Requirement to SafetyGoal (IEC 61508 / ISO 13849)
- `derivedFromSecurityGoal:` link from Requirement to CybersecurityGoal
- `allocatedFrom` / `allocatedTo` now accept lists for multi-source allocation
- Full Tier 4 safety analysis elements: FaultTree (file-per-node), FMEA (exploded entries), TARASheet (exploded container)
- Full safety/security analysis documentation added to the Modeling Guide

### Web browser

- Interactive Cytoscape.js model canvas at `/canvas`
- Validation errors and warnings highlighted on canvas nodes
- Validation findings shown in element detail panel
- Element documentation body rendered as Markdown with embedded Mermaid diagram support
- `diagram` CLI with `compose`, `layout`, `expose` sub-commands; Cassowary-based layout solver
- SEQ and REQ diagram renderers; A\* obstacle-aware edge routing; full SysML edge style set
- JS dependencies vendored and served from the embedded binary (no external CDN at runtime)
- Unified engineering blueprint colour scheme across docs and canvas

---

## 0.1.0 — 2026-05-26

Initial public release.

### Format

- Full Syscribe format specification (§1–§12)
- 40+ element types covering SysMLv2 structural, behavioral, and requirements constructs
- Native `Requirement` (REQ-*), `TestCase` (TC-*), `ADR` (ADR-*), and `Configuration` (CONF-*) elements
- `operations:` field (§8.3.4) on PortDef/InterfaceDef for synchronous operations and async receptions
- Six §12 traceability rules enforced by the validator

### Validation engine

- 80+ validation rules across 12 groups (E001–E503, W001–W601)
- Computed reverse indices: `verified_by`, `derived_children`
- CLI report tool: `syscribe -m model/` — 10-section Markdown output

### Web browser

- Axum + Askama + HTMX — no JavaScript framework
- BDD, IBD, StateMachine, and Requirement diagram rendering (SVG, server-built)
- Mermaid diagram rendering (client-side, CDN)
- Drag-to-reposition with layout persistence to `.md` files
- WebSocket live reload on file-system changes

### Demo model

- UAV system — 111 elements across 20 packages
- 9 native Requirements (3 parents, 6 leaves), 9 active TestCases, 2 ADRs
- Full §12 traceability: domain classification, breakdownAdr, satisfaction links
- 5 diagrams: BDD, IBD, StateMachine, Requirement, Mermaid
