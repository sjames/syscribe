# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

- **Working directory**: `/home/sjames/playground/syscribe`
- **Rust crates**: `syscribe` (CLI binary), `syscribe-model` (library), `syscribe-server` (web server)

## Project Overview

This repository has two components:

1. **Syscribe Format** — a human- and LLM-friendly format for systems models that replicates the semantics of SysMLv2, using Markdown files with YAML frontmatter and directory hierarchy as structural elements of the model.
2. **Web Browser + Validator** — a Rust/Axum server that parses and serves the model with a browser-based UI, and a CLI validation tool.

Reference material: `temp/sysml2_spec.pdf` (SysMLv2 language spec) and `temp/formal-26-03-04.pdf`.

## Common Commands

```bash
# Build everything (produces syscribe and syscribe-server binaries)
cargo build --workspace

# Validate the demo model (prints Markdown report to stdout)
cargo run --package syscribe -- -m model/
# or after build: ./target/debug/syscribe -m model/

# Print the LLM model generation prompt
cargo run --package syscribe -- --agent-instructions

# Print the tool version (also -V, or `version`)
cargo run --package syscribe -- --version

# Start the web server
cargo run --package syscribe-server -- -m model/

# PlantUML companion diagrams
./target/debug/syscribe -m model/ plantuml               # generate all .puml companion files
./target/debug/syscribe -m model/ plantuml render        # render .puml → .svg (needs plantuml on PATH or PLANTUML_JAR set)
./target/debug/syscribe -m model/ plantuml render --jar /path/to/plantuml.jar
./target/debug/syscribe -m model/ plantuml render --dry-run   # preview which files would be rendered

# Variability / product-line commands (opt-in; see docs/model-guide/variability.md)
./target/debug/syscribe -m model/ matrix                 # Requirement × Configuration coverage grid
./target/debug/syscribe -m model/ feature-check          # holistic feature-model validation
./target/debug/syscribe -m model/ feature-check --deep   # SAT-backed: void/dead/core/false-optional, config validity, diagnoses
./target/debug/syscribe -m model/ configure CONF-X       # assisted configuration (forced/free features)
./target/debug/syscribe -m model/ validate --config CONF-X     # validate one projected variant
./target/debug/syscribe -m model/ validate --all-configs       # CI gate over every Configuration
./target/debug/syscribe -m model/ diff --config CONF-A --config CONF-B

# Multi-repository composition (opt-in; see docs/model-guide/multi-repo.md)
./target/debug/syscribe -m model/ repos list             # configured peer repos + on-disk/sync status
./target/debug/syscribe -m model/ repos status           # whether each pinned repo is at its ref (exit 2 on drift)
./target/debug/syscribe -m model/ repos sync --all       # git fetch + checkout <ref> for pinned repos
./target/debug/syscribe -m model/ validate --deny W511 --deny W512   # gate CI on ref drift / submodule gitlink
```

The feature-model SAT engine is `batsat` (vendored, MIT, pure Rust — `ADR-FM-002`); the in-tree clause IR + brute-force oracle live in `crates/syscribe-model/src/{solver,projection,feature_model}.rs`.

Multi-repo composition (§14) is config-driven: `[repos]` in `.syscribe.toml` + `repoImports:` on a package `_index.md`. The loader (`crates/syscribe-model/src/config.rs`) walks each peer, indexes qnames/stable-IDs, and precomputes git ref/gitlink state (`RefState`); the validator emits `E510`–`E515`/`W510`–`W512` only when `[repos]` is configured.

---

## Part 1 — Syscribe Format

### Core Idea

Each model element is a `.md` file. The **directory path** encodes namespace/ownership (analogous to SysMLv2 package nesting). YAML frontmatter declares the element's type and metadata; the Markdown body is the documentation (`doc` annotation in SysML terms).

### Element Mapping

