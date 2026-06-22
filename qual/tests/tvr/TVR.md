# Tool Validation Report

**Tool:** syscribe CLI validator  
**Version:** syscribe 0.26.40  
**Standard:** ISO 26262:2018 Part 8 §11 (TCL2), IEC 61508:2010 Part 3 Annex D  
**Date:** 2026-06-22  
**TRS:** `qual/Requirements/`  **Test cases:** `qual/TestCases/`

---

## 1. Summary

| Metric | Value |
|---|---|
| Total test cases | 246 |
| Passed | 246 |
| Failed | 0 |
| Overall verdict | **PASS** |

---

## 2. Results

### TC-TRS-ALLOC-001 — Verify two allocation forms over one edge model: allocatedTo-on-source clears MG041/MG081 + derives allocatedFrom; type-less legacy features are edges; E503 unresolved; W503 redundant duplicate.

**Verifies:** REQ-TRS-ALLOC-001  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| the same edge in both forms raises W503 | ✓ PASS |

---

### TC-TRS-AW-001 — Verify the applies-when CLI: set by feature id or path, E209/E228 refusal, void-model bad-config check on set, clear, dry-run.

**Verifies:** REQ-TRS-AW-001  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| dry-run validates but writes nothing | ✓ PASS |

---

### TC-TRS-AW-002 — Verify applies-when read mode: own gate, inherited (package) effective gate, always-applies, read-only, --json, unresolved.

**Verifies:** REQ-TRS-AW-002  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| an unresolved element exits non-zero | ✓ PASS |

---

### TC-TRS-BUDGET-001 — Verify the budget expression language: a resolvable in-bound budget is clean; E866 (evaluate not a ConstraintDef), E867 (syntax error), E868 (unresolved operand), W060 (value violates the evaluate constraint); W060 draft-suppressed and gateable.

**Verifies:** REQ-TRS-BUDGET-001  
**Result:** ✓ PASS (11 passed, 0 failed)

| Scenario | Result |
|---|---|
| well-formed in-bound budget is clean | ✓ PASS |
| E866 — evaluate not a ConstraintDef | ✓ PASS |
| E867 — malformed budget expression | ✓ PASS |
| E868 — unresolved operand | ✓ PASS |
| W060 — budget violates the evaluate constraint | ✓ PASS |
| W060 draft-suppressed | ✓ PASS |
| validate --deny W060 promotes to gate failure | ✓ PASS |

---

### TC-TRS-CFLD-001 — Verify custom_fields shape validation: scalars/lists clean, nested map raises W041.

**Verifies:** REQ-TRS-CFLD-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| a nested-map custom field raises W041 | ✓ PASS |

---

### TC-TRS-CFLD-002 — Verify the --where custom-field query: exact, regex, list-membership, presence, and bad-predicate exit.

**Verifies:** REQ-TRS-CFLD-002  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| unparseable predicate exits non-zero | ✓ PASS |

---

### TC-TRS-CFLD-003 — Verify show renders a custom-fields section when present and omits it when absent.

**Verifies:** REQ-TRS-CFLD-003  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| show omits the section when absent | ✓ PASS |

---

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

### TC-TRS-CLI-005 — Verify detailed per-command help: help <cmd>, <cmd> --help, the index, and unknown handling.

**Verifies:** REQ-TRS-CLI-005  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| help <unknown> exits non-zero | ✓ PASS |

---

### TC-TRS-CLI-006 — Verify --agent-instructions topic: magicgrid prints the MagicGrid prompt; no topic prints the general prompt; an unknown topic exits non-zero; works with no model directory.

**Verifies:** REQ-TRS-CLI-006  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| works without a model directory | ✓ PASS |

---

### TC-TRS-CLI-007 — Verify version reporting via --version, -V, and the version subcommand (exit 0, no model dir).

**Verifies:** REQ-TRS-CLI-007  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| works from a directory with no model or .syscribe.toml | ✓ PASS |

---

### TC-TRS-CLI-008 — Verify the clap top-level router: unknown command rejected (non-zero, no model needed); known commands, man-page help, and version preserved.

**Verifies:** REQ-TRS-CLI-008  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| explicit 'report' runs the default validation report | ✓ PASS |

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

### TC-TRS-DERIVE-001 — Basic derive block evaluates and appears in query show output

**Verifies:**   
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| Validation reports no errors for valid derive block | ✓ PASS |

---

### TC-TRS-DERIVE-002 — Aggregate operators sum and count work over children

**Verifies:**   
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| count(children) returns number of direct children | ✓ PASS |

---

### TC-TRS-DERIVE-003 — Null coalesce and custom_fields arithmetic evaluate correctly

**Verifies:**   
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| No errors for valid arithmetic derive block | ✓ PASS |

---

### TC-TRS-DERIVE-004 — Invalid derive formula emits E501 parse error

**Verifies:**   
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| E501 message names the field 'broken' | ✓ PASS |

---

### TC-TRS-DERIVE-005 — Cross-element reference to unknown element emits E502

**Verifies:**   
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| E502 message names the missing element | ✓ PASS |

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

### TC-TRS-DIAG-002 — Verify W080 — a Sequence diagram raises a finding for each SendAction/AcceptAction of its subject ActionDef not covered by an edge; covered diagrams are clean; draft-suppressed; gateable with --deny W080.

