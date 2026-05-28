# Validation

`VALIDATION · OVERVIEW`

The validation engine is a Rust library (`syscribe-model`) that runs every parse-time and model-time check in a single pass over all loaded elements. It is invoked by both the CLI report tool and the web server (which exposes results at `/api/validation`).

## Running validation

```bash
# Markdown report to stdout
cargo run --package syscribe -- model/

# JSON via the web API (server must be running)
curl http://localhost:3000/api/validation
```

## Finding structure

Each finding has:

| Field | Description |
|---|---|
| Code | `E___` (error) or `W___` (warning) |
| File | Path to the `.md` file |
| Message | Human-readable description |
| Severity | `Error` or `Warning` |

Errors block a clean build. Warnings are advisory.

## Rule groups

| Range | Group | Description |
|---|---|---|
| E001–E022 | Parse-time | Required fields, ID patterns, status enums, Gherkin structure, ASPICE fields |
| W001–W008 | Parse-time warnings | Normative text, SIL/ASIL pairing, sourceFile presence, unused type defs |
| E101–E106 | Cross-reference | Duplicate IDs, unresolved `verifies`/`derivedFrom`, scenario names |
| E016–E018 | Cycle detection | Cycles in supertype, derivedFrom, or subsets graphs |
| E200–E209 | PLE | Configuration required fields, featureModel resolution, appliesWhen |
| E300–E304 | ADR | ID pattern, required fields, status enum, reqDomain/domain enums |
| W300–W305 | Traceability | Leaf coverage, multiple satisfiers, domain refinement, integration test on parent reqs |
| E310–E315 | §12 Traceability | breakdownAdr, parent in satisfies, domain mismatch, HW/SW independence |
| E400–E402 | Diagram | Mermaid/PlantUML body blocks, svgFile on disk |
| W400–W412 | Diagram | diagramKind, subject/shape/edge resolution, Mermaid annotations, SVG hrefs |
| E500–E503 | Allocation | allocatedFrom/allocatedTo resolution |
| W500–W502 | Structural | viewpoint, exhibitsStates, expose resolution |
| W600–W601 | Documentation | Empty PartDef/Part or ActionDef/Action doc body |
| W701–W703 | Safety / ASPICE | verificationMethod on high-ASIL reqs, L5 test for ASIL D, mixed standards |
| E800–E830 | Tier 2 Safety/Security | HazardousEvent, SafetyGoal, DamageScenario, ThreatScenario, CybersecurityGoal, SecurityControl, VulnerabilityReport required fields and cross-references |
| W800–W803 | Tier 2 coverage | Unreferenced HazardousEvents, unimplemented CybersecurityGoals, open VulnerabilityReports |

See [Rule Reference](rules.md) for every code.

## Zero-finding model

The UAV demo model in `model/` runs with **0 errors** and **2 warnings** (both W404 for `ScalarValues::*` standard library types not registered in the model tree — expected and correct).

## Computed reverse indices

The validator builds two reverse indices that the report and web API expose:

- `verified_by[req_id]` — list of active TC IDs covering this requirement
- `derived_children[req_id]` — list of child requirement IDs derived from this requirement