| SysMLv2 construct | Syscribe representation |
|---|---|
| `package` | Directory (with optional `_index.md` carrying frontmatter) |
| `part def` / `item def` | `.md` file, `type: PartDef` / `ItemDef` |
| `part` / `item` | `.md` file, `type: Part` / `Item` |
| `port def` / `port` | `.md` file, `type: PortDef` / `Port` |
| `connection def` / `connect` | `.md` file, `type: ConnectionDef` / `Connection` |
| `attribute def` / `attribute` | YAML key inside parent's frontmatter (for simple scalars) or own `.md` file |
| `action def` / `action` | `.md` file, `type: ActionDef` / `Action` |
| `requirement def` / `require` | `.md` file, `type: RequirementDef` / `Requirement` |
| `interface def` / `interface` | `.md` file, `type: InterfaceDef` / `Interface` |
| — (native Requirement) | `.md` file, `type: Requirement` — dedicated handler, REQ-* id |
| — (native TestCase) | `.md` file, `type: TestCase` — dedicated handler, TC-* id |
| — (native TestPlan) | `.md` file, `type: TestPlan` — dedicated handler, TP-* id; groups TestCases by configuration/scope |
| — (Architecture Decision Record) | `.md` file, `type: ADR` — dedicated handler, ADR-* id |
| `allocation` | `.md` file, `type: Allocation` |
| `view def` / `view` | `.md` file, `type: ViewDef` / `View` |

### Frontmatter Schema (common fields)

```yaml
---
id: <qualified-name>          # auto-derived from path if omitted
type: <SysML element type>    # required
name: <display name>          # defaults to filename stem
extRef: <string or list>      # external reference(s) (e.g. DNG:4521)
supertype: <qualified-name>   # specialization (':>')
subsets: [<qualified-name>]   # subsetting ('::>')
redefines: <qualified-name>   # redefinition
multiplicity: "1"             # default 1
isAbstract: false
features:                     # inline attribute / port declarations
  - name: mass
    type: ScalarValues::Real
    unit: kg
connections:                  # for Part files — bind ports
  - from: subpartA::outPort
    to: subpartB::inPort
---
```

### Directory / Namespace Convention

```
model/
  _index.md              # root package metadata
  VehicleSystem/
    _index.md            # VehicleSystem package
    Chassis.md           # part def Chassis
    Powertrain/
      _index.md
      Engine.md
      Transmission.md
  Requirements/
    SafetyReqs.md
```

A file at `model/VehicleSystem/Powertrain/Engine.md` has qualified name `VehicleSystem::Powertrain::Engine`.

### Cross-references

Use qualified names (`::`-separated) to reference elements in other files. The parser resolves them relative to the model root.

```yaml
supertype: VehicleSystem::Chassis
subsets: [Interfaces::PowerInterface]
```

### Name Scheme

Element **identity segments** (the file/directory stem, which becomes the last qualified-name segment for name-identified types) follow the SysMLv2 **basic-name** grammar: `^[A-Za-z_][A-Za-z0-9_]*$` — letters, digits and `_`, not starting with a digit. **No hyphens or spaces** (a hyphen is the subtraction operator in `appliesWhen`/`parameterConstraints` expressions, so a hyphenated name cannot be referenced there). A non-basic identity segment raises warning `W042`; rename using `_` or CamelCase (`Anti-Lock` → `Anti_Lock`/`AntiLock`). Stable ids (below) are exempt — they legitimately contain `-`. For **id-identified types** the identity segment is the `id`, so `W042` governs the `id`, **not** the free-text `name` (which may contain spaces and punctuation).

### Label field: `name` is the single universal label

Every element type — `Requirement`, `TestCase`, `ADR`, `PartDef`, `Package`, `FeatureDef`, the safety/security types, **everything** — uses **`name`** as its one human-readable label (SysMLv2 `declaredName`). There is no longer a per-identity-class split.

- **`name`** is the label on all types. It is **optional in general** but **required** on every type that previously required a label (the native and safety/security id-identified types — see "ID Scheme" below). For id-identified types `name` is **free prose** (spaces and punctuation allowed; `W042` does **not** apply). For name-identified types `name` is **both** the label **and** the identity segment, so the basic-name grammar (`W042`) applies to it.
- **`id`** (the stable shortName) remains the **identity** of the id-identified types — files stay `<id>.md` and cross-references (`verifies:`, `derivedFrom:`, …) resolve by id. The `id` axis and the `name` (label) axis are independent.
- **`title` is removed.** It is no longer a recognized label field on any element. Declaring `title:` on **any** element raises error **`E025`** ("the `title` field is removed — rename it to `name`"), type-independently. Error **`E024`** (formerly: `name` on an id-identified type) is **retired** — `name` is now the correct, required label on those types, so a `Requirement` carrying `id` + `name` validates clean.