**Verifies:** REQ-TRS-DIAG-002  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| W080 — uncovered SendAction | ✓ PASS |
| no W080 — every action covered | ✓ PASS |
| W080 — nested SendAction in IfAction then-branch | ✓ PASS |
| W080 draft-suppressed | ✓ PASS |
| validate --deny W080 promotes to gate failure | ✓ PASS |

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

### TC-TRS-FMEA-002 — Verify FMEA entry canonical fields: fmeaSeverity accepted, RPN auto-computed, unknown keys raise E922

**Verifies:** REQ-TRS-FMEA-002  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| template FMEASheet emits fmeaSeverity not severity | ✓ PASS |

---

### TC-TRS-FMEA-003 — Verify fmeaRef on FaultTreeEvent and ftaRef on FMEAEntry cross-references raise W926/W927 on broken refs

**Verifies:** REQ-TRS-FMEA-003  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| refs FM-KERN-001 lists FTE-KERN-001 as referencing via fmeaRef | ✓ PASS |

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

### TC-TRS-ID-005 — Verify configurable stable-ID suffix width: 3-8 default, E023 over the cap, E006 under 3, configurable via [ids] max_digits.

**Verifies:** REQ-TRS-ID-005  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| max_digits=4 tightens the cap: a 5-digit id trips E023 | ✓ PASS |

---

### TC-TRS-ID-006 — Verify FeatureDef stable FEAT id: mandatory id (E201), id-or-qname feature references in appliesWhen and Configuration features; E006/E101/E209 rules.

**Verifies:** REQ-TRS-ID-006  
**Result:** ✓ PASS (21 passed, 0 failed)

| Scenario | Result |
|---|---|
| a stable-id-shaped reference resolving to nothing raises E209 | ✓ PASS |

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

### TC-TRS-LIB-001 — Verify built-in type recognition: ScalarValues/Base members resolve with no W404/W043; unknown members raise W043; import-only packages stay lenient.

**Verifies:** REQ-TRS-LIB-001  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| the check covers Base and multiple contexts (supertype, typedBy, returnType, parameter type) | ✓ PASS |

---

### TC-TRS-LIB-002 — Verify SI/ISQ recognition (open/curated tier): recognised members resolve clean (no W404), unknown members lenient (no W043), unit: permissive; closed-package W043 unaffected.

**Verifies:** REQ-TRS-LIB-002  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| non-SI domain units in unit: are permissive (no finding) | ✓ PASS |

---

### TC-TRS-LIB-003 — Verify dimensional consistency (W044): quantity-type vs unit dimension must match; mismatch flagged; bare symbols handled; lenient when either side unrecognised.

**Verifies:** REQ-TRS-LIB-003  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| a non-quantity typedBy makes the check lenient (no W044) | ✓ PASS |

---

### TC-TRS-LINK-001 — Verify hosted source URLs resolve from the [links] config for file-backed elements.

**Verifies:** REQ-TRS-LINK-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| no [links] table yields no URLs | ✓ PASS |

---

### TC-TRS-LINK-002 — Verify SVG element shapes are wrapped in a hyperlink to the hosted URL.

**Verifies:** REQ-TRS-LINK-002  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| no [links] table leaves shapes unwrapped by hosted links | ✓ PASS |

---

### TC-TRS-LINK-003 — Verify Mermaid diagrams emit click directives to the hosted URL.

**Verifies:** REQ-TRS-LINK-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| no [links] table emits no hosted click directive | ✓ PASS |

---

### TC-TRS-LINK-004 — Verify Markdown report and export render element references as hosted links.

**Verifies:** REQ-TRS-LINK-004  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| no [links] table leaves the report unlinked | ✓ PASS |

---

### TC-TRS-LINK-005 — Verify the live web UI detail panel shows a per-element source-link icon to the hosted model element.

**Verifies:** REQ-TRS-LINK-005  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| live detail panel renders/omits the source-link icon per [links] | ✓ PASS |

---

### TC-TRS-LINT-001 — Verify lint-docs scans external Markdown for unresolvable stable ID tokens and exits non-zero

**Verifies:** REQ-TRS-LINT-001  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| file with no stable-ID tokens exits 0 and produces no output | ✓ PASS |

---

### TC-TRS-LINTDOC-001 — Verify lint-docs diagram resolution: W100 for an unresolved Mermaid qualified name, W101 for a stale SVG sysml:ref, W102 for a missing image embed; resolving refs and prose qnames are clean; --json shape.

**Verifies:** REQ-TRS-LINT-002  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| W100 — unresolved Mermaid qname + W102 missing embed | ✓ PASS |
| W101 — stale SVG sysml:ref | ✓ PASS |
| resolving refs and prose qnames are clean | ✓ PASS |
| --json shape | ✓ PASS |

---

### TC-TRS-META-001 — Verify stereotypes as MetadataDef applications: valid apply (bare + tagged), E317 unresolved, E318 appliesTo mismatch, W045 undeclared tag key, show «Name», list --metadata.

**Verifies:** REQ-TRS-META-001  
**Result:** ✓ PASS (9 passed, 0 failed)

| Scenario | Result |
|---|---|
| list --metadata filters to elements applying the stereotype | ✓ PASS |

---

### TC-TRS-META-002 — Verify diagrams render applied MetadataDef stereotypes as «Name» banners: a stereotyped element shows «Critical» in addition to its type-keyword banner; an element with no application shows no spurious banner.

