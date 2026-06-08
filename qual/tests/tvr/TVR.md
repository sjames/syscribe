# Tool Validation Report

**Tool:** syscribe CLI validator  
**Version:** unknown  
**Standard:** ISO 26262:2018 Part 8 §11 (TCL2), IEC 61508:2010 Part 3 Annex D  
**Date:** 2026-06-08  
**TRS:** `qual/Requirements/`  **Test cases:** `qual/TestCases/`

---

## 1. Summary

| Metric | Value |
|---|---|
| Total test cases | 124 |
| Passed | 124 |
| Failed | 0 |
| Overall verdict | **PASS** |

---

## 2. Results

### TC-TRS-CLI-001 — Verify that the tool accepts the model directory via -m and --model arguments.

**Verifies:** REQ-TRS-CLI-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| --model long form | ✓ PASS |

---

### TC-TRS-CLI-002 — Verify that the tool reports an error to stderr and exits non-zero for invalid model paths.

**Verifies:** REQ-TRS-CLI-002  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| non-existent path exits non-zero | ✓ PASS |

---

### TC-TRS-CLI-003 — Verify that --agent-instructions prints the LLM prompt and exits 0 without requiring -m.

**Verifies:** REQ-TRS-CLI-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| --agent-instructions prints prompt and exits 0 | ✓ PASS |

---

### TC-TRS-CLI-004 — Verify model-root auto-discovery via walk-up to .syscribe.toml, flag override, and fallback.

**Verifies:** REQ-TRS-CLI-004  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| no marker and no model falls back to default and reports the miss | ✓ PASS |

---

### TC-TRS-CONF-001 — Verify that E200, E201, and E209 are emitted for Configuration and appliesWhen violations.

**Verifies:** REQ-TRS-CONF-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E200 | ✓ PASS |
| trigger E201 | ✓ PASS |
| trigger E209 | ✓ PASS |

---

### TC-TRS-CONF-002 — Verify Configuration selection parsing: template uses features:, W016 on empty selections, show displays selections.

**Verifies:** REQ-TRS-CONF-002  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| template emits features: map, not selections: | ✓ PASS |
| legacy selections: under a feature model warns | ✓ PASS |
| features: map configuration does not warn | ✓ PASS |
| show displays parsed feature selections | ✓ PASS |

---

### TC-TRS-DIAG-001 — Verify that E400–E402 and W400–W412 are emitted for Diagram element validation conditions.

