# Releases

`RELEASES`

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