**Verifies:** REQ-TRS-META-002  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| an element with no application renders no spurious stereotype banner | ✓ PASS |

---

### TC-TRS-MG-001 — Verify the refines link on UseCaseDef: write/parse, refinedBy index, E316 bad target, W307 missing (draft-suppressed), magicgrid profile promotion.

**Verifies:** REQ-TRS-MG-001  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| the magicgrid profile promotes W307 to a gate failure | ✓ PASS |

---

### TC-TRS-MG-002 — Verify gated actor validation: inert without the gate; MG010 unresolved, MG011 non-part, MG012 not external, MG013 no actors; actorIn index.

**Verifies:** REQ-TRS-MG-002  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| actor participation is indexed | ✓ PASS |

---

### TC-TRS-MG-003 — Verify mg_cell classification and the magicgrid grid report: MG020 invalid coord, MG021 type/column mismatch, grid render with empty-cell flag, --json.

**Verifies:** REQ-TRS-MG-003  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| mg_cell is inert without the magicgrid profile | ✓ PASS |

---

### TC-TRS-MG-004 — Verify MoE validation: valid MoE clean; MG030 wrong host, MG031 measures missing/unresolved, MG032 bad direction, MG033 bad bounds; inert without the gate.

**Verifies:** REQ-TRS-MG-004  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| mg_moe fields are inert without the gate | ✓ PASS |

---

### TC-TRS-MG-005 — Verify logical/physical layering: MG040 bad layer, MG041 unrealised logical, MG042 cross-layer coupling; allocation clears MG042; inert without the gate.

**Verifies:** REQ-TRS-MG-005  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| mg_layer is inert without the gate | ✓ PASS |

---

### TC-TRS-MG-006 — Verify the allocation matrix view: rows=sources, cols=targets, allocated cells; unallocated/unused rollup; mg_layer partition; flat fallback; --json.

**Verifies:** REQ-TRS-MG-006  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| the matrix emits JSON | ✓ PASS |

---

### TC-TRS-MG-007 — Verify the MoE-weighted trade study: MoE rows x configuration columns; objective scores 1.0; sub-threshold scores 0 and fails the config; ranked rollup; --config; --json; n/a cells.

**Verifies:** REQ-TRS-MG-007  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| the trade study emits JSON | ✓ PASS |

---

### TC-TRS-MG-008 — Verify MoP validation: clean MoP + mopRefinedBy index; MG050 wrong host, MG051 refines missing/unresolved, MG052 target not an MoE; inert without the gate.

**Verifies:** REQ-TRS-MG-008  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| mg_mop fields are inert without the gate | ✓ PASS |

---

### TC-TRS-MG-009 — Verify SoI marker: single SoI clean + identified in magicgrid report; MG060 wrong host, MG061 multiple SoI, MG062 also external; inert without the gate.

**Verifies:** REQ-TRS-MG-009  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| mg_soi is inert without the gate | ✓ PASS |

---

### TC-TRS-MG-010 — Verify refines on behavioral defs: ActionDef/StateDef refine resolves + refinedBy; E316 on bad target; no W307 on a behavioral def.

**Verifies:** REQ-TRS-MG-010  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| a behavioral def with no refines raises no W307 | ✓ PASS |

---

### TC-TRS-MG-011 — Verify mg_variant Configuration: no E201 without featureModel when marked; E201 still fires unmarked; trade-study scores it; identity projection; MG070 on non-Configuration.

**Verifies:** REQ-TRS-MG-011  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| mg_variant on a non-Configuration raises MG070 under the gate | ✓ PASS |

---

### TC-TRS-MG-012 — Verify trade-study ambiguous binding handling: colliding final-segment bindings => n/a; an exact key wins; a single segment match still resolves.

**Verifies:** REQ-TRS-MG-012  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| a single segment match still resolves | ✓ PASS |

---

### TC-TRS-MG-013 — Verify magicgrid --audit: clean model PASS (exit 0) + readiness; a MagicGrid error lists the code and FAILs (exit 2); plain magicgrid has no verdict; --json.

**Verifies:** REQ-TRS-MG-013  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| the audit emits JSON | ✓ PASS |

---

### TC-TRS-MG-014 — Verify MagicGrid completeness checks: MG080 orphan need, MG081 unallocated W2 function, MG082 missing SoI, MG083 MoE without MoP; each clears when satisfied; inert without the gate.

**Verifies:** REQ-TRS-MG-014  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| the completeness checks are inert without the gate | ✓ PASS |

---

### TC-TRS-MG-015 — Verify the magicgrid report renders the 3x4 B/W/S × pillar grid matrix (counts, SoI marker, empty cells) alongside the per-cell detail.

**Verifies:** REQ-TRS-MG-015  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| the per-cell element detail is retained | ✓ PASS |

---

### TC-TRS-MG-016 — Verify magicgrid --svg emits a well-formed SVG of the grid (rows/pillars/SoI), -o writes a file, and the SVG works as a Diagram companion (no E402).

**Verifies:** REQ-TRS-MG-016  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| a long cell label is word-wrapped, not truncated | ✓ PASS |

---

### TC-TRS-MG-017 — Verify the companion matrices render as 2-D grids: matrix --allocations is a sources×targets ✓ matrix with a gap rollup; trade-study is a Configuration×MoE matrix with a winner.