**Verifies:** REQ-TRS-DIAG-001  
**Result:** ✓ PASS (17 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E400 | ✓ PASS |
| trigger E401 | ✓ PASS |
| trigger E402 | ✓ PASS |
| trigger W400 | ✓ PASS |
| trigger W401 | ✓ PASS |
| trigger W402 | ✓ PASS |
| trigger W403 | ✓ PASS |
| trigger W404 | ✓ PASS |
| trigger W406 | ✓ PASS |
| trigger W407 | ✓ PASS |
| trigger W408 | ✓ PASS |
| trigger W409 | ✓ PASS |
| trigger W410 | ✓ PASS |
| trigger W411 | ✓ PASS |
| trigger W412 | ✓ PASS |
| trigger W405 (companion mode, no img tag) | ✓ PASS |
| trigger W405 (inline mode, no svg block) | ✓ PASS |

---

### TC-TRS-DISC-001 — Verify the `features` command: feature-model overview with groupKind, parameters, and per-feature selection rollup; --json; dormancy.

**Verifies:** REQ-TRS-DISC-001  
**Result:** ✓ PASS (15 passed, 0 failed)

| Scenario | Result |
|---|---|
| dormant on a model with no feature model | ✓ PASS |

---

### TC-TRS-DISC-002 — Verify the `feature <qname>` card: gated elements, selecting configurations, parameters; errors on unknown feature.

**Verifies:** REQ-TRS-DISC-002  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| unknown feature errors | ✓ PASS |

---

### TC-TRS-DISC-003 — Verify `matrix --features`: Feature × Configuration selection grid; default matrix regression.

**Verifies:** REQ-TRS-DISC-003  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| default matrix still shows Requirement × Configuration view | ✓ PASS |

---

### TC-TRS-DISC-004 — Verify `list <type> --feature <F>`: filters to elements gated on F; errors on unknown feature.

**Verifies:** REQ-TRS-DISC-004  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| unknown feature errors | ✓ PASS |

---

### TC-TRS-DISC-005 — Verify `why-active <el> --config <C>`: active/inactive/always-active verdict; errors on missing/unresolved --config.

**Verifies:** REQ-TRS-DISC-005  
**Result:** ✓ PASS (9 passed, 0 failed)

| Scenario | Result |
|---|---|
| --config is required and must resolve | ✓ PASS |

---

### TC-TRS-DISC-006 — Verify orphan-feature warning W024 in feature-check: exactly one, names the orphan, not in base validate, gateable.

**Verifies:** REQ-TRS-DISC-006  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| base validate never emits W024 | ✓ PASS |
| W024 is gateable with --deny | ✓ PASS |

---

### TC-TRS-DISC-007 — Verify list --status/--sil/--json filters and matrix --status/--gaps-only/coverage footer.

**Verifies:** REQ-TRS-DISC-007  
**Result:** ✓ PASS (14 passed, 0 failed)

| Scenario | Result |
|---|---|
| matrix --json carries a coverage object | ✓ PASS |

---

### TC-TRS-ELEM-001 — Verify that all element types defined in §2 are recognised and processed without E005.

**Verifies:** REQ-TRS-ELEM-001  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| all defined element types are recognised without E005 | ✓ PASS |

---

### TC-TRS-ELEM-002 — Verify that an unrecognised type: value produces exactly one E005 finding.

**Verifies:** REQ-TRS-ELEM-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| completely unknown type value produces E005 | ✓ PASS |
| wrong-case type value produces E005 | ✓ PASS |

---

### TC-TRS-ELEM-003 — Verify that implicit base library supertypes are applied when no supertype: is given.

**Verifies:** REQ-TRS-ELEM-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| PartDef with no supertype: loads without E004 | ✓ PASS |

---

### TC-TRS-EXTREF-001 — Verify the extRef common field parses (string or list) and duplicate detection W028.

**Verifies:** REQ-TRS-EXTREF-001  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| single-string and list-valued extRef parse without error | ✓ PASS |
| the same extRef on two elements produces W028 | ✓ PASS |
| W028 is gateable with --deny | ✓ PASS |

---

### TC-TRS-EXTREF-002 — Verify extref lookup command, --json, show surfacing, and spec fields listing.

**Verifies:** REQ-TRS-EXTREF-002  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| spec fields lists extRef | ✓ PASS |

---

### TC-TRS-FM-001 — Verify the feature-check command: discoverable, exit codes, dormancy, --json.

**Verifies:** REQ-TRS-FM-001  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| --json emits structured findings | ✓ PASS |

---

### TC-TRS-FM-002 — Verify feature-check structural rules E212, E219, E220, W011, W012.

**Verifies:** REQ-TRS-FM-002  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| clean feature model emits none of them | ✓ PASS |

---

### TC-TRS-FM-003 — Verify feature-check parameter rules E207, E202, E213, W014.

**Verifies:** REQ-TRS-FM-003  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| clean feature model emits none of them | ✓ PASS |

---

### TC-TRS-FM-004 — Verify `mandatory:` membership field: a mandatory feature is core in deep analysis; legacy groupKind: mandatory still forced.

**Verifies:** REQ-TRS-FM-004  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| legacy groupKind: mandatory child still treated as forced | ✓ PASS |

---

### TC-TRS-FMA-001 — Verify the Boolean encoding via solver-observable semantics.

**Verifies:** REQ-TRS-FMA-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| contradictory model is void | ✓ PASS |

---

### TC-TRS-FMA-002 — Verify feature-check --deep command surface, gating, exit codes, and --json.

**Verifies:** REQ-TRS-FMA-002  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| dormant with no feature model | ✓ PASS |

---

### TC-TRS-FMA-003 — Verify anomaly analyses: void, dead, core, false-optional.

**Verifies:** REQ-TRS-FMA-003  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| void dominates (no dead spam) | ✓ PASS |

---

### TC-TRS-FMA-004 — Verify full-semantics configuration validity (E225) without duplicating E219/E220.

**Verifies:** REQ-TRS-FMA-004  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| requires violation stays E219, not E225 | ✓ PASS |

---

### TC-TRS-FMA-005 — Verify sound explanations for void models.

**Verifies:** REQ-TRS-FMA-005  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| soundness: removing the excludes un-voids the model | ✓ PASS |

---

### TC-TRS-FMA-006 — Verify determinism, ~500-feature scale, the size guard, and the Boolean-only scope statement.

**Verifies:** REQ-TRS-FMA-006  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| scope statement: Boolean layer only | ✓ PASS |

---

### TC-TRS-FMA-007 — Verify minimal (MUS) unsat-core explanations exclude unrelated constraints.

**Verifies:** REQ-TRS-FMA-007  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| explanation is a minimal conflict set (excludes unrelated features) | ✓ PASS |

---

### TC-TRS-FMA-008 — Verify the configure command: satisfiability, forced and free features, contradictions.

**Verifies:** REQ-TRS-FMA-008  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| dormant with no feature model | ✓ PASS |

---

### TC-TRS-FMA-009 — Verify variant-space count and enumeration.

**Verifies:** REQ-TRS-FMA-009  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| enumeration lists each valid configuration | ✓ PASS |

---

### TC-TRS-FMA-010 — Verify diagnoses (minimal correction sets) for void models.

**Verifies:** REQ-TRS-FMA-010  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| a correction set names the requires or excludes constraint | ✓ PASS |

---

### TC-TRS-FMA-011 — Verify opt-in proof-evidence emission (DIMACS CNF) for UNSAT findings.

**Verifies:** REQ-TRS-FMA-011  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| no proof files without --prove | ✓ PASS |

---

### TC-TRS-FMEA-001 — Verify that FMEASheet and FMEAEntry validation rules E911–E914, W902–W904 are enforced.

**Verifies:** REQ-TRS-FMEA-001  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E911 | ✓ PASS |
| trigger E912 | ✓ PASS |
| trigger E913 | ✓ PASS |
| trigger E914 | ✓ PASS |
| trigger W902 | ✓ PASS |
| trigger W903 | ✓ PASS |
| trigger W904 | ✓ PASS |

---

### TC-TRS-FTA-001 — Verify that FaultTree, FaultTreeGate, and FaultTreeEvent validation rules E900–E909, W900–W901 are enforced.

**Verifies:** REQ-TRS-FTA-001  
**Result:** ✓ PASS (12 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E900 | ✓ PASS |
| trigger E901 | ✓ PASS |
| trigger E902 | ✓ PASS |
| trigger E903 | ✓ PASS |
| trigger E904 | ✓ PASS |
| trigger E905 | ✓ PASS |
| trigger E906 | ✓ PASS |
| trigger E907 | ✓ PASS |
| trigger E908 | ✓ PASS |
| trigger E909 | ✓ PASS |
| trigger W900 | ✓ PASS |
| trigger W901 | ✓ PASS |

---

### TC-TRS-ID-001 — Verify that Requirement elements are validated against the REQ-* id pattern.

**Verifies:** REQ-TRS-ID-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid REQ-* id patterns are accepted | ✓ PASS |
| invalid REQ-* id pattern produces E006 | ✓ PASS |

---

### TC-TRS-ID-002 — Verify that TestCase elements are validated against the TC-* id pattern.

**Verifies:** REQ-TRS-ID-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid TC-* id pattern is accepted | ✓ PASS |
| invalid TC-* id pattern produces E006 | ✓ PASS |

---

### TC-TRS-ID-003 — Verify that ADR elements are validated against the ADR-* id pattern.

**Verifies:** REQ-TRS-ID-003  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid ADR-* id is accepted | ✓ PASS |
| ADR id not matching pattern produces E300 | ✓ PASS |
| ADR missing id produces E301 | ✓ PASS |

---

### TC-TRS-ID-004 — Verify that duplicate id: values across elements produce E101.

**Verifies:** REQ-TRS-ID-004  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| two elements with the same id produce E101 | ✓ PASS |
| unique ids produce no E101 | ✓ PASS |

---

### TC-TRS-IMPL-001 — Verify implementedBy path-exists rule W023: missing path, opt-in, draft suppression, gating.

**Verifies:** REQ-TRS-IMPL-001  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| W023 is gateable with --deny | ✓ PASS |

---

### TC-TRS-IMPL-002 — Verify implementedBy discoverability: links, refs, spec fields.

**Verifies:** REQ-TRS-IMPL-002  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| spec fields lists implementedBy | ✓ PASS |

---

### TC-TRS-MOVE-001 — Verify move relocates an element and a package (with subtree) and rejects invalid destinations.

**Verifies:** REQ-TRS-MOVE-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| reject move into own subtree | ✓ PASS |

---

### TC-TRS-MOVE-002 — Verify move updates all qualified-name references, including nested ones, without false matches.

**Verifies:** REQ-TRS-MOVE-002  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| descendant endpoint follows package move | ✓ PASS |

---

### TC-TRS-MOVE-003 — Verify move is atomic — a failing precondition leaves the model unchanged.

**Verifies:** REQ-TRS-MOVE-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| --dry-run reports without writing | ✓ PASS |

---

### TC-TRS-MOVE-004 — Verify move preserves stable IDs and references made through them.

**Verifies:** REQ-TRS-MOVE-004  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| id-based reference is not rewritten and still resolves | ✓ PASS |

---

### TC-TRS-OUT-001 — Verify that the tool writes its validation report to stdout in Markdown format.

**Verifies:** REQ-TRS-OUT-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| report is written to stdout in Markdown format | ✓ PASS |

---

### TC-TRS-OUT-002 — Verify that each finding in the report contains severity, code, element reference, and description.

**Verifies:** REQ-TRS-OUT-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| findings table has Code, File, and Message columns | ✓ PASS |

---

### TC-TRS-OUT-003 — Verify that the report includes a summary section with error and warning counts.

**Verifies:** REQ-TRS-OUT-003  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| summary section includes error and warning counts | ✓ PASS |

---

### TC-TRS-OUT-004 — Verify that the tool exits non-zero when any Error-severity finding is present.

**Verifies:** REQ-TRS-OUT-004  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| model with errors exits non-zero | ✓ PASS |
| model with errors and warnings still exits non-zero | ✓ PASS |

---

### TC-TRS-OUT-005 — Verify that the tool exits with code 0 when no Error findings are present.

**Verifies:** REQ-TRS-OUT-005  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| model with only warnings exits 0 | ✓ PASS |
| clean model exits 0 | ✓ PASS |

---

### TC-TRS-OUT-006 — Verify CI severity-gating flags and the 0/1/2 exit-code contract.

**Verifies:** REQ-TRS-OUT-006  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| errors dominate gating flags (exit 1) | ✓ PASS |

---

### TC-TRS-OUT-007 — Verify the structured model graph export (JSON + NDJSON, schemaVersion, resolved relationships).

**Verifies:** REQ-TRS-OUT-007  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| ndjson emits header then elements | ✓ PASS |

---

### TC-TRS-OUT-008 — Verify test-result ingestion and W010 for failing/missing tests (cargo-json + JUnit).

**Verifies:** REQ-TRS-OUT-008  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| junit results supported via --results | ✓ PASS |

---

### TC-TRS-OUT-009 — Verify executed-evidence glyphs/annotations in matrix and trace, plus --linked-only and graceful degradation.

**Verifies:** REQ-TRS-OUT-009  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| no sidecar → no ▣ glyph and no verdict annotations | ✓ PASS |

---

### TC-TRS-OUT-010 — Verify the element-rooted connectivity subgraph export (text tree, JSON nodes/edges, styled DOT, whole-model dump, depth bound).

**Verifies:** REQ-TRS-OUT-010  
**Result:** ✓ PASS (14 passed, 0 failed)

| Scenario | Result |
|---|---|
| unknown root exits non-zero | ✓ PASS |

---

### TC-TRS-OUT-011 — Verify the verification-depth report: per-requirement level depth, flags, filters, JSON, and --min-levels gate.

**Verifies:** REQ-TRS-OUT-011  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| --min-levels 1 passes when all have >=1 level | ✓ PASS |

---

### TC-TRS-PARAM-001 — Verify FeatureDef parameter binding rules E203–E206, E222, and W017.

**Verifies:** REQ-TRS-PARAM-001  
**Result:** ✓ PASS (12 passed, 0 failed)

| Scenario | Result |
|---|---|
| parameter binding violations emit E203–E206, E222, W017 | ✓ PASS |
| valid bindings emit no parameter findings | ✓ PASS |

---

### TC-TRS-PARAM-002 — Verify parameterConstraints evaluation: E221/W025, compound appliesWhen, dotted refs.

**Verifies:** REQ-TRS-PARAM-002  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| W025 is gateable via --deny | ✓ PASS |

---

### TC-TRS-PARAM-003 — Verify inclusive range syntax min..=max is enforced (E205).

**Verifies:** REQ-TRS-PARAM-003  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| binding 99 against range 1..=8 is out of range | ✓ PASS |
| feature-check enforces parameter range (E205) | ✓ PASS |

---

### TC-TRS-PARAM-004 — Verify FeatureDef parameter bindingTime rules E230, E229, W027, and W017 suppression.

**Verifies:** REQ-TRS-PARAM-004  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| a parameter bound no earlier than its source produces no E229 | ✓ PASS |
| binding a runtime parameter in a Configuration produces W027 | ✓ PASS |
| a required unbound runtime parameter suppresses W017 | ✓ PASS |
| a required unbound non-runtime parameter still produces W017 | ✓ PASS |
| well-ordered binding times emit no binding-time findings | ✓ PASS |

---

### TC-TRS-PARSE-001 — Verify that the tool accepts a model root directory path and uses it as the namespace root.

**Verifies:** REQ-TRS-PARSE-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid directory is accepted | ✓ PASS |
| empty directory produces zero elements | ✓ PASS |

---

### TC-TRS-PARSE-002 — Verify that the tool recursively discovers .md files in nested subdirectories.

**Verifies:** REQ-TRS-PARSE-002  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| elements in nested directories are discovered | ✓ PASS |
| non-.md files are ignored | ✓ PASS |

---

### TC-TRS-PARSE-003 — Verify that standard build and tool directories are excluded from discovery.

**Verifies:** REQ-TRS-PARSE-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| target/ directory is excluded from discovery | ✓ PASS |

---

### TC-TRS-PARSE-004 — Verify that .sysmlignore patterns suppress file discovery.

**Verifies:** REQ-TRS-PARSE-004  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| .sysmlignore suppresses matching files | ✓ PASS |
| absence of .sysmlignore causes no error | ✓ PASS |

---

### TC-TRS-PARSE-005 — Verify that _index.md is treated as the package declaration for its directory.

**Verifies:** REQ-TRS-PARSE-005  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| _index.md is treated as package declaration | ✓ PASS |

---

### TC-TRS-PARSE-006 — Verify that a file with unparseable frontmatter produces a warning but does not halt processing.

**Verifies:** REQ-TRS-PARSE-006  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| malformed YAML frontmatter produces a warning, not a fatal error | ✓ PASS |

---

### TC-TRS-PARSE-007 — Verify that frontmatter is recognized only when the opening --- is the first line.

**Verifies:** REQ-TRS-PARSE-007  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| file with --- on first line is parsed correctly | ✓ PASS |
| file with blank first line produces E001 (missing frontmatter) | ✓ PASS |

---

### TC-TRS-PARSE-008 — Verify that invalid YAML frontmatter produces error E002.

**Verifies:** REQ-TRS-PARSE-008  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid YAML frontmatter produces no E002 | ✓ PASS |
| invalid YAML frontmatter produces E002 | ✓ PASS |

---

### TC-TRS-PARSE-009 — Verify that a file without a type: field is skipped with a warning.

**Verifies:** REQ-TRS-PARSE-009  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| file with no type: field is skipped with a warning | ✓ PASS |
| file with type: present is processed normally | ✓ PASS |

---

### TC-TRS-PROJ-001 — Verify the --config projection lens: stored + ad-hoc selection, dormancy, unresolved error.

**Verifies:** REQ-TRS-PROJ-001  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| unresolved configuration errors | ✓ PASS |

---

### TC-TRS-PROJ-002 — Verify full re-validation in the configuration lens.

**Verifies:** REQ-TRS-PROJ-002  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| whole-model validate reports no such coverage gap | ✓ PASS |
| whole-model validate | ✓ PASS |

---

### TC-TRS-PROJ-003 — Verify escaping-reference detection: structural E226 (error), traceability W019 (warning).

**Verifies:** REQ-TRS-PROJ-003  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| references between active elements do not escape | ✓ PASS |

---

### TC-TRS-PROJ-004 — Verify the global appliesWhen-implication guarantee (E227 / W020) with witness.

**Verifies:** REQ-TRS-PROJ-004  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| traceability edge is a warning (W020) | ✓ PASS |

---

### TC-TRS-PROJ-005 — Verify family checks: all-configs gate, dead elements (W021), aggregate coverage (W022), diff.

**Verifies:** REQ-TRS-PROJ-005  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| variant diff lists the symmetric difference | ✓ PASS |

---

### TC-TRS-QNAME-001 — Verify that qualified names are derived correctly from directory path and filename stem.

**Verifies:** REQ-TRS-QNAME-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| single-level element has single-segment qualified name | ✓ PASS |
| three-level nested element has three-segment qualified name | ✓ PASS |

---

### TC-TRS-QNAME-002 — Verify that the name: field in _index.md overrides the directory name in qualified names.

**Verifies:** REQ-TRS-QNAME-002  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| name: in _index.md replaces directory name in qualified names | ✓ PASS |

---

### TC-TRS-QNAME-003 — Verify that the name: field in element frontmatter overrides the filename stem.

**Verifies:** REQ-TRS-QNAME-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| name: in frontmatter replaces the filename stem | ✓ PASS |

---

### TC-TRS-QNAME-004 — Verify that _index.md contributes no name segment to its package or sibling elements.

**Verifies:** REQ-TRS-QNAME-004  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| _index.md contributes no _index name segment | ✓ PASS |

---

### TC-TRS-SAFE-001 — Verify that HazardousEvent validation rules E800-E804, E833-E836, and W800 are enforced

**Verifies:** REQ-TRS-SAFE-001  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E800 | ✓ PASS |
| trigger E801 | ✓ PASS |
| trigger E802 | ✓ PASS |
| trigger E803 | ✓ PASS |
| trigger E804 | ✓ PASS |
| trigger E833 | ✓ PASS |
| trigger E834 | ✓ PASS |
| trigger E835 | ✓ PASS |
| trigger E836 | ✓ PASS |
| trigger W800 | ✓ PASS |

---

### TC-TRS-SAFE-002 — Verify that SafetyGoal validation rules E805-E806, E825, E837, W801, W805, and W806 are enforced

**Verifies:** REQ-TRS-SAFE-002  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E805 | ✓ PASS |
| trigger E806 | ✓ PASS |
| trigger E825 | ✓ PASS |
| trigger E837 | ✓ PASS |
| trigger W801 | ✓ PASS |
| trigger W805 | ✓ PASS |
| trigger W806 | ✓ PASS |

---

### TC-TRS-SAFE-003 — Verify that DamageScenario and ThreatScenario validation rules E807-E814 and E826 are enforced

**Verifies:** REQ-TRS-SAFE-003  
**Result:** ✓ PASS (9 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E807 | ✓ PASS |
| trigger E808 | ✓ PASS |
| trigger E809 | ✓ PASS |
| trigger E810 | ✓ PASS |
| trigger E811 | ✓ PASS |
| trigger E812 | ✓ PASS |
| trigger E813 | ✓ PASS |
| trigger E814 | ✓ PASS |
| trigger E826 | ✓ PASS |

---

### TC-TRS-SAFE-004 — Verify that CybersecurityGoal, SecurityControl, and VulnerabilityReport validation rules E815-E824, E827-E832, W802-W804, and W807 are enforced

**Verifies:** REQ-TRS-SAFE-004  
**Result:** ✓ PASS (20 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E815 | ✓ PASS |
| trigger E816 | ✓ PASS |
| trigger E817 | ✓ PASS |
| trigger E818 | ✓ PASS |
| trigger E819 | ✓ PASS |
| trigger E820 | ✓ PASS |
| trigger E821 | ✓ PASS |
| trigger E822 | ✓ PASS |
| trigger E823 | ✓ PASS |
| trigger E824 | ✓ PASS |
| trigger E827 | ✓ PASS |
| trigger E828 | ✓ PASS |
| trigger E829 | ✓ PASS |
| trigger E830 | ✓ PASS |
| trigger E831 | ✓ PASS |
| trigger E832 | ✓ PASS |
| trigger W802 | ✓ PASS |
| trigger W803 | ✓ PASS |
| trigger W804 | ✓ PASS |
| trigger W807 | ✓ PASS |

---

### TC-TRS-SPEC-001 — Verify the discoverable syscribe spec documents the safety/security types and analysis fields.

**Verifies:** REQ-TRS-SPEC-001  
**Result:** ✓ PASS (61 passed, 0 failed)

| Scenario | Result |
|---|---|
| spec safety documents cveId, safeState and ftti | ✓ PASS |

---

### TC-TRS-SPEC-002 — Verify the discoverable spec includes a ports & interfaces decision guide.

**Verifies:** REQ-TRS-SPEC-002  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| spec types distinguishes the constructs | ✓ PASS |

---

### TC-TRS-TAG-001 — Verify the generic --tag filter selects by free-text tags without affecting variant logic.

**Verifies:** REQ-TRS-TAG-001  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| matrix --tag filters rows but not columns | ✓ PASS |

---

### TC-TRS-TARA-001 — Verify that TARASheet validation rules E940–E941, W905 are enforced.

**Verifies:** REQ-TRS-TARA-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E940 | ✓ PASS |
| trigger E941 | ✓ PASS |
| trigger W905 | ✓ PASS |

---

### TC-TRS-TRACE-001 — Verify that computed reverse indices are populated from downstream link fields.

**Verifies:** REQ-TRS-TRACE-001  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| verifiedBy is computed from TestCase.verifies: | ✓ PASS |

---

### TC-TRS-TRACE-002 — Verify that E310 is emitted when derivedFrom: is present but breakdownAdr: is absent.

**Verifies:** REQ-TRS-TRACE-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| derivedFrom with no breakdownAdr produces E310 | ✓ PASS |
| derivedFrom with valid breakdownAdr produces no E310 | ✓ PASS |

---

### TC-TRS-TRACE-003 — Verify that W303 is emitted when a breakdownAdr: references a proposed ADR on an approved requirement.

**Verifies:** REQ-TRS-TRACE-003  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| approved requirement with proposed ADR produces W303 | ✓ PASS |

---

### TC-TRS-TRACE-004 — Verify that W300 is emitted for an approved leaf Requirement with no satisfying element.

**Verifies:** REQ-TRS-TRACE-004  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| approved leaf requirement with no satisfies produces W300 | ✓ PASS |
| approved leaf requirement with satisfies produces no W300 | ✓ PASS |

---

### TC-TRS-TRACE-005 — Verify that E312 is emitted when a parent Requirement appears in a satisfies: list.

**Verifies:** REQ-TRS-TRACE-005  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| architecture element satisfying a parent requirement produces E312 | ✓ PASS |

---

### TC-TRS-TRACE-006 — Verify that E313 is emitted for incompatible domain/reqDomain in satisfies: links.

**Verifies:** REQ-TRS-TRACE-006  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| software element satisfying hardware requirement produces E313 | ✓ PASS |
| software element satisfying system requirement produces no E313 | ✓ PASS |

---

### TC-TRS-TRACE-007 — Verify that E315 is emitted for cross-domain supertype: or typedBy: links.

**Verifies:** REQ-TRS-TRACE-007  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| software PartDef with hardware supertype produces E315 | ✓ PASS |

---

### TC-TRS-TRACE-008 — Verify that E314 is emitted for a deployment package with no hardware Allocation.

**Verifies:** REQ-TRS-TRACE-008  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| deployment package with no hardware allocation produces E314 | ✓ PASS |
| deployment package with hardware allocation produces no E314 | ✓ PASS |

---

### TC-TRS-TRACE-009 — Verify that E016/E017/E018 are emitted for cycles in supertype, derivedFrom, and subsets graphs.

**Verifies:** REQ-TRS-TRACE-009  
**Result:** ✓ PASS (24 passed, 0 failed)

| Scenario | Result |
|---|---|
| supertype cycle produces E016 | ✓ PASS |
| derivedFrom cycle produces E017 | ✓ PASS |
| subsets cycle produces E018 | ✓ PASS |
| typedBy self-reference produces E107 | ✓ PASS |
| typedBy cycle produces E107 | ✓ PASS |
| acyclic model produces no cycle errors | ✓ PASS |

---

### TC-TRS-TRACE-010 — Verify the unsatisfied safety-mechanism check W306 (high-integrity + draft/unsatisfied/all-N-A).

**Verifies:** REQ-TRS-TRACE-010  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| W306 message names the sub-conditions | ✓ PASS |
| W306 is gateable with --deny | ✓ PASS |

---

### TC-TRS-VAL-001 — Verify that each parse-time error rule is triggered by the corresponding malformed input.

**Verifies:** REQ-TRS-VAL-001  
**Result:** ✓ PASS (19 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E001 | ✓ PASS |
| trigger E002 | ✓ PASS |
| trigger E004 | ✓ PASS |
| trigger E005 | ✓ PASS |
| trigger E006 | ✓ PASS |
| trigger E007 | ✓ PASS |
| trigger E008 | ✓ PASS |
| trigger E009 | ✓ PASS |
| trigger E010 | ✓ PASS |
| trigger E011 | ✓ PASS |
| trigger E012 | ✓ PASS |
| trigger E013 | ✓ PASS |
| trigger E014 | ✓ PASS |
| trigger E015 | ✓ PASS |
| trigger E300 | ✓ PASS |
| trigger E301 | ✓ PASS |
| trigger E302 | ✓ PASS |
| trigger E303 | ✓ PASS |
| trigger E304 | ✓ PASS |

---

### TC-TRS-VAL-002 — Verify that each model-time error rule is triggered by its cross-element condition.

**Verifies:** REQ-TRS-VAL-002  
**Result:** ✓ PASS (12 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E101 | ✓ PASS |
| trigger E102 | ✓ PASS |
| trigger E103 | ✓ PASS |
| trigger E104 | ✓ PASS |
| trigger E105 | ✓ PASS |
| trigger E106 | ✓ PASS |
| trigger E310 | ✓ PASS |
| trigger E311 | ✓ PASS |
| trigger E312 | ✓ PASS |
| trigger E313 | ✓ PASS |
| trigger E314 | ✓ PASS |
| trigger E315 | ✓ PASS |

---

### TC-TRS-VAL-003 — Verify that each warning rule is triggered by its condition with Warning severity.

**Verifies:** REQ-TRS-VAL-003  
**Result:** ✓ PASS (13 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger W001 | ✓ PASS |
| trigger W002 | ✓ PASS |
| trigger W003 | ✓ PASS |
| trigger W004 | ✓ PASS |
| trigger W005 | ✓ PASS |
| trigger W006 | ✓ PASS |
| trigger W007 | ✓ PASS |
| trigger W300 | ✓ PASS |
| trigger W301 | ✓ PASS |
| trigger W302 | ✓ PASS |
| trigger W303 | ✓ PASS |
| trigger W304 | ✓ PASS |
| trigger W305 | ✓ PASS |

---

### TC-TRS-VAL-004 — Verify that integrity-level propagation errors E841-E843 and W808 are enforced.

**Verifies:** REQ-TRS-VAL-004  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| E841: derivedFromSafetyGoal element missing integrity level | ✓ PASS |
| E842: derivedFrom element missing integrity level | ✓ PASS |
| E843: satisfies element missing integrity level | ✓ PASS |
| W808: integrity level lower than source without breakdownAdr | ✓ PASS |

---

### TC-TRS-VAL-005 — Verify that each finding includes the required fields: rule code, element reference, and description.

**Verifies:** REQ-TRS-VAL-005,REQ-TRS-VAL-006  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| findings include rule code, element reference, and description | ✓ PASS |
| parse-time error is attributed to source file | ✓ PASS |

---

### TC-TRS-VAL-006 — Verify that E-code findings are marked Error and W-code findings are marked Warning.

**Verifies:** REQ-TRS-VAL-007  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| E-code findings are under Errors section | ✓ PASS |
| W-code findings are under Warnings section | ✓ PASS |

---

### TC-TRS-VAL-007 — Verify that Error and Warning severity are reported consistently in the output.

**Verifies:** REQ-TRS-VAL-007  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| Errors section does not contain W-codes | ✓ PASS |
| Warnings section does not contain E-codes | ✓ PASS |

---

### TC-TRS-VAL-008 — Verify that safety-level, standards-compliance, and type-field validation rules are enforced.

**Verifies:** REQ-TRS-VAL-008  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E019 | ✓ PASS |
| trigger E020 | ✓ PASS |
| trigger E021 | ✓ PASS |
| trigger E022 | ✓ PASS |
| trigger W703 | ✓ PASS |
| W008: file with valid frontmatter but no type: field | ✓ PASS |
| W701: Requirement with asilLevel B/C/D and no verificationMethod | ✓ PASS |
| W702: ASIL-D Requirement with active TestCase but none at L5 | ✓ PASS |

---

### TC-TRS-VAL-009 — Verify that E500-E503, W500-W502, and W600-W601 are emitted for Allocation, View, and documentation violations.

**Verifies:** REQ-TRS-VAL-009  
**Result:** ✓ PASS (9 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E500 | ✓ PASS |
| trigger E501 | ✓ PASS |
| trigger E502 | ✓ PASS |
| trigger E503 | ✓ PASS |
| trigger W500 | ✓ PASS |
| trigger W501 | ✓ PASS |
| trigger W502 | ✓ PASS |
| trigger W600 | ✓ PASS |
| trigger W601 | ✓ PASS |

---

### TC-TRS-VAL-010 — Verify function-level traceability (W009) across all supported languages and generic files.

**Verifies:** REQ-TRS-VAL-010  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| resolving functions across all languages produce no W009 | ✓ PASS |
| renamed/missing tests produce W009 | ✓ PASS |

---

### TC-TRS-VAL-011 — Verify actionable E106 messages and scaffold-gherkin --fix alignment.

**Verifies:** REQ-TRS-VAL-011  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| scaffold-gherkin --fix clears E106 | ✓ PASS |

---

### TC-TRS-VAL-012 — Verify sourceFile location semantics: model-relative, absolute, file URI, and remote URI.

**Verifies:** REQ-TRS-VAL-012  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| absolute and file:// resolve without new W004 | ✓ PASS |

---

### TC-TRS-VAL-013 — Verify the remote sourceFile download hook: opt-in fetch, function verification, and retrieval-failure flagging.

**Verifies:** REQ-TRS-VAL-013  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| --fetch-remote fetches and verifies | ✓ PASS |

---

### TC-TRS-VAL-014 — Verify W004/W009 fire for active TestCases only, while non-TestCase sourceFiles are still checked.

**Verifies:** REQ-TRS-VAL-014  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| drift checks scoped to active TestCases | ✓ PASS |

---

### TC-TRS-VAL-015 — Verify informational I010 for planned TestCase sources: emitted for draft, deniable, exit-neutral, none for retired.

**Verifies:** REQ-TRS-VAL-015  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| --deny I010 gates | ✓ PASS |

---

### TC-TRS-VAL-016 — Verify wcet queryability (--has-wcet, list --json) and the W029 WCET-not-measured check.

**Verifies:** REQ-TRS-VAL-016  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| list --has-wcet --json includes wcet | ✓ PASS |
| SIL requirement with wcet, no measuring test produces W029 | ✓ PASS |
| W029 is gateable with --deny | ✓ PASS |

---

### TC-TRS-VAR-001 — Verify that the variability dimension is dormant unless a feature model is linked.

**Verifies:** REQ-TRS-VAR-001  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| matrix on flat model falls back without error | ✓ PASS |
| unresolved appliesWhen is E209 when dormant | ✓ PASS |
| feature model without Configuration emits no W015 | ✓ PASS |

---

### TC-TRS-VAR-002 — Verify TestCase-to-Configuration membership is derived from appliesWhen.

**Verifies:** REQ-TRS-VAR-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| refs of deselecting config excludes conditioned TestCase | ✓ PASS |

---

### TC-TRS-VAR-003 — Verify boolean expressions (and/or/not/parens) in appliesWhen parse and evaluate.

**Verifies:** REQ-TRS-VAR-003  
**Result:** ✓ PASS (14 passed, 0 failed)

| Scenario | Result |
|---|---|
| bare QName remains back-compatible | ✓ PASS |

---

### TC-TRS-VAR-004 — Verify the matrix command emits a Requirement x Configuration coverage grid.

**Verifies:** REQ-TRS-VAR-004  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| text matrix renders configurations and cells | ✓ PASS |

---

### TC-TRS-VAR-005 — Verify per-Configuration uncovered-requirement rule W015 and its suppression/gating.

**Verifies:** REQ-TRS-VAR-005  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| W015 is gateable with --deny | ✓ PASS |
| dormant model emits no W015 | ✓ PASS |

---

### TC-TRS-VAR-006 — Verify transitive package appliesWhen: effective condition, E228 nesting/placement, W026, escapes.

**Verifies:** REQ-TRS-VAR-006  
**Result:** ✓ PASS (18 passed, 0 failed)

| Scenario | Result |
|---|---|
| good model has no E228/W026 | ✓ PASS |
| nested appliesWhen under a gated package is E228 | ✓ PASS |
| appliesWhen on FeatureDef / Configuration / config package is E228 | ✓ PASS |
| appliesWhen on the model root is E228 | ✓ PASS |
| package gating an empty subtree is W026 | ✓ PASS |
| external ref into gated subtree escapes; internal ref does not | ✓ PASS |

---

### TC-TRS-XREF-001 — Verify that absolute qualified names are resolved correctly from the model root.

**Verifies:** REQ-TRS-XREF-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| absolute supertype reference resolves correctly | ✓ PASS |
| absolute reference to non-existent element produces an error | ✓ PASS |

---

### TC-TRS-XREF-002 — Verify that relative references are resolved outward from the current package.

**Verifies:** REQ-TRS-XREF-002  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| sibling reference resolves within the same package | ✓ PASS |

---

### TC-TRS-XREF-003 — Verify that an unresolved cross-reference produces an error but does not abort processing.

**Verifies:** REQ-TRS-XREF-003  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| dangling reference produces an error without crashing | ✓ PASS |

---

### TC-TRS-XREF-004 — Verify that circular supertype chains are detected and reported without crashing.

**Verifies:** REQ-TRS-XREF-004  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| two-element supertype cycle is detected without crashing | ✓ PASS |

---

### TC-TRS-XREF-005 — Verify that verifies: and derivedFrom: references are resolved by stable id:.

**Verifies:** REQ-TRS-XREF-005  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| verifies: resolves by stable id: regardless of file path | ✓ PASS |

---
