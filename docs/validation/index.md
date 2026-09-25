# Validation

`VALIDATION · OVERVIEW`

The validation engine is a Rust library (`syscribe-model`) that runs every parse-time and model-time check in a single pass over all loaded elements. It is invoked by both the CLI report tool and the web server (which exposes results at `/api/validation`).

## Running validation

```bash
# Markdown report to stdout
cargo run --package syscribe -- -m model/

# JSON via the web API (server must be running)
curl http://localhost:3000/api/validation
```

## Finding structure

Each finding has:

| Field | Description |
|---|---|
| Code | `E___` (error), `W___` (warning) or `I___` (informational) |
| File | Path to the `.md` file |
| Message | Human-readable description |
| Severity | `Error`, `Warning` or `Info` |

Errors block a clean build (exit `1`). Warnings are advisory unless promoted by a CI gate (`--deny`, `--max-warnings`, `--warnings-as-errors`, `--profile`; exit `2`). Informational findings never change the exit status unless selected with `--deny`.

## Rule groups

Ranges are inclusive and name the codes actually in use; gaps inside a range are unassigned or retired numbers. Many groups are **opt-in** — dormant until the model uses the feature (a `FeatureDef`, `[repos]`, `[linkTypes]`, a `Baseline`, a `foreignFormat:` package, …).