**Verifies:** REQ-TRS-MG-017  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| the winning configuration is marked | ✓ PASS |

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

### TC-TRS-NAME-001 — Verify SysMLv2 basic-name validation: W042 on non-basic names, exempting stable ids; hyphenated appliesWhen still E209.

**Verifies:** REQ-TRS-NAME-001  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| a basic directory name is not flagged | ✓ PASS |

---

### TC-TRS-NAME-002 — Verify name is the universal label: E024 retired, E025 fires on any title field, FeatureDef id+name clean.

**Verifies:** REQ-TRS-NAME-002  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| a FeatureDef with a FEAT id and a name (no title) is clean of both | ✓ PASS |

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

### TC-TRS-OUT-012 — Verify named, SIL/ASIL-scopable validation severity profiles and their exit codes.

**Verifies:** REQ-TRS-OUT-012  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| --profile nonexistent is an error (exit 1) | ✓ PASS |

---

### TC-TRS-OUT-013 — Verify the safety-readiness audit dashboard, its sections, JSON output and PASS/FAIL exit codes.

**Verifies:** REQ-TRS-OUT-013  
**Result:** ✓ PASS (21 passed, 0 failed)

---

### TC-TRS-OUT-014 — Verify list TestCase emits test-execution table and JSON with testFunctions; config+tag combined.

**Verifies:** REQ-TRS-OUT-014,REQ-TRS-TAG-002  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| list Requirement still uses the generic table (no regression) | ✓ PASS |

---

### TC-TRS-OUT-015 — Verify list AssumptionOfUse emits SRAC-oriented table columns and appliesTo/body in JSON

**Verifies:** REQ-TRS-OUT-015  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| list Requirement still uses generic table (no regression) | ✓ PASS |

---

### TC-TRS-OUT-016 — Verify the n2 interface-matrix command: connection edges appear as named interfaces in the right cells; --allocations adds allocation edges; --format json matches the schema; --format html is a table; --interfaces-only and --depth behave.

**Verifies:** REQ-TRS-OUT-016  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| connection edges appear as named interfaces | ✓ PASS |
| --allocations adds allocation edges | ✓ PASS |
| --format json matches the schema | ✓ PASS |
| --format html is a table | ✓ PASS |
| --interfaces-only retains wired elements | ✓ PASS |

---

### TC-TRS-OUT-017 — Verify the impact command: downstream reaches derived children / satisfying elements / verifying tests; upstream traces back to the safety goal; --kinds and --depth filter; --format json/dot are well-formed.

**Verifies:** REQ-TRS-OUT-017  
**Result:** ✓ PASS (14 passed, 0 failed)

| Scenario | Result |
|---|---|
| downstream reaches the full chain | ✓ PASS |
| upstream traces back to the safety goal | ✓ PASS |
| --kinds restricts to verifies | ✓ PASS |
| --depth 1 limits the hops | ✓ PASS |
| --format json schema | ✓ PASS |
| --format dot is a digraph | ✓ PASS |
| qualified-name root works | ✓ PASS |

---

### TC-TRS-OUT-018 — Verify behavioral-coverage: source-overlap (path 1) and allocation (path 4) coverage, active-only by default, --include-planned, --uncovered-only, json schema, correct percentage, and the demo model >50%.

**Verifies:** REQ-TRS-OUT-018  
**Result:** ✓ PASS (11 passed, 0 failed)

| Scenario | Result |
|---|---|
| default coverage (paths 1 and 4) | ✓ PASS |
| --include-planned surfaces planned coverage | ✓ PASS |
| --uncovered-only keeps the true percentage | ✓ PASS |
| json schema | ✓ PASS |
| demo model >50% behavioral coverage | ✓ PASS |

---

### TC-TRS-OUT-019 — Verify the sbom command: local paths → file components with model externalReferences; registry URIs → package components with PURLs; CycloneDX 1.6 and SPDX 2.3 are well-formed; --include-tests adds test components; --output writes a file.

**Verifies:** REQ-TRS-OUT-019  
**Result:** ✓ PASS (11 passed, 0 failed)

| Scenario | Result |
|---|---|
| CycloneDX file + package components | ✓ PASS |
| local component links to the requirement | ✓ PASS |
| registry URIs become PURLs | ✓ PASS |
| SPDX 2.3 output | ✓ PASS |
| --include-tests adds test components | ✓ PASS |
| --output writes a file | ✓ PASS |

---

### TC-TRS-OUT-020 — Verify export-reqif: well-formed ReqIF 1.2 XML; a SPEC-OBJECT per Requirement; nested SPEC-HIERARCHY for packages; DERIVED_FROM relations; --include-tests adds VERIFIED_BY; --zip writes a readable .reqifz.

**Verifies:** REQ-TRS-OUT-020  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| output is well-formed ReqIF XML | ✓ PASS |
| a SPEC-OBJECT per requirement | ✓ PASS |
| package hierarchy as nested SPEC-HIERARCHY | ✓ PASS |
| derivedFrom becomes DERIVED_FROM | ✓ PASS |
| --include-tests adds VERIFIED_BY | ✓ PASS |
| --zip writes a readable .reqifz | ✓ PASS |

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

### TC-TRS-PLAN-001 — Verify the native TestPlan schema: TP-id pattern, status enum, scope vocabulary and duplicate-id.

