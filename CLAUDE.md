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
cargo run --package syscribe -- model/
# or after build: ./target/debug/syscribe model/

# Print the LLM model generation prompt
cargo run --package syscribe -- --agent-instructions

# Start the web server
cargo run --package syscribe-server -- model/
```

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
| — (Architecture Decision Record) | `.md` file, `type: ADR` — dedicated handler, ADR-* id |
| `allocation` | `.md` file, `type: Allocation` |
| `view def` / `view` | `.md` file, `type: ViewDef` / `View` |

### Frontmatter Schema (common fields)

```yaml
---
id: <qualified-name>          # auto-derived from path if omitted
type: <SysML element type>    # required
name: <display name>          # defaults to filename stem
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

### ID Scheme

SysML elements (`PartDef`, `Port`, etc.) use `id` auto-derived from the file path if omitted. `Requirement` and `TestCase` carry a **stable opaque identifier** that must be explicitly set and never changes.

**Requirement ID pattern** — `^REQ(-[A-Z0-9]{2,12})+-[0-9]{3}$`
- Prefix `REQ`, one or more uppercase-alphanumeric segments (2–12 chars), three-digit suffix.
- Examples: `REQ-SCHED-001`, `REQ-SCHED-BITMAP-001`, `REQ-PORT-CTX-001`

**TestCase ID pattern** — `^TC(-[A-Z0-9]{2,12})+-[0-9]{3}$`
- Same segment rules, prefix `TC`. Test level is not encoded in the ID (lives in `testLevel:`).
- Examples: `TC-SCHED-BITMAP-001`, `TC-SYNC-SEM-002`

**ADR ID pattern** — `^ADR(-[A-Z0-9]{2,12})+-[0-9]{3}$`
- Same segment rules, prefix `ADR`. Statuses: `proposed | accepted | deprecated | superseded`.
- Examples: `ADR-SYS-001`, `ADR-SW-SCHED-001`

Both the `id` field and the qualified name (path-derived) are valid cross-reference targets in `verifies:` and `derivedFrom:`.

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

Six enforced traceability rules govern how model elements relate to each other:

1. **OSLC link direction (§12.1)** — Links always point upstream: the derived/verifying/satisfying artifact holds the reference, not the artifact it traces to.
2. **Requirement breakdown needs an ADR (§12.2)** — Every `Requirement` with `derivedFrom:` must set `breakdownAdr:` to an `accepted` ADR (`type: ADR`, `ADR-*` id). Error `E310` if absent; warning `W303` if the ADR is still `proposed`.
3. **Leaf-level assignment (§12.3)** — Requirements must be broken down until each leaf can be assigned to a single architecture element. Leaf requirements with `status: approved` or `implemented` and no satisfying element trigger warning `W300`.
4. **No parent assignment (§12.4)** — A requirement with `derivedChildren` must not appear in any `satisfies:` list. Error `E312`.
5. **Domain classification (§12.5)** — Requirements carry `reqDomain: system | hardware | software`; architecture elements carry `domain: system | hardware | software`. A leaf requirement must be satisfied only by an element whose domain matches its `reqDomain` (unless either is `system`). Error `E313` on mismatch.
6. **HW/SW independence (§12.6)** — Elements with `domain: software` and `domain: hardware` must not share `supertype:` or `typedBy:` links (error `E315`). Cross-domain integration uses explicit `Allocation` elements. A `Part`/`PartDef` with `isDeploymentPackage: true` must have at least one `Allocation` to a `hardware` element (error `E314`).

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
