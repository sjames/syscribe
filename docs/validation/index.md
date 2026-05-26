# Validation

`VALIDATION · OVERVIEW`

The validation engine is a Rust library (`syscribe-model`) that runs every parse-time and model-time check in a single pass over all loaded elements. It is invoked by both the CLI report tool and the web server (which exposes results at `/api/validation`).

## Running validation

```bash
# Markdown report to stdout
cargo run --example validate_model -- model/

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
| E001–E015 | Parse-time | Required fields, ID patterns, status enums, Gherkin structure |
| W001–W006 | Parse-time warnings | Normative text, SIL/ASIL pairing, sourceFile presence |
| E101–E106 | Cross-reference | Duplicate IDs, unresolved `verifies`/`derivedFrom`, scenario names |
| E200–E209 | PLE | Configuration required fields, featureModel resolution, appliesWhen |
| E300–E304 | ADR | ID pattern, required fields, status enum, reqDomain/domain enums |
| W300–W304 | Traceability | Leaf coverage, multiple satisfiers, domain refinement, deployment package |
| E310–E315 | §12 Traceability | breakdownAdr, parent in satisfies, domain mismatch, HW/SW independence |
| E400–E402 | Diagram | Mermaid/PlantUML body blocks, svgFile on disk |
| W400–W403 | Diagram | diagramKind absent, subject resolution, shape/edge refs |
| E500–E503 | Allocation | allocatedFrom/allocatedTo resolution |
| W500–W502 | Structural | viewpoint, exhibitsStates, expose resolution |
| W600–W601 | Documentation | Empty PartDef/Part or ActionDef/Action doc body |
| W404 | Operations | Operation parameter or return type unresolved |

See [Rule Reference](rules.md) for every code.

## Zero-finding model

The UAV demo model in `model/` runs with **0 errors** and **2 warnings** (both W404 for `ScalarValues::*` standard library types not registered in the model tree — expected and correct).

## Computed reverse indices

The validator builds two reverse indices that the report and web API expose:

- `verified_by[req_id]` — list of active TC IDs covering this requirement
- `derived_children[req_id]` — list of child requirement IDs derived from this requirement