**Verifies:** REQ-TRS-PLAN-001  
**Result:** ✓ PASS (11 passed, 0 failed)

| Scenario | Result |
|---|---|
| template TestPlan emits a valid skeleton | ✓ PASS |

---

### TC-TRS-PLAN-002 — Verify TestPlan configuration binding: E606, escaping member W611 and duplicate-plan W616.

**Verifies:** REQ-TRS-PLAN-002  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| two plans with identical (configurations, scope) raise W616 | ✓ PASS |

---

### TC-TRS-PLAN-003 — Verify TestPlan membership: E601, E602, E605, empty-set W612 and explicit-draft W613.

**Verifies:** REQ-TRS-PLAN-003  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| an explicitly named draft TestCase raises W613 | ✓ PASS |

---

### TC-TRS-PLAN-004 — Verify TestPlan demonstrated goals: E603 and the evidence-gap W614.

**Verifies:** REQ-TRS-PLAN-004  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| demonstrating a parent goal whose leaf is tested does not raise W614 (goal-closure) | ✓ PASS |

---

### TC-TRS-PLAN-005 — Verify the testplan command: list, detail --json contract, goal-closure in-scope, verdict roll-up.

**Verifies:** REQ-TRS-PLAN-005  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| testplan on an unknown plan exits non-zero | ✓ PASS |

---

### TC-TRS-PLAN-006 — Verify the --plan lens on matrix, verification-depth and audit: row restriction, scoped verdict, composition, unknown-id exit.

**Verifies:** REQ-TRS-PLAN-006  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| audit --plan scopes the verdict to the plan (no escaping-ref artifacts) | ✓ PASS |

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

### TC-TRS-PROJ-006 — Verify the --config projection lens on metrics, cyber-risk, co-analysis, verification-depth and safety-case.

**Verifies:** REQ-TRS-PROJ-006  
**Result:** ✓ PASS (12 passed, 0 failed)

| Scenario | Result |
|---|---|
| verification-depth projects out a gated SIL-3 requirement | ✓ PASS |

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

### TC-TRS-RPT-001 — Verify fmea report emits RPN-sorted Markdown table and fault-tree render emits Mermaid flowchart

**Verifies:** REQ-TRS-RPT-001  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| fault-tree render FT-KERN-001 emits Mermaid flowchart | ✓ PASS |

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

### TC-TRS-SAFE-005 — Verify SPFM/LFM/PMHF computation, ASIL/SIL gating (W033), diagnosticCoverage range (E846), and the metrics command

**Verifies:** REQ-TRS-SAFE-005  
**Result:** ✓ PASS (13 passed, 0 failed)

| Scenario | Result |
|---|---|
| metrics --json output | ✓ PASS |
| --deny W033 exits non-zero | ✓ PASS |

---

### TC-TRS-SAFE-006 — Verify W034 freedom-from-interference detection for mixed-criticality shared resources, the FFI-argument exemptions, and the opt-in rule

**Verifies:** REQ-TRS-SAFE-006  
**Result:** ✓ PASS (11 passed, 0 failed)

| Scenario | Result |
|---|---|
| flagged: mixed-criticality sharing with no FFI argument | ✓ PASS |
| excused (ffiRationale): no W034 | ✓ PASS |
| excused (accepted breakdownAdr): no W034 | ✓ PASS |
| --deny W034 exits non-zero | ✓ PASS |

---

### TC-TRS-SAFE-007 — Verify W038 work-product responsibility, the ConfirmationMeasure type with W039 independent-assessment coverage, the E84x structural errors, and the opt-in rules

**Verifies:** REQ-TRS-SAFE-007  
**Result:** ✓ PASS (17 passed, 0 failed)

| Scenario | Result |
|---|---|
| dia: non-draft work product with no responsibility | ✓ PASS |
| dia_clean: every work product has responsibility, no W038 | ✓ PASS |
| confirm: ASIL D goal without I3 functional_safety_assessment | ✓ PASS |
| confirmed: I3 functional_safety_assessment present, no W039 | ✓ PASS |
| badenum: invalid measureType/independenceLevel | ✓ PASS |
| --deny W039 exits non-zero | ✓ PASS |

---

### TC-TRS-SAFE-008 — Verify the GSN argument layer (Argument ARG-*, AssumptionOfUse AOU-*), the E852–E858/W040 checks, and the safety-case (GSN) view incl. the implicit goal→req→test fold-in

**Verifies:** REQ-TRS-SAFE-008  
**Result:** ✓ PASS (23 passed, 0 failed)

| Scenario | Result |
|---|---|
| main: Argument + AssumptionOfUse validate with no errors | ✓ PASS |
| badref: unresolved Argument.supports/evidence yields E855 | ✓ PASS |
| orphan: claim Argument with no supports/evidence yields W040 | ✓ PASS |

---

### TC-TRS-SAFE-009 — Verify W039 fires for silLevel 3 and 4 SafetyGoals missing I3 assessment

**Verifies:** REQ-TRS-SAFE-009  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| W039 message mentions IEC 61508-1 | ✓ PASS |

---

### TC-TRS-SAFE-010 — Verify safety-case appends [unknown] verdict footnote when no results sidecar is loaded

**Verifies:** REQ-TRS-SAFE-010  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| safety-case text includes the ingest-results tip command | ✓ PASS |

---