| Range | Group | Description |
|---|---|---|
| E000–E015, E019–E025 | Parse-time | Missing frontmatter delimiter (E001), invalid YAML, required fields, ID patterns, status/testLevel/integrity enums, Gherkin structure, ASPICE fields, ID-digit cap (E023), removed `title:` (E025; E003/E024 retired); E000 is an internal fallback that should never appear |
| E016–E018, E107 | Cycle detection | Cycles in `supertype:`, `derivedFrom:`, `subsets:` and `typedBy:` graphs |
| W001–W010, I010 | Parse-time and source drift | Normative `shall`, leaf test coverage (W002/W003), orphan requirements, SIL/ASIL pairing, unused type defs, missing `type:`, `sourceFile`/test-function drift, ingested test results; I010 flags a planned test not yet present |
| E101–E108, E110–E114 | Cross-reference | Duplicate ids (E101) and qualified names (E108), unresolved or ill-typed `verifies`/`derivedFrom`, scenario names, unresolved `supertype`/`typedBy`/`subsets`/`redefines`/`satisfies` |
| E050, W050 | Build-system integration (§9.9) | Conflicting `buildExports` variables; selected feature exporting nothing |
| E200–E237 | Product lines (§9) | Configuration and FeatureDef fields, parameter binding (E202–E207, E222, E229, E230), `appliesWhen` (E209, E228), feature-model constraints (E212, E213, E219–E221), `feature-check --deep` (E223–E225, E227), `--config` escapes (E226), single-file feature models (E231–E233), Configuration inheritance (E215, E234–E237) |
| W011–W027, W048 | Product-line warnings (§9) | Dead/always-selected features, untested active requirements, unbound parameters, false-optional, escaping/violable references, orphan features, runtime bindings, empty gating packages, misplaced feature-model fields |
| W023, W028, W029 | Implementation and external refs | Missing `implementedBy:` path (§12.8), duplicate `extRef`, unmeasured `wcet:` claim |
| W030–W040 | Safety↔security | Co-engineering, cyber-risk treatment and CAL, HW metrics, freedom from interference, attack-tree feasibility, responsibility and independent assessment, orphan GSN nodes |
| W041–W047 | Schema hygiene | Nested `custom_fields`, non-basic names, standard-library typos and unit/quantity mismatches, stereotype tagged values, `[ids.prefixes]` entries, unrecognised frontmatter keys |
| E300–E304 | ADR | ID pattern, required fields, status enum, reqDomain/domain enums |
| E310–E318 | §12 Traceability and metadata | breakdownAdr, parent in satisfies, domain mismatch, deployment allocation, HW/SW independence, `refines:` (E316), stereotype resolution/applicability (E317, E318) |
| W300–W311 | Traceability and planning warnings | Leaf satisfaction, domain refinement, proposed breakdown ADR, deployment domain, parent integration tests, unsatisfied safety mechanism, use cases without `refines:` (W301 retired); PlanningItem staleness, roster, completion bar, overlapping work (W308–W311) |
| E400–E404, W400–W415 | Diagrams (§8.16) | Mermaid/PlantUML bodies, companion SVG/PUML, `pumlMode`, diagramKind, subject/shape/edge resolution, SVG ids and hrefs, Mermaid annotations, operation types, PlantUML style file |
| W070–W080, W929 | Behavior (§22.1, §22.4) | State-machine completeness (dead/trap/initial/parallel/transitions), sequence-diagram send/receive completeness |
| E500–E506, W500–W503, W930 | Allocation, derive, structure | Allocation resolution, `derive:` cycles/parse/unknown elements, View viewpoint/expose, exhibitsStates, redundant or misplaced allocations |
| E510–E515, W510–W512 | Multi-repository (§14) | Circular import, missing path, cross-repo ref resolution, import alias/qname, duplicate stable ID, ref pin / drift / submodule gitlink |
| E516–E519, E523, W513 | Hierarchical product lines (§14.7) | `subConfigurations:` dangling / wrong type / not internally valid; cross-tier `parameterBindings:`; open required parameters across the subtree |
| E520–E522, W520 | Release baselines (§8.19) | Drift of a released (E520) or approved (W520) baseline, seal/manifest tamper, unresolved `supersedes` |
| E530–E532, W530–W534 | Reserved | Parked sandboxed-WASM plugin design (`ADR-SYS-PLUGIN-001`); never emitted |
| W540–W542 | SysML v2 submodel ingestion | Stray `.md` in a submodel, unreadable/unparseable `.sysml`, truncated feature chains |
| E550–E551, W550–W553 | Stdio plugins | Missing command or `[plugins]` entry, execution failure, bad envelope, dropped elements |
| E560–E561, W560–W563 | Annotated source | Bad `annotationFormat:` config, invalid marker YAML, skipped blocks, conflicting ingestion modes, auto-filled `implementedBy:` |
| W090 | Suspect links | A baselined trace-link target changed since review |
| W099–W103 | Documentation linting (`lint-docs`) | Dangling ids, qnames, SVG refs and image paths in external docs; enumerated package members |
| E600–E606, W600–W601, W610–W616 | TestPlan and documentation | TestPlan fields, members, selection, demonstrates, configurations; empty PartDef/ActionDef docs |
| E630–E636, W630–W631 | User-defined link types (§12.10) | Undeclared/malformed `links:`, unresolved targets, source/target type, cardinality, acyclic types, malformed `[linkTypes]` |
| E700–E705, W700, W704 | Review records (§19) | ReviewRecord fields, ID/status/type enums, reviewed-element resolution, dispositions, unreviewed requirements |
| E706–E723 | PlanningItem (§23) | ID/fields/status/itemType, parent resolution and cycle, top-level `achieves`, evidence, leaf-done-needs-evidence, `blockedBy`, `assignedTo`; E718 is a non-scalar `Argument.evidence` entry |
| W701–W703 | Safety / ASPICE | verificationMethod on high-ASIL requirements, L5 test for ASIL D, mixed standards |
| E800–E837, W800–W810 | Tier 2 safety and security | HARA/TARA element fields, ID patterns, enums, cross-references; coverage and traceability gaps, security test methods, assets |
| E841–E865, E924, W860 | Integrity and assurance | Integrity-level propagation, safety↔security links, risk treatment, diagnostic coverage, confirmation measures (E924: status enum), GSN arguments and assumptions, assets, decomposition pairs |
| E866–E877, W060–W064 | Budgets and trade studies (§22.2, §15) | Budget expressions and bounds; TradeStudy fields, criteria, scores, decision |
| E900–E923, E927, W900–W905, W926–W928 | Tier 4 FTA / FMEA / attack trees | FaultTree, FMEA, TARA sheet and attack-tree fields, IDs, enums, inputs, RPN, cross-links |
| E940–E941 | Tier 4 TARA container | TARASheet fields and ID pattern |
| E950–E956, E925, E926, W950–W953 | IEC 62443 (§13) | Zone/Conduit fields, IDs, status (E926), SL range (E925), resolution, Security-Level gaps |

See [Rule Reference](rules.md) for every code.

## Tool qualification

The validator is itself qualified under ISO 26262 Part 8 §11 (TCL2). The `qual/` directory contains the qualification model — over 240 requirements (`REQ-TRS-*`) and a matching test case (`TC-TRS-*`) per requirement, in Syscribe format, covering every emitted validation code — along with a shell test runner that invokes the binary against crafted fixture models.

Run `syscribe -m qual/` to validate the qualification model, or `bash qual/tests/run_qual.sh` to execute the full TCL2 test suite. See [Tool Qualification](../tool-qualification/index.md) for the complete story.

## Demo model

The UAV demo model in `model/` validates with **0 errors**. Its remaining warnings are deliberate, advisory findings (for example unused type definitions, use cases without `refines:`, and test plans naming draft test cases) that show what the checks report; run `syscribe -m model/ validate` for the current list.

## Computed reverse indices

The validator builds two reverse indices that the report and web API expose:

- `verified_by[req_id]` — list of active TC IDs covering this requirement
- `derived_children[req_id]` — list of child requirement IDs derived from this requirement
