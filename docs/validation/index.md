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
| E001–E025 | Parse-time | Required fields, ID patterns, status enums, Gherkin structure, ASPICE fields, ID-digit cap (E023), label-field rule `name` vs `title` (E024/E025) |
| W001–W008 | Parse-time warnings | Normative text, SIL/ASIL pairing, sourceFile presence, unused type defs |
| E016–E018 | Cycle detection | Cycles in supertype, derivedFrom, or subsets graphs |
| E101–E106 | Cross-reference | Duplicate IDs, unresolved `verifies`/`derivedFrom`, scenario names |
| E200–E209 | PLE | Configuration required fields, featureModel resolution, appliesWhen |
| E300–E304 | ADR | ID pattern, required fields, status enum, reqDomain/domain enums |
| W300–W305 | Traceability | Leaf coverage, multiple satisfiers, domain refinement, integration test on parent reqs |
| E310–E315 | §12 Traceability | breakdownAdr, parent in satisfies, domain mismatch, HW/SW independence |
| E400–E402 | Diagram | Mermaid/PlantUML body blocks, companion SVG on disk |
| W400–W412 | Diagram | diagramKind, subject/shape/edge resolution, Mermaid annotations, SVG hrefs, operation typedBy |
| E500–E503 | Allocation | allocatedFrom/allocatedTo resolution on Allocation elements and any element |
| W500–W502 | Structural | viewpoint, exhibitsStates, expose resolution on View elements |
| W600–W601 | Documentation | Empty PartDef/Part or ActionDef/Action doc body |
| W701–W703, W807 | Safety / ASPICE | verificationMethod on high-ASIL reqs, L5 test for ASIL D, mixed standards, security reqs |
| E800–E837 | Tier 2 Safety | HazardousEvent, SafetyGoal required fields, ID patterns, HARA parameter enums (ISO 26262 / IEC 61508 / ISO 13849-1) |
| E807–E814 | Tier 2 Safety | DamageScenario, ThreatScenario required fields, ID patterns, enum validation |
| E815–E832 | Tier 2 Security | CybersecurityGoal, SecurityControl, VulnerabilityReport required fields, ID patterns, cross-references |
| W800–W807 | Tier 2 coverage | Unreferenced hazards, unimplemented goals, open vulnerabilities, traceability gaps |
| E841–E843, W808 | Integrity propagation | asilLevel/silLevel must propagate through derivedFromSafetyGoal, derivedFrom, and satisfies chains |
| E900–E909 | Tier 4 — FaultTree | FaultTree, FaultTreeGate, FaultTreeEvent required fields, ID patterns, gate type and event kind enums, input resolution |
| W900–W901 | Tier 4 — FaultTree | Empty fault tree, gate with no inputs |
| E911–E914 | Tier 4 — FMEA | FMEASheet and FMEAEntry required fields, ID patterns, severity/occurrence/detection range 1–10 |
| W902–W904 | Tier 4 — FMEA | Empty FMEA sheet, high-RPN entry without recommended action, unresolved ref |
| E940–E941, W905 | Tier 4 — TARA | TARASheet required fields, ID pattern, empty sheet |

See [Rule Reference](rules.md) for every code.

## Tool qualification

The validator is itself qualified under ISO 26262 Part 8 §11 (TCL2). The `qual/` directory contains 60 requirements (`REQ-TRS-*`) and 60 test cases (`TC-TRS-*`) in Syscribe format — covering every emitted validation code — along with a shell test runner that invokes the binary against crafted fixture models.

Run `syscribe -m qual/` to validate the qualification model, or `bash qual/tests/run_qual.sh` to execute the full TCL2 test suite. See [Tool Qualification](../tool-qualification/index.md) for the complete story.

## Zero-finding model

The UAV demo model in `model/` runs with **0 errors** and **2 warnings** (both W404 for `ScalarValues::*` standard library types not registered in the model tree — expected and correct).

## Computed reverse indices

The validator builds two reverse indices that the report and web API expose:

- `verified_by[req_id]` — list of active TC IDs covering this requirement
- `derived_children[req_id]` — list of child requirement IDs derived from this requirement