### TC-TRS-SAFE-011 — Verify safety-case suppresses implicit fold-in per-goal and globally via --no-implicit

**Verifies:** REQ-TRS-SAFE-011  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| JSON has empty requirements array for goal with explicit Argument | ✓ PASS |

---

### TC-TRS-SAFE-012 — Verify ASIL D / SIL 4 decomposition pair completeness: E865 when siblings share a satisfies target, W860 for a single-child decomposition, clean when distinct; decompositionKind surfaces in the safety-case report and the Requirement template; draft-suppressed; gateable.

**Verifies:** REQ-TRS-SAFE-012  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| E865 — siblings share a satisfies target | ✓ PASS |
| no E865 — distinct satisfies targets | ✓ PASS |
| W860 — single-child decomposition | ✓ PASS |
| W860 draft-suppressed | ✓ PASS |
| validate --deny W860 promotes to gate failure | ✓ PASS |
| decompositionKind appears in safety-case report | ✓ PASS |
| template Requirement includes decompositionKind | ✓ PASS |

---

### TC-TRS-SCRIPT-001 — Verify Rhai extension scripts load from the configured scripts dir, discover recursively, and support library-module import reuse.

**Verifies:** REQ-TRS-SCRIPT-001  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| a model with no scripts directory runs normally | ✓ PASS |

---

### TC-TRS-SCRIPT-002 — Verify extension scripts run sandboxed and resource-limited: runaway aborted, eval and filesystem escape refused, parse errors named without crashing siblings, output deterministic.

**Verifies:** REQ-TRS-SCRIPT-002  
**Result:** ✓ PASS (12 passed, 0 failed)

| Scenario | Result |
|---|---|
| deterministic output | ✓ PASS |

---

### TC-TRS-SCRIPT-003 — Verify the read-only model API: element iteration, getters, find by id and qname, e.field, custom_fields, computed reverse indices, and print/eprint output.

**Verifies:** REQ-TRS-SCRIPT-003  
**Result:** ✓ PASS (16 passed, 0 failed)

| Scenario | Result |
|---|---|
| stdout and stderr output | ✓ PASS |

---

### TC-TRS-SCRIPT-004 — Verify the two registration shapes (register_command and register_check), a pure library file, and the duplicate-name load error.

**Verifies:** REQ-TRS-SCRIPT-004  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| duplicate name is a load error | ✓ PASS |

---

### TC-TRS-SCRIPT-005 — Verify scripts list and scripts run: enumeration with kind/description/source, running a command (text and --json), unknown-name failure, and check-not-runnable-as-command.

**Verifies:** REQ-TRS-SCRIPT-005  
**Result:** ✓ PASS (18 passed, 0 failed)

| Scenario | Result |
|---|---|
| no scripts directory reports none and exits 0 | ✓ PASS |

---

### TC-TRS-SCRIPT-006 — Verify scripts validate: namespaced <check>/<code> findings, the 0/1/2 exit contract with gate flags, and independence from the built-in validate.

**Verifies:** REQ-TRS-SCRIPT-006  
**Result:** ✓ PASS (11 passed, 0 failed)

| Scenario | Result |
|---|---|
| built-in validate is unaffected by check scripts | ✓ PASS |

---

### TC-TRS-SEC-001 — Verify safety↔security co-engineering: hazardRef, E844, W030, and the co-analysis view.

**Verifies:** REQ-TRS-SEC-001  
**Result:** ✓ PASS (14 passed, 0 failed)

| Scenario | Result |
|---|---|
| linked safety damage scenario: no errors, W030 only on the unlinked one | ✓ PASS |
| co-analysis --json carries goals and unlinkedSafetyDamage | ✓ PASS |

---

### TC-TRS-SEC-002 — Verify ISO/SAE 21434 risk determination: risk model, E845, W031, W032, and the cyber-risk view.

**Verifies:** REQ-TRS-SEC-002  
**Result:** ✓ PASS (17 passed, 0 failed)

| Scenario | Result |
|---|---|
| untreated high-risk threat: W031 on it only; W032 on under-CAL goal; no errors | ✓ PASS |
| cyber-risk --json carries the per-threat risk fields | ✓ PASS |

---

### TC-TRS-SEC-003 — Verify ISO/SAE 21434 attack tree types: AttackTree/AttackTreeGate/AttackStep, weakest-link roll-up, E915–E921, W035.

**Verifies:** REQ-TRS-SEC-003  
**Result:** ✓ PASS (21 passed, 0 failed)

| Scenario | Result |
|---|---|
| worked example rolls up to medium; W035 vs declared high; no errors | ✓ PASS |
| matched declared feasibility clears W035 | ✓ PASS |
| types command lists the new attack-tree types | ✓ PASS |

---

### TC-TRS-SEC-004 — AOU.appliesTo accepts CybersecurityGoal and enforces E859 for wrong types

**Verifies:**   
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| AOU.appliesTo wrong type triggers E859 | ✓ PASS |

---

### TC-TRS-SEC-005 — ConfirmationMeasure.confirms accepts CybersecurityGoal and enforces E860 for wrong types

**Verifies:**   
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| CM.confirms wrong element type triggers E860 | ✓ PASS |

---

### TC-TRS-SEC-006 — derivedFromCybersecurityGoal field resolves correctly; alias derivedFromSecurityGoal also works

**Verifies:**   
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| legacy derivedFromSecurityGoal alias also resolves | ✓ PASS |