`FeatureDef` is name-identified (its `name` is its label *and* qualified-name segment) and **also** carries a **mandatory** stable `id` (`FEAT-*`, its shortName; a feature with no `id` is error `E201`). Under the unified rule it carries `name` (label) + `id` (shortName) and no `title`.

### ID Scheme

SysML elements (`PartDef`, `Port`, etc.) use `id` auto-derived from the file path if omitted. `Requirement` and `TestCase` carry a **stable opaque identifier** that must be explicitly set and never changes.

**Requirement ID pattern** — `^REQ(-[A-Z0-9]{2,12})+-[0-9]{3,8}$`
- Prefix `REQ`, one or more uppercase-alphanumeric segments (2–12 chars), 3–8 digit numeric suffix (default cap 8; configurable via `[ids] max_digits`, minimum 3).
- Examples: `REQ-SCHED-001`, `REQ-SCHED-BITMAP-001`, `REQ-PORT-CTX-001`

**TestCase ID pattern** — `^TC(-[A-Z0-9]{2,12})+-[0-9]{3,8}$`
- Same segment rules, prefix `TC`. Test level is not encoded in the ID (lives in `testLevel:`).
- Examples: `TC-SCHED-BITMAP-001`, `TC-SYNC-SEM-002`

**ADR ID pattern** — `^ADR(-[A-Z0-9]{2,12})+-[0-9]{3,8}$`
- Same segment rules, prefix `ADR`. Statuses: `proposed | accepted | deprecated | superseded`.
- Examples: `ADR-SYS-001`, `ADR-SW-SCHED-001`

**FeatureDef ID pattern** — `^FEAT(-[A-Z0-9]{2,12})+$`
- Prefix `FEAT`, **mandatory** on every `FeatureDef` (a feature with no `id` is `E201`). Unlike other stable ids, a feature id **need not** end in a number — `FEAT-ABS` and `FEAT-ABS-001` are both valid. The feature stays name-identified (label/qname = `name`); the id is a stable reference usable in `appliesWhen:` and `Configuration` `features:` keys interchangeably with the qname.

`Configuration` (`CONF-*`) is id-identified: its identity is its `id`, files are `<id>.md`, and its label is a free-prose `name`.
- Examples: `FEAT-ABS`, `FEAT-QUADROTOR`, `FEAT-DATALINK-LORA`, `FEAT-CAM-4K`, `FEAT-ABS-001`

Both the `id` field and the qualified name (path-derived) are valid cross-reference targets in `verifies:` and `derivedFrom:`.

**Configurable additional prefixes** — every id-identified type accepts its built-in prefix (`REQ`, `TC`, `ADR`, … `FEAT`); a project may declare **extra** prefixes per type in the `[ids.prefixes]` table of `.syscribe.toml`, keyed by element-type name. They are **additive** (the built-in always stays valid) and **pure identity** (they affect only id validation/resolution). Each prefix must match `^[A-Z][A-Z0-9]{1,11}$`; a malformed or unknown-type entry raises warning `W046` and is ignored. Example: `[ids.prefixes]` with `Requirement = ["STK", "SYS"]` makes `STK-SCHED-001` and `SYS-SCHED-001` valid `Requirement` ids alongside `REQ-SCHED-001`. (REQ-TRS-ID-007.)

---

## Part 1B — Native Requirement and TestCase Elements

The native `Requirement` and `TestCase` types are first-class elements for safety-critical engineering. Full specifications are in `spec/markdown-sysml-format.md` (the canonical format spec):

- **§8.11.6** — Native `Requirement`: stable `REQ-*` IDs, lifecycle status, SIL/ASIL fields, normative body text.
- **§8.12.5** — Native `TestCase`: stable `TC-*` IDs, `testLevel` (L1–L5), Gherkin scenario blocks, `testFunctions` cross-reference.
- **§11.10** — ID-based cross-reference resolution for `verifies:` and `derivedFrom:`.
- **§11.11** — Computed reverse indices (`verifiedBy`, `derivedChildren`) and coverage checks.
- **§11.12** — Complete validation rule reference (E001–E106, W001–W007, E200–E209, E300–E315, W300–W304).

---

## Part 1C — Traceability Rules (§12)

Seven enforced traceability rules govern how model elements relate to each other:

