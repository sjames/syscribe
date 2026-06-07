# Releases

`RELEASES`

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