---

### TC-TRS-SEC-007 — W039 fires for CAL3 CybersecurityGoal with no I2 confirmation measure

**Verifies:**   
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| CAL3 CSG with I3 CM also clears W039 | ✓ PASS |

---

### TC-TRS-SEC-008 — TestCase.securityTestMethod validates recognised values; W809 for unknown

**Verifies:**   
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| invalid securityTestMethod triggers W809 | ✓ PASS |

---

### TC-TRS-SM-001 — Verify the canonical SysMLv2 transition schema: nested (implicit source) and top-level (explicit source) placements yield the same edges; accept string and {payload} forms are equivalent; both validate clean.

**Verifies:** REQ-TRS-SM-001  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| nested per-substate transitions are canonical | ✓ PASS |
| top-level transitions with explicit source | ✓ PASS |

---

### TC-TRS-SM-002 — Verify W075 — legacy from/to/trigger transition keys raise the deprecation warning while still contributing the correct edge; canonical keys are silent; draft-suppressed; gateable with --deny W075.

**Verifies:** REQ-TRS-SM-002  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| W075 — legacy from/to/trigger keys | ✓ PASS |
| no W075 — canonical source/target/accept | ✓ PASS |
| W075 draft-suppressed | ✓ PASS |
| validate --deny W075 promotes to gate failure | ✓ PASS |

---

### TC-TRS-SM-003 — Verify flat state-machine completeness W070–W074: dead/trap/non-determinism/missing-initial/multiple-initial each fire on a crafted defect; a well-formed machine and a parallel/composite machine are clean; draft-suppressed; gateable.

**Verifies:** REQ-TRS-SM-003  
**Result:** ✓ PASS (18 passed, 0 failed)

| Scenario | Result |
|---|---|
| W070 — dead state | ✓ PASS |
| W071 — trap state | ✓ PASS |
| W072 — non-determinism | ✓ PASS |
| W073 — missing initial state | ✓ PASS |
| W074 — multiple initial states | ✓ PASS |
| well-formed machine is clean | ✓ PASS |
| parallel/composite machines out of scope | ✓ PASS |
| W073 draft-suppressed | ✓ PASS |
| validate --deny W073 promotes to gate failure | ✓ PASS |

---

### TC-TRS-SM-004 — Verify parallel state machines: per-region completeness (region-named W073), cross-region transition W077, parallel arity W078; a well-formed parallel machine is clean; draft-suppressed; gateable.

**Verifies:** REQ-TRS-SM-004,REQ-TRS-SM-005  
**Result:** ✓ PASS (13 passed, 0 failed)

| Scenario | Result |
|---|---|
| well-formed parallel machine is clean | ✓ PASS |
| W073 — region has no initial | ✓ PASS |
| W077 — cross-region transition | ✓ PASS |
| W078 — single-region parallel state | ✓ PASS |
| W078 draft-suppressed | ✓ PASS |
| validate --deny W078 promotes to gate failure | ✓ PASS |

---

### TC-TRS-SM-005 — Verify composite/hierarchical state machines: recursion into inner regions (region-named W073), composite-as-node top-level checks, and W076 for unresolved transition endpoints; clean composite is silent; draft-suppressed; gateable.

**Verifies:** REQ-TRS-SM-006,REQ-TRS-SM-007  
**Result:** ✓ PASS (11 passed, 0 failed)

| Scenario | Result |
|---|---|
| well-formed composite machine is clean | ✓ PASS |
| W073 — inner region has no initial | ✓ PASS |
| W076 — unresolved transition endpoint | ✓ PASS |
| W076 draft-suppressed | ✓ PASS |
| validate --deny W076 promotes to gate failure | ✓ PASS |

---

### TC-TRS-SM-006 — Verify W079 (unresolved entry/do/exit/effect behavior reference) fires on a dangling effect and is silent when resolvable; decision transitions (guarded same-source branches) do not raise W072; draft-suppressed; gateable.

**Verifies:** REQ-TRS-SM-008  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| W079 — dangling transition effect | ✓ PASS |
| no W079 — resolvable entry action and effect | ✓ PASS |
| decision transition does not raise W072 | ✓ PASS |
| W079 draft-suppressed | ✓ PASS |
| validate --deny W079 promotes to gate failure | ✓ PASS |

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

### TC-TRS-TAG-002 — Verify list --tag multi-tag AND filtering: repeated --tag narrows to elements carrying all tags.

**Verifies:** REQ-TRS-TAG-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| no --tag lists all (filter inactive) | ✓ PASS |

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
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| W306 message names the sub-conditions | ✓ PASS |
| a fully-integrated high-integrity requirement produces no W306 | ✓ PASS |
| W306 is gateable with --deny | ✓ PASS |

---

### TC-TRS-TYPE-001 — Verify ConstraintDef and Constraint are recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-001  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-002 — Verify CalculationDef and Calculation are recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-002  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-003 — Verify ConcernDef and Concern are recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-003  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-004 — Verify EventOccurrenceDef and EventOccurrence are recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-004  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-005 — Verify CaseDef is recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-005  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-006 — Verify AnalysisCase is recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-006  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-007 — Verify VerificationCase is recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-007  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-008 — Verify UseCase is recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-008  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-009 — Verify AllocationDef is recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-009  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-010 — Verify SuccessionDef is recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-010  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-011 — Verify RenderingDef is recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-011  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-012 — Verify State and ExhibitState are recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-012  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-013 — Verify Metadata is recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-013  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-014 — Verify BindingConnector is recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-014  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-015 — Verify LibraryPackage and Namespace are recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-015  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-016 — Verify Dependency is recognised and validated without E005.