1. **OSLC link direction (§12.1)** — Links always point upstream: the derived/verifying/satisfying artifact holds the reference, not the artifact it traces to.
2. **Requirement breakdown needs an ADR (§12.2)** — Every `Requirement` with `derivedFrom:` must set `breakdownAdr:` to an `accepted` ADR (`type: ADR`, `ADR-*` id). Error `E310` if absent; warning `W303` if the ADR is still `proposed`.
3. **Leaf-level assignment (§12.3)** — Requirements must be broken down until each leaf can be assigned to a single architecture element. Leaf requirements with `status: approved` or `implemented` and no satisfying element trigger warning `W300`.
4. **No parent assignment (§12.4)** — A requirement with `derivedChildren` must not appear in any `satisfies:` list. Error `E312`.
5. **Domain classification (§12.5)** — Requirements carry `reqDomain: system | hardware | software`; architecture elements carry `domain: system | hardware | software`. A leaf requirement must be satisfied only by an element whose domain matches its `reqDomain` (unless either is `system`). Error `E313` on mismatch.
6. **HW/SW independence (§12.6)** — Elements with `domain: software` and `domain: hardware` must not share `supertype:` or `typedBy:` links (error `E315`). Cross-domain integration uses explicit `Allocation` elements. A `Part`/`PartDef` with `isDeploymentPackage: true` must have at least one `Allocation` to a `hardware` element (error `E314`).
7. **Implementation trace (§12.8)** — A `Part`/`PartDef` may declare `implementedBy:` (string or list) linking it to the source artifact(s) that realise it, closing the V-model leg `Requirement ─satisfies→ Architecture ─implementedBy→ Code ─verifies→ Test`. Paths resolve like a TestCase's `sourceFile`. A missing local path on a non-`draft` element triggers warning `W023` (opt-in, draft-suppressed, gate with `--deny W023`); remote URIs are accepted as external.

---

## Part 2 — Web Service Architecture

### Stack

- **Backend**: Rust (`syscribe-server`) — Axum parses the model directory tree, builds an in-memory graph, exposes REST + WebSocket endpoints, and serves HTML via Askama templates.
- **Frontend**: Askama templates (server-side HTML rendering) + HTMX for dynamic interactions. No JavaScript framework.
- **Diagrams**: SVG built server-side by `syscribe-model::renderer`; Mermaid rendered client-side from CDN.
- **Live reload**: `notify` crate watches the model directory; changes are pushed to connected clients over WebSocket.

### Crate layout

| Crate | Path | Role |
|---|---|---|
| `syscribe-model` | `crates/syscribe-model/` | Parser, validator, graph builder, renderer, resolver |
| `syscribe-server` | `crates/syscribe-server/` | Axum routes, Askama templates, WebSocket watch mode |

### API Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/elements` | List all elements (optional `?type=` filter) |
| `GET` | `/api/elements/<qname>` | Single element JSON |
| `PUT` | `/api/elements/<qname>` | Write element YAML frontmatter |
| `GET` | `/api/children?qname=<qname>` | Containment tree |
| `GET` | `/api/connections?qname=<qname>` | Connection graph |
| `PATCH` | `/api/diagrams/layout/<qname>` | Persist drag-adjusted layout coordinates |
| `GET` | `/api/validation` | Validation findings JSON |
| `WS` | `/ws` | Live model-change events |

### UI Routes

| Path | Description |
|---|---|
| `GET /` | Root — model tree browser |
| `GET /ui/tree?parent=<qname>` | HTMX — tree items for a namespace |
| `GET /ui/detail/<qname>` | HTMX — element detail panel |
| `GET /ui/diagram/<qname>` | HTMX — diagram panel (SVG or Mermaid) |

---

## Development Notes

- The `temp/` directory contains reference PDFs only — not tracked by git.
- `site/` is MkDocs build output — not tracked by git.
- Qualified name resolution handles circular references gracefully (reports, does not panic).
- The Syscribe format is the source of truth; the web service is read-only over the model files.
- The LLM generation prompt lives at `prompts/create-model.md` and is embedded in the validator binary via `include_str!` — edit the `.md` file, not the Rust source.
- **Diagram layout files** (`*.layout.json`) are ephemeral workspace inputs to `diagram compose` — they are not part of the Syscribe schema and must never be committed. Name them `<anything>.layout.json` so the `.gitignore` pattern excludes them automatically.

## LLM Workflow

The project is designed for LLM-assisted model authoring. The prompt at `prompts/create-model.md` gives an LLM everything it needs to produce valid Syscribe models. See `docs/model-guide/llm-workflow.md` for the full workflow.