**Verifies:** REQ-TRS-TYPE-016  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-017 — Asset type validates with E861/E862/E863/E864/W810 rules

**Verifies:**   
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| list Asset shows asset elements | ✓ PASS |

---

### TC-TRS-TYPE-018 — Verify the native ReviewRecord element: E700–E705 structural rules, W700 (closed review with open item), W704 (uncovered requirement), and the reviews / review / reviews --coverage commands plus template ReviewRecord.

**Verifies:** REQ-TRS-TYPE-018  
**Result:** ✓ PASS (19 passed, 0 failed)

| Scenario | Result |
|---|---|
| well-formed ReviewRecord is clean | ✓ PASS |
| E700 — missing required fields | ✓ PASS |
| E701 — bad id pattern | ✓ PASS |
| E702 — bad status | ✓ PASS |
| E703 — bad reviewType | ✓ PASS |
| E704 — unresolved reviews entry | ✓ PASS |
| E705 — bad item disposition | ✓ PASS |
| W700 — closed review with open item | ✓ PASS |
| W704 — uncovered requirement | ✓ PASS |
| reviews lists the record | ✓ PASS |
| reviews --coverage shows uncovered requirement | ✓ PASS |
| review shows detail | ✓ PASS |
| template ReviewRecord skeleton | ✓ PASS |

---

### TC-TRS-TYPE-019 — Verify the native TradeStudy element: E869–E877 structural rules, W061–W064 advisories, normalised/weighted/ranked scoring, and the trade-study list/detail commands plus template TradeStudy.

**Verifies:** REQ-TRS-TYPE-019  
**Result:** ✓ PASS (27 passed, 0 failed)

| Scenario | Result |
|---|---|
| well-formed TradeStudy is clean | ✓ PASS |
| E869 — missing scores | ✓ PASS |
| E870 — bad id | ✓ PASS |
| E871 — criterion missing field | ✓ PASS |
| E872 — weight out of range | ✓ PASS |
| E873 — bad direction | ✓ PASS |
| E874 — empty alternatives | ✓ PASS |
| E875 — alternative missing name | ✓ PASS |
| E876 — unknown alternative | ✓ PASS |
| E877 — non-numeric score | ✓ PASS |
| W061 — complete without decision | ✓ PASS |
| W063 — incomplete matrix | ✓ PASS |
| trade-study detail ranked table | ✓ PASS |
| template TradeStudy skeleton | ✓ PASS |

---

### TC-TRS-TYPE-020 — Verify IEC 62443 Zone/Conduit: E950/E951/E954/E956 structural rules, W950 SL gap, W953 isolated zone; a clean model is silent; the zones / conduits / zones --coverage commands and template Zone/Conduit.

**Verifies:** REQ-TRS-TYPE-020  
**Result:** ✓ PASS (19 passed, 0 failed)

| Scenario | Result |
|---|---|
| well-formed zones/conduit are clean | ✓ PASS |
| E950 — zone missing targetSL | ✓ PASS |
| E951 — bad zone id | ✓ PASS |
| E954 — conduit endpoint not a zone | ✓ PASS |
| E956 — part inZone not a zone | ✓ PASS |
| W950 — SL gap | ✓ PASS |
| W953 — approved zone with no conduit | ✓ PASS |
| zones lists the zones | ✓ PASS |
| conduits lists the conduit | ✓ PASS |
| zones --coverage shows the control | ✓ PASS |
| template Zone | ✓ PASS |
| template Conduit | ✓ PASS |

---

### TC-TRS-TYPE-021 — Verify multi-repository composition: E510 circular, E511 missing path, E512 dangling cross-repo ref, E513 unknown alias, E514 unknown qname, E515 duplicate stable id, W510 unpinned repo; a valid composition resolves a cross-repo verifies cleanly; the repos list command.

**Verifies:** REQ-TRS-TYPE-021  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| repos list command | ✓ PASS |

---

### TC-TRS-TYPE-022 — Verify peer-repository ref-drift detection: W511 fires when a peer HEAD drifts from its pinned ref, --deny W511 gates CI (exit 2), an in-sync peer is silent, and undeterminable drift (no git) does not emit W511.

**Verifies:** REQ-TRS-TYPE-022  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| undeterminable drift does not warn | ✓ PASS |

---

### TC-TRS-TYPE-023 — Verify gitlink/ref mismatch detection: W512 fires when a submodule peer's ref resolves to a different commit than the parent's recorded gitlink, --deny W512 gates CI, a ref matching the gitlink is silent, and a non-submodule sibling peer never emits W512.

**Verifies:** REQ-TRS-TYPE-023  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| non-submodule peer does not emit W512 | ✓ PASS |

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

### TC-TRS-XREF-006 — Verify root-package-name hint: unresolved ref prefixed with the root name (stripped form resolves) gets a hint; correct ref no finding; non-matching ref no hint.

**Verifies:** REQ-TRS-XREF-006  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| an unresolved reference not starting with the root name gets no hint | ✓ PASS |

---
