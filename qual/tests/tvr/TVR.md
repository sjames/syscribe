# Tool Validation Report

**Tool:** syscribe CLI validator  
**Version:** unknown  
**Standard:** ISO 26262:2018 Part 8 §11 (TCL2), IEC 61508:2010 Part 3 Annex D  
**Date:** 2026-06-12  
**TRS:** `qual/Requirements/`  **Test cases:** `qual/TestCases/`

---

## 1. Summary

| Metric | Value |
|---|---|
| Total test cases | 202 |
| Passed | 202 |
| Failed | 0 |
| Overall verdict | **PASS** |

---

## 2. Results

### TC-TRS-ALLOC-001 — TC-TRS-ALLOC-001

**Verifies:** REQ-TRS-ALLOC-001  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| the same edge in both forms raises W503 | ✓ PASS |

---

### TC-TRS-AW-001 — TC-TRS-AW-001

**Verifies:** REQ-TRS-AW-001  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| dry-run validates but writes nothing | ✓ PASS |

---

### TC-TRS-AW-002 — TC-TRS-AW-002

**Verifies:** REQ-TRS-AW-002  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| an unresolved element exits non-zero | ✓ PASS |

---

### TC-TRS-CFLD-001 — TC-TRS-CFLD-001

**Verifies:** REQ-TRS-CFLD-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| a nested-map custom field raises W041 | ✓ PASS |

---

### TC-TRS-CFLD-002 — TC-TRS-CFLD-002

**Verifies:** REQ-TRS-CFLD-002  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| unparseable predicate exits non-zero | ✓ PASS |

---

### TC-TRS-CFLD-003 — TC-TRS-CFLD-003

**Verifies:** REQ-TRS-CFLD-003  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| show omits the section when absent | ✓ PASS |

---

### TC-TRS-CLI-001 — TC-TRS-CLI-001

**Verifies:** REQ-TRS-CLI-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| --model long form | ✓ PASS |

---

### TC-TRS-CLI-002 — TC-TRS-CLI-002

**Verifies:** REQ-TRS-CLI-002  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| non-existent path exits non-zero | ✓ PASS |

---

### TC-TRS-CLI-003 — TC-TRS-CLI-003

**Verifies:** REQ-TRS-CLI-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| --agent-instructions prints prompt and exits 0 | ✓ PASS |

---

### TC-TRS-CLI-004 — TC-TRS-CLI-004

**Verifies:** REQ-TRS-CLI-004  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| no marker and no model falls back to default and reports the miss | ✓ PASS |

---

### TC-TRS-CLI-005 — TC-TRS-CLI-005

**Verifies:** REQ-TRS-CLI-005  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| help <unknown> exits non-zero | ✓ PASS |

---

### TC-TRS-CLI-006 — TC-TRS-CLI-006

**Verifies:** REQ-TRS-CLI-006  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| works without a model directory | ✓ PASS |

---

### TC-TRS-CONF-001 — TC-TRS-CONF-001

**Verifies:** REQ-TRS-CONF-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E200 | ✓ PASS |
| trigger E201 | ✓ PASS |
| trigger E209 | ✓ PASS |

---

### TC-TRS-CONF-002 — TC-TRS-CONF-002

**Verifies:** REQ-TRS-CONF-002  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| template emits features: map, not selections: | ✓ PASS |
| legacy selections: under a feature model warns | ✓ PASS |
| features: map configuration does not warn | ✓ PASS |
| show displays parsed feature selections | ✓ PASS |

---

### TC-TRS-DIAG-001 — TC-TRS-DIAG-001

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

### TC-TRS-DISC-001 — TC-TRS-DISC-001

**Verifies:** REQ-TRS-DISC-001  
**Result:** ✓ PASS (15 passed, 0 failed)

| Scenario | Result |
|---|---|
| dormant on a model with no feature model | ✓ PASS |

---

### TC-TRS-DISC-002 — TC-TRS-DISC-002

**Verifies:** REQ-TRS-DISC-002  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| unknown feature errors | ✓ PASS |

---

### TC-TRS-DISC-003 — TC-TRS-DISC-003

**Verifies:** REQ-TRS-DISC-003  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| default matrix still shows Requirement × Configuration view | ✓ PASS |

---

### TC-TRS-DISC-004 — TC-TRS-DISC-004

**Verifies:** REQ-TRS-DISC-004  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| unknown feature errors | ✓ PASS |

---

### TC-TRS-DISC-005 — TC-TRS-DISC-005

**Verifies:** REQ-TRS-DISC-005  
**Result:** ✓ PASS (9 passed, 0 failed)

| Scenario | Result |
|---|---|
| --config is required and must resolve | ✓ PASS |

---

### TC-TRS-DISC-006 — TC-TRS-DISC-006

**Verifies:** REQ-TRS-DISC-006  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| base validate never emits W024 | ✓ PASS |
| W024 is gateable with --deny | ✓ PASS |

---

### TC-TRS-DISC-007 — TC-TRS-DISC-007

**Verifies:** REQ-TRS-DISC-007  
**Result:** ✓ PASS (14 passed, 0 failed)

| Scenario | Result |
|---|---|
| matrix --json carries a coverage object | ✓ PASS |

---

### TC-TRS-ELEM-001 — TC-TRS-ELEM-001

**Verifies:** REQ-TRS-ELEM-001  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| all defined element types are recognised without E005 | ✓ PASS |

---

### TC-TRS-ELEM-002 — TC-TRS-ELEM-002

**Verifies:** REQ-TRS-ELEM-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| completely unknown type value produces E005 | ✓ PASS |
| wrong-case type value produces E005 | ✓ PASS |

---

### TC-TRS-ELEM-003 — TC-TRS-ELEM-003

**Verifies:** REQ-TRS-ELEM-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| PartDef with no supertype: loads without E004 | ✓ PASS |

---

### TC-TRS-EXTREF-001 — TC-TRS-EXTREF-001

**Verifies:** REQ-TRS-EXTREF-001  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| single-string and list-valued extRef parse without error | ✓ PASS |
| the same extRef on two elements produces W028 | ✓ PASS |
| W028 is gateable with --deny | ✓ PASS |

---

### TC-TRS-EXTREF-002 — TC-TRS-EXTREF-002

**Verifies:** REQ-TRS-EXTREF-002  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| spec fields lists extRef | ✓ PASS |

---

### TC-TRS-FM-001 — TC-TRS-FM-001

**Verifies:** REQ-TRS-FM-001  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| --json emits structured findings | ✓ PASS |

---

### TC-TRS-FM-002 — TC-TRS-FM-002

**Verifies:** REQ-TRS-FM-002  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| clean feature model emits none of them | ✓ PASS |

---

### TC-TRS-FM-003 — TC-TRS-FM-003

**Verifies:** REQ-TRS-FM-003  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| clean feature model emits none of them | ✓ PASS |

---

### TC-TRS-FM-004 — TC-TRS-FM-004

**Verifies:** REQ-TRS-FM-004  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| legacy groupKind: mandatory child still treated as forced | ✓ PASS |

---

### TC-TRS-FMA-001 — TC-TRS-FMA-001

**Verifies:** REQ-TRS-FMA-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| contradictory model is void | ✓ PASS |

---

### TC-TRS-FMA-002 — TC-TRS-FMA-002

**Verifies:** REQ-TRS-FMA-002  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| dormant with no feature model | ✓ PASS |

---

### TC-TRS-FMA-003 — TC-TRS-FMA-003

**Verifies:** REQ-TRS-FMA-003  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| void dominates (no dead spam) | ✓ PASS |

---

### TC-TRS-FMA-004 — TC-TRS-FMA-004

**Verifies:** REQ-TRS-FMA-004  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| requires violation stays E219, not E225 | ✓ PASS |

---

### TC-TRS-FMA-005 — TC-TRS-FMA-005

**Verifies:** REQ-TRS-FMA-005  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| soundness: removing the excludes un-voids the model | ✓ PASS |

---

### TC-TRS-FMA-006 — TC-TRS-FMA-006

**Verifies:** REQ-TRS-FMA-006  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| scope statement: Boolean layer only | ✓ PASS |

---

### TC-TRS-FMA-007 — TC-TRS-FMA-007

**Verifies:** REQ-TRS-FMA-007  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| explanation is a minimal conflict set (excludes unrelated features) | ✓ PASS |

---

### TC-TRS-FMA-008 — TC-TRS-FMA-008

**Verifies:** REQ-TRS-FMA-008  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| dormant with no feature model | ✓ PASS |

---

### TC-TRS-FMA-009 — TC-TRS-FMA-009

**Verifies:** REQ-TRS-FMA-009  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| enumeration lists each valid configuration | ✓ PASS |

---

### TC-TRS-FMA-010 — TC-TRS-FMA-010

**Verifies:** REQ-TRS-FMA-010  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| a correction set names the requires or excludes constraint | ✓ PASS |

---

### TC-TRS-FMA-011 — TC-TRS-FMA-011

**Verifies:** REQ-TRS-FMA-011  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| no proof files without --prove | ✓ PASS |

---

### TC-TRS-FMEA-001 — TC-TRS-FMEA-001

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

### TC-TRS-FTA-001 — TC-TRS-FTA-001

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

### TC-TRS-ID-001 — TC-TRS-ID-001

**Verifies:** REQ-TRS-ID-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid REQ-* id patterns are accepted | ✓ PASS |
| invalid REQ-* id pattern produces E006 | ✓ PASS |

---

### TC-TRS-ID-002 — TC-TRS-ID-002

**Verifies:** REQ-TRS-ID-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid TC-* id pattern is accepted | ✓ PASS |
| invalid TC-* id pattern produces E006 | ✓ PASS |

---

### TC-TRS-ID-003 — TC-TRS-ID-003

**Verifies:** REQ-TRS-ID-003  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid ADR-* id is accepted | ✓ PASS |
| ADR id not matching pattern produces E300 | ✓ PASS |
| ADR missing id produces E301 | ✓ PASS |

---

### TC-TRS-ID-004 — TC-TRS-ID-004

**Verifies:** REQ-TRS-ID-004  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| two elements with the same id produce E101 | ✓ PASS |
| unique ids produce no E101 | ✓ PASS |

---

### TC-TRS-ID-005 — TC-TRS-ID-005

**Verifies:** REQ-TRS-ID-005  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| max_digits=4 tightens the cap: a 5-digit id trips E023 | ✓ PASS |

---

### TC-TRS-ID-006 — TC-TRS-ID-006

**Verifies:** REQ-TRS-ID-006  
**Result:** ✓ PASS (21 passed, 0 failed)

| Scenario | Result |
|---|---|
| a stable-id-shaped reference resolving to nothing raises E209 | ✓ PASS |

---

### TC-TRS-IMPL-001 — TC-TRS-IMPL-001

**Verifies:** REQ-TRS-IMPL-001  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| W023 is gateable with --deny | ✓ PASS |

---

### TC-TRS-IMPL-002 — TC-TRS-IMPL-002

**Verifies:** REQ-TRS-IMPL-002  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| spec fields lists implementedBy | ✓ PASS |

---

### TC-TRS-LIB-001 — TC-TRS-LIB-001

**Verifies:** REQ-TRS-LIB-001  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| the check covers Base and multiple contexts (supertype, typedBy, returnType, parameter type) | ✓ PASS |

---

### TC-TRS-LIB-002 — TC-TRS-LIB-002

**Verifies:** REQ-TRS-LIB-002  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| non-SI domain units in unit: are permissive (no finding) | ✓ PASS |

---

### TC-TRS-LIB-003 — TC-TRS-LIB-003

**Verifies:** REQ-TRS-LIB-003  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| a non-quantity typedBy makes the check lenient (no W044) | ✓ PASS |

---

### TC-TRS-LINK-001 — TC-TRS-LINK-001

**Verifies:** REQ-TRS-LINK-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| no [links] table yields no URLs | ✓ PASS |

---

### TC-TRS-LINK-002 — TC-TRS-LINK-002

**Verifies:** REQ-TRS-LINK-002  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| no [links] table leaves shapes unwrapped by hosted links | ✓ PASS |

---

### TC-TRS-LINK-003 — TC-TRS-LINK-003

**Verifies:** REQ-TRS-LINK-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| no [links] table emits no hosted click directive | ✓ PASS |

---

### TC-TRS-LINK-004 — TC-TRS-LINK-004

**Verifies:** REQ-TRS-LINK-004  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| no [links] table leaves the report unlinked | ✓ PASS |

---

### TC-TRS-LINK-005 — TC-TRS-LINK-005

**Verifies:** REQ-TRS-LINK-005  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| live detail panel renders/omits the source-link icon per [links] | ✓ PASS |

---

### TC-TRS-META-001 — TC-TRS-META-001

**Verifies:** REQ-TRS-META-001  
**Result:** ✓ PASS (9 passed, 0 failed)

| Scenario | Result |
|---|---|
| list --metadata filters to elements applying the stereotype | ✓ PASS |

---

### TC-TRS-META-002 — TC-TRS-META-002

**Verifies:** REQ-TRS-META-002  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| an element with no application renders no spurious stereotype banner | ✓ PASS |

---

### TC-TRS-MG-001 — TC-TRS-MG-001

**Verifies:** REQ-TRS-MG-001  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| the magicgrid profile promotes W307 to a gate failure | ✓ PASS |

---

### TC-TRS-MG-002 — TC-TRS-MG-002

**Verifies:** REQ-TRS-MG-002  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| actor participation is indexed | ✓ PASS |

---

### TC-TRS-MG-003 — TC-TRS-MG-003

**Verifies:** REQ-TRS-MG-003  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| mg_cell is inert without the magicgrid profile | ✓ PASS |

---

### TC-TRS-MG-004 — TC-TRS-MG-004

**Verifies:** REQ-TRS-MG-004  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| mg_moe fields are inert without the gate | ✓ PASS |

---

### TC-TRS-MG-005 — TC-TRS-MG-005

**Verifies:** REQ-TRS-MG-005  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| mg_layer is inert without the gate | ✓ PASS |

---

### TC-TRS-MG-006 — TC-TRS-MG-006

**Verifies:** REQ-TRS-MG-006  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| the matrix emits JSON | ✓ PASS |

---

### TC-TRS-MG-007 — TC-TRS-MG-007

**Verifies:** REQ-TRS-MG-007  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| the trade study emits JSON | ✓ PASS |

---

### TC-TRS-MG-008 — TC-TRS-MG-008

**Verifies:** REQ-TRS-MG-008  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| mg_mop fields are inert without the gate | ✓ PASS |

---

### TC-TRS-MG-009 — TC-TRS-MG-009

**Verifies:** REQ-TRS-MG-009  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| mg_soi is inert without the gate | ✓ PASS |

---

### TC-TRS-MG-010 — TC-TRS-MG-010

**Verifies:** REQ-TRS-MG-010  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| a behavioral def with no refines raises no W307 | ✓ PASS |

---

### TC-TRS-MG-011 — TC-TRS-MG-011

**Verifies:** REQ-TRS-MG-011  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| mg_variant on a non-Configuration raises MG070 under the gate | ✓ PASS |

---

### TC-TRS-MG-012 — TC-TRS-MG-012

**Verifies:** REQ-TRS-MG-012  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| a single segment match still resolves | ✓ PASS |

---

### TC-TRS-MG-013 — TC-TRS-MG-013

**Verifies:** REQ-TRS-MG-013  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| the audit emits JSON | ✓ PASS |

---

### TC-TRS-MG-014 — TC-TRS-MG-014

**Verifies:** REQ-TRS-MG-014  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| the completeness checks are inert without the gate | ✓ PASS |

---

### TC-TRS-MG-015 — TC-TRS-MG-015

**Verifies:** REQ-TRS-MG-015  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| the per-cell element detail is retained | ✓ PASS |

---

### TC-TRS-MG-016 — TC-TRS-MG-016

**Verifies:** REQ-TRS-MG-016  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| a long cell label is word-wrapped, not truncated | ✓ PASS |

---

### TC-TRS-MG-017 — TC-TRS-MG-017

**Verifies:** REQ-TRS-MG-017  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| the winning configuration is marked | ✓ PASS |

---

### TC-TRS-MOVE-001 — TC-TRS-MOVE-001

**Verifies:** REQ-TRS-MOVE-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| reject move into own subtree | ✓ PASS |

---

### TC-TRS-MOVE-002 — TC-TRS-MOVE-002

**Verifies:** REQ-TRS-MOVE-002  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| descendant endpoint follows package move | ✓ PASS |

---

### TC-TRS-MOVE-003 — TC-TRS-MOVE-003

**Verifies:** REQ-TRS-MOVE-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| --dry-run reports without writing | ✓ PASS |

---

### TC-TRS-MOVE-004 — TC-TRS-MOVE-004

**Verifies:** REQ-TRS-MOVE-004  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| id-based reference is not rewritten and still resolves | ✓ PASS |

---

### TC-TRS-NAME-001 — TC-TRS-NAME-001

**Verifies:** REQ-TRS-NAME-001  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| a basic directory name is not flagged | ✓ PASS |

---

### TC-TRS-NAME-002 — TC-TRS-NAME-002

**Verifies:** REQ-TRS-NAME-002  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| a FeatureDef with a FEAT id and a name (no title) is clean of both | ✓ PASS |

---

### TC-TRS-OUT-001 — TC-TRS-OUT-001

**Verifies:** REQ-TRS-OUT-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| report is written to stdout in Markdown format | ✓ PASS |

---

### TC-TRS-OUT-002 — TC-TRS-OUT-002

**Verifies:** REQ-TRS-OUT-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| findings table has Code, File, and Message columns | ✓ PASS |

---

### TC-TRS-OUT-003 — TC-TRS-OUT-003

**Verifies:** REQ-TRS-OUT-003  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| summary section includes error and warning counts | ✓ PASS |

---

### TC-TRS-OUT-004 — TC-TRS-OUT-004

**Verifies:** REQ-TRS-OUT-004  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| model with errors exits non-zero | ✓ PASS |
| model with errors and warnings still exits non-zero | ✓ PASS |

---

### TC-TRS-OUT-005 — TC-TRS-OUT-005

**Verifies:** REQ-TRS-OUT-005  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| model with only warnings exits 0 | ✓ PASS |
| clean model exits 0 | ✓ PASS |

---

### TC-TRS-OUT-006 — TC-TRS-OUT-006

**Verifies:** REQ-TRS-OUT-006  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| errors dominate gating flags (exit 1) | ✓ PASS |

---

### TC-TRS-OUT-007 — TC-TRS-OUT-007

**Verifies:** REQ-TRS-OUT-007  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| ndjson emits header then elements | ✓ PASS |

---

### TC-TRS-OUT-008 — TC-TRS-OUT-008

**Verifies:** REQ-TRS-OUT-008  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| junit results supported via --results | ✓ PASS |

---

### TC-TRS-OUT-009 — TC-TRS-OUT-009

**Verifies:** REQ-TRS-OUT-009  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| no sidecar → no ▣ glyph and no verdict annotations | ✓ PASS |

---

### TC-TRS-OUT-010 — TC-TRS-OUT-010

**Verifies:** REQ-TRS-OUT-010  
**Result:** ✓ PASS (14 passed, 0 failed)

| Scenario | Result |
|---|---|
| unknown root exits non-zero | ✓ PASS |

---

### TC-TRS-OUT-011 — TC-TRS-OUT-011

**Verifies:** REQ-TRS-OUT-011  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| --min-levels 1 passes when all have >=1 level | ✓ PASS |

---

### TC-TRS-OUT-012 — TC-TRS-OUT-012

**Verifies:** REQ-TRS-OUT-012  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| --profile nonexistent is an error (exit 1) | ✓ PASS |

---

### TC-TRS-OUT-013 — TC-TRS-OUT-013

**Verifies:** REQ-TRS-OUT-013  
**Result:** ✓ PASS (21 passed, 0 failed)

---

### TC-TRS-PARAM-001 — TC-TRS-PARAM-001

**Verifies:** REQ-TRS-PARAM-001  
**Result:** ✓ PASS (12 passed, 0 failed)

| Scenario | Result |
|---|---|
| parameter binding violations emit E203–E206, E222, W017 | ✓ PASS |
| valid bindings emit no parameter findings | ✓ PASS |

---

### TC-TRS-PARAM-002 — TC-TRS-PARAM-002

**Verifies:** REQ-TRS-PARAM-002  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| W025 is gateable via --deny | ✓ PASS |

---

### TC-TRS-PARAM-003 — TC-TRS-PARAM-003

**Verifies:** REQ-TRS-PARAM-003  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| binding 99 against range 1..=8 is out of range | ✓ PASS |
| feature-check enforces parameter range (E205) | ✓ PASS |

---

### TC-TRS-PARAM-004 — TC-TRS-PARAM-004

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

### TC-TRS-PARSE-001 — TC-TRS-PARSE-001

**Verifies:** REQ-TRS-PARSE-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid directory is accepted | ✓ PASS |
| empty directory produces zero elements | ✓ PASS |

---

### TC-TRS-PARSE-002 — TC-TRS-PARSE-002

**Verifies:** REQ-TRS-PARSE-002  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| elements in nested directories are discovered | ✓ PASS |
| non-.md files are ignored | ✓ PASS |

---

### TC-TRS-PARSE-003 — TC-TRS-PARSE-003

**Verifies:** REQ-TRS-PARSE-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| target/ directory is excluded from discovery | ✓ PASS |

---

### TC-TRS-PARSE-004 — TC-TRS-PARSE-004

**Verifies:** REQ-TRS-PARSE-004  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| .sysmlignore suppresses matching files | ✓ PASS |
| absence of .sysmlignore causes no error | ✓ PASS |

---

### TC-TRS-PARSE-005 — TC-TRS-PARSE-005

**Verifies:** REQ-TRS-PARSE-005  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| _index.md is treated as package declaration | ✓ PASS |

---

### TC-TRS-PARSE-006 — TC-TRS-PARSE-006

**Verifies:** REQ-TRS-PARSE-006  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| malformed YAML frontmatter produces a warning, not a fatal error | ✓ PASS |

---

### TC-TRS-PARSE-007 — TC-TRS-PARSE-007

**Verifies:** REQ-TRS-PARSE-007  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| file with --- on first line is parsed correctly | ✓ PASS |
| file with blank first line produces E001 (missing frontmatter) | ✓ PASS |

---

### TC-TRS-PARSE-008 — TC-TRS-PARSE-008

**Verifies:** REQ-TRS-PARSE-008  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid YAML frontmatter produces no E002 | ✓ PASS |
| invalid YAML frontmatter produces E002 | ✓ PASS |

---

### TC-TRS-PARSE-009 — TC-TRS-PARSE-009

**Verifies:** REQ-TRS-PARSE-009  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| file with no type: field is skipped with a warning | ✓ PASS |
| file with type: present is processed normally | ✓ PASS |

---

### TC-TRS-PLAN-001 — TC-TRS-PLAN-001

**Verifies:** REQ-TRS-PLAN-001  
**Result:** ✓ PASS (11 passed, 0 failed)

| Scenario | Result |
|---|---|
| template TestPlan emits a valid skeleton | ✓ PASS |

---

### TC-TRS-PLAN-002 — TC-TRS-PLAN-002

**Verifies:** REQ-TRS-PLAN-002  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| two plans with identical (configurations, scope) raise W616 | ✓ PASS |

---

### TC-TRS-PLAN-003 — TC-TRS-PLAN-003

**Verifies:** REQ-TRS-PLAN-003  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| an explicitly named draft TestCase raises W613 | ✓ PASS |

---

### TC-TRS-PLAN-004 — TC-TRS-PLAN-004

**Verifies:** REQ-TRS-PLAN-004  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| demonstrating a parent goal whose leaf is tested does not raise W614 (goal-closure) | ✓ PASS |

---

### TC-TRS-PLAN-005 — TC-TRS-PLAN-005

**Verifies:** REQ-TRS-PLAN-005  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| testplan on an unknown plan exits non-zero | ✓ PASS |

---

### TC-TRS-PLAN-006 — TC-TRS-PLAN-006

**Verifies:** REQ-TRS-PLAN-006  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| audit --plan scopes the verdict to the plan (no escaping-ref artifacts) | ✓ PASS |

---

### TC-TRS-PROJ-001 — TC-TRS-PROJ-001

**Verifies:** REQ-TRS-PROJ-001  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| unresolved configuration errors | ✓ PASS |

---

### TC-TRS-PROJ-002 — TC-TRS-PROJ-002

**Verifies:** REQ-TRS-PROJ-002  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| whole-model validate reports no such coverage gap | ✓ PASS |
| whole-model validate | ✓ PASS |

---

### TC-TRS-PROJ-003 — TC-TRS-PROJ-003

**Verifies:** REQ-TRS-PROJ-003  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| references between active elements do not escape | ✓ PASS |

---

### TC-TRS-PROJ-004 — TC-TRS-PROJ-004

**Verifies:** REQ-TRS-PROJ-004  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| traceability edge is a warning (W020) | ✓ PASS |

---

### TC-TRS-PROJ-005 — TC-TRS-PROJ-005

**Verifies:** REQ-TRS-PROJ-005  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| variant diff lists the symmetric difference | ✓ PASS |

---

### TC-TRS-PROJ-006 — TC-TRS-PROJ-006

**Verifies:** REQ-TRS-PROJ-006  
**Result:** ✓ PASS (12 passed, 0 failed)

| Scenario | Result |
|---|---|
| verification-depth projects out a gated SIL-3 requirement | ✓ PASS |

---

### TC-TRS-QNAME-001 — TC-TRS-QNAME-001

**Verifies:** REQ-TRS-QNAME-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| single-level element has single-segment qualified name | ✓ PASS |
| three-level nested element has three-segment qualified name | ✓ PASS |

---

### TC-TRS-QNAME-002 — TC-TRS-QNAME-002

**Verifies:** REQ-TRS-QNAME-002  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| name: in _index.md replaces directory name in qualified names | ✓ PASS |

---

### TC-TRS-QNAME-003 — TC-TRS-QNAME-003

**Verifies:** REQ-TRS-QNAME-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| name: in frontmatter replaces the filename stem | ✓ PASS |

---

### TC-TRS-QNAME-004 — TC-TRS-QNAME-004

**Verifies:** REQ-TRS-QNAME-004  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| _index.md contributes no _index name segment | ✓ PASS |

---

### TC-TRS-SAFE-001 — TC-TRS-SAFE-001

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

### TC-TRS-SAFE-002 — TC-TRS-SAFE-002

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

### TC-TRS-SAFE-003 — TC-TRS-SAFE-003

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

### TC-TRS-SAFE-004 — TC-TRS-SAFE-004

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

### TC-TRS-SAFE-005 — TC-TRS-SAFE-005

**Verifies:** REQ-TRS-SAFE-005  
**Result:** ✓ PASS (13 passed, 0 failed)

| Scenario | Result |
|---|---|
| metrics --json output | ✓ PASS |
| --deny W033 exits non-zero | ✓ PASS |

---

### TC-TRS-SAFE-006 — TC-TRS-SAFE-006

**Verifies:** REQ-TRS-SAFE-006  
**Result:** ✓ PASS (11 passed, 0 failed)

| Scenario | Result |
|---|---|
| flagged: mixed-criticality sharing with no FFI argument | ✓ PASS |
| excused (ffiRationale): no W034 | ✓ PASS |
| excused (accepted breakdownAdr): no W034 | ✓ PASS |
| --deny W034 exits non-zero | ✓ PASS |

---

### TC-TRS-SAFE-007 — TC-TRS-SAFE-007

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

### TC-TRS-SAFE-008 — TC-TRS-SAFE-008

**Verifies:** REQ-TRS-SAFE-008  
**Result:** ✓ PASS (23 passed, 0 failed)

| Scenario | Result |
|---|---|
| main: Argument + AssumptionOfUse validate with no errors | ✓ PASS |
| badref: unresolved Argument.supports/evidence yields E855 | ✓ PASS |
| orphan: claim Argument with no supports/evidence yields W040 | ✓ PASS |

---

### TC-TRS-SCRIPT-001 — TC-TRS-SCRIPT-001

**Verifies:** REQ-TRS-SCRIPT-001  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| a model with no scripts directory runs normally | ✓ PASS |

---

### TC-TRS-SCRIPT-002 — TC-TRS-SCRIPT-002

**Verifies:** REQ-TRS-SCRIPT-002  
**Result:** ✓ PASS (12 passed, 0 failed)

| Scenario | Result |
|---|---|
| deterministic output | ✓ PASS |

---

### TC-TRS-SCRIPT-003 — TC-TRS-SCRIPT-003

**Verifies:** REQ-TRS-SCRIPT-003  
**Result:** ✓ PASS (16 passed, 0 failed)

| Scenario | Result |
|---|---|
| stdout and stderr output | ✓ PASS |

---

### TC-TRS-SCRIPT-004 — TC-TRS-SCRIPT-004

**Verifies:** REQ-TRS-SCRIPT-004  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| duplicate name is a load error | ✓ PASS |

---

### TC-TRS-SCRIPT-005 — TC-TRS-SCRIPT-005

**Verifies:** REQ-TRS-SCRIPT-005  
**Result:** ✓ PASS (18 passed, 0 failed)

| Scenario | Result |
|---|---|
| no scripts directory reports none and exits 0 | ✓ PASS |

---

### TC-TRS-SCRIPT-006 — TC-TRS-SCRIPT-006

**Verifies:** REQ-TRS-SCRIPT-006  
**Result:** ✓ PASS (11 passed, 0 failed)

| Scenario | Result |
|---|---|
| built-in validate is unaffected by check scripts | ✓ PASS |

---

### TC-TRS-SEC-001 — TC-TRS-SEC-001

**Verifies:** REQ-TRS-SEC-001  
**Result:** ✓ PASS (14 passed, 0 failed)

| Scenario | Result |
|---|---|
| linked safety damage scenario: no errors, W030 only on the unlinked one | ✓ PASS |
| co-analysis --json carries goals and unlinkedSafetyDamage | ✓ PASS |

---

### TC-TRS-SEC-002 — TC-TRS-SEC-002

**Verifies:** REQ-TRS-SEC-002  
**Result:** ✓ PASS (17 passed, 0 failed)

| Scenario | Result |
|---|---|
| untreated high-risk threat: W031 on it only; W032 on under-CAL goal; no errors | ✓ PASS |
| cyber-risk --json carries the per-threat risk fields | ✓ PASS |

---

### TC-TRS-SEC-003 — TC-TRS-SEC-003

**Verifies:** REQ-TRS-SEC-003  
**Result:** ✓ PASS (21 passed, 0 failed)

| Scenario | Result |
|---|---|
| worked example rolls up to medium; W035 vs declared high; no errors | ✓ PASS |
| matched declared feasibility clears W035 | ✓ PASS |
| types command lists the new attack-tree types | ✓ PASS |

---

### TC-TRS-SPEC-001 — TC-TRS-SPEC-001

**Verifies:** REQ-TRS-SPEC-001  
**Result:** ✓ PASS (61 passed, 0 failed)

| Scenario | Result |
|---|---|
| spec safety documents cveId, safeState and ftti | ✓ PASS |

---

### TC-TRS-SPEC-002 — TC-TRS-SPEC-002

**Verifies:** REQ-TRS-SPEC-002  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| spec types distinguishes the constructs | ✓ PASS |

---

### TC-TRS-TAG-001 — TC-TRS-TAG-001

**Verifies:** REQ-TRS-TAG-001  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| matrix --tag filters rows but not columns | ✓ PASS |

---

### TC-TRS-TARA-001 — TC-TRS-TARA-001

**Verifies:** REQ-TRS-TARA-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E940 | ✓ PASS |
| trigger E941 | ✓ PASS |
| trigger W905 | ✓ PASS |

---

### TC-TRS-TRACE-001 — TC-TRS-TRACE-001

**Verifies:** REQ-TRS-TRACE-001  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| verifiedBy is computed from TestCase.verifies: | ✓ PASS |

---

### TC-TRS-TRACE-002 — TC-TRS-TRACE-002

**Verifies:** REQ-TRS-TRACE-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| derivedFrom with no breakdownAdr produces E310 | ✓ PASS |
| derivedFrom with valid breakdownAdr produces no E310 | ✓ PASS |

---

### TC-TRS-TRACE-003 — TC-TRS-TRACE-003

**Verifies:** REQ-TRS-TRACE-003  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| approved requirement with proposed ADR produces W303 | ✓ PASS |

---

### TC-TRS-TRACE-004 — TC-TRS-TRACE-004

**Verifies:** REQ-TRS-TRACE-004  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| approved leaf requirement with no satisfies produces W300 | ✓ PASS |
| approved leaf requirement with satisfies produces no W300 | ✓ PASS |

---

### TC-TRS-TRACE-005 — TC-TRS-TRACE-005

**Verifies:** REQ-TRS-TRACE-005  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| architecture element satisfying a parent requirement produces E312 | ✓ PASS |

---

### TC-TRS-TRACE-006 — TC-TRS-TRACE-006

**Verifies:** REQ-TRS-TRACE-006  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| software element satisfying hardware requirement produces E313 | ✓ PASS |
| software element satisfying system requirement produces no E313 | ✓ PASS |

---

### TC-TRS-TRACE-007 — TC-TRS-TRACE-007

**Verifies:** REQ-TRS-TRACE-007  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| software PartDef with hardware supertype produces E315 | ✓ PASS |

---

### TC-TRS-TRACE-008 — TC-TRS-TRACE-008

**Verifies:** REQ-TRS-TRACE-008  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| deployment package with no hardware allocation produces E314 | ✓ PASS |
| deployment package with hardware allocation produces no E314 | ✓ PASS |

---

### TC-TRS-TRACE-009 — TC-TRS-TRACE-009

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

### TC-TRS-TRACE-010 — TC-TRS-TRACE-010

**Verifies:** REQ-TRS-TRACE-010  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| W306 message names the sub-conditions | ✓ PASS |
| a fully-integrated high-integrity requirement produces no W306 | ✓ PASS |
| W306 is gateable with --deny | ✓ PASS |

---

### TC-TRS-TYPE-001 — TC-TRS-TYPE-001

**Verifies:** REQ-TRS-TYPE-001  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-002 — TC-TRS-TYPE-002

**Verifies:** REQ-TRS-TYPE-002  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-003 — TC-TRS-TYPE-003

**Verifies:** REQ-TRS-TYPE-003  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-004 — TC-TRS-TYPE-004

**Verifies:** REQ-TRS-TYPE-004  
**Result:** ✓ PASS (8 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-005 — TC-TRS-TYPE-005

**Verifies:** REQ-TRS-TYPE-005  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-006 — TC-TRS-TYPE-006

**Verifies:** REQ-TRS-TYPE-006  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-007 — TC-TRS-TYPE-007

**Verifies:** REQ-TRS-TYPE-007  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-008 — TC-TRS-TYPE-008

**Verifies:** REQ-TRS-TYPE-008  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-009 — TC-TRS-TYPE-009

**Verifies:** REQ-TRS-TYPE-009  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-010 — TC-TRS-TYPE-010

**Verifies:** REQ-TRS-TYPE-010  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-011 — TC-TRS-TYPE-011

**Verifies:** REQ-TRS-TYPE-011  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-012 — TC-TRS-TYPE-012

**Verifies:** REQ-TRS-TYPE-012  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-013 — TC-TRS-TYPE-013

**Verifies:** REQ-TRS-TYPE-013  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-014 — TC-TRS-TYPE-014

**Verifies:** REQ-TRS-TYPE-014  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| show renders the declared type (GH #42 type_label) | ✓ PASS |

---

### TC-TRS-TYPE-015 — TC-TRS-TYPE-015

**Verifies:** REQ-TRS-TYPE-015  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-TYPE-016 — TC-TRS-TYPE-016

**Verifies:** REQ-TRS-TYPE-016  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| export reports each element at its declared type | ✓ PASS |
| sibling file with bogus type produces E005 | ✓ PASS |

---

### TC-TRS-VAL-001 — TC-TRS-VAL-001

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

### TC-TRS-VAL-002 — TC-TRS-VAL-002

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

### TC-TRS-VAL-003 — TC-TRS-VAL-003

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

### TC-TRS-VAL-004 — TC-TRS-VAL-004

**Verifies:** REQ-TRS-VAL-004  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| E841: derivedFromSafetyGoal element missing integrity level | ✓ PASS |
| E842: derivedFrom element missing integrity level | ✓ PASS |
| E843: satisfies element missing integrity level | ✓ PASS |
| W808: integrity level lower than source without breakdownAdr | ✓ PASS |

---

### TC-TRS-VAL-005 — TC-TRS-VAL-005

**Verifies:** REQ-TRS-VAL-005,REQ-TRS-VAL-006  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| findings include rule code, element reference, and description | ✓ PASS |
| parse-time error is attributed to source file | ✓ PASS |

---

### TC-TRS-VAL-006 — TC-TRS-VAL-006

**Verifies:** REQ-TRS-VAL-007  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| E-code findings are under Errors section | ✓ PASS |
| W-code findings are under Warnings section | ✓ PASS |

---

### TC-TRS-VAL-007 — TC-TRS-VAL-007

**Verifies:** REQ-TRS-VAL-007  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| Errors section does not contain W-codes | ✓ PASS |
| Warnings section does not contain E-codes | ✓ PASS |

---

### TC-TRS-VAL-008 — TC-TRS-VAL-008

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

### TC-TRS-VAL-009 — TC-TRS-VAL-009

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

### TC-TRS-VAL-010 — TC-TRS-VAL-010

**Verifies:** REQ-TRS-VAL-010  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| resolving functions across all languages produce no W009 | ✓ PASS |
| renamed/missing tests produce W009 | ✓ PASS |

---

### TC-TRS-VAL-011 — TC-TRS-VAL-011

**Verifies:** REQ-TRS-VAL-011  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| scaffold-gherkin --fix clears E106 | ✓ PASS |

---

### TC-TRS-VAL-012 — TC-TRS-VAL-012

**Verifies:** REQ-TRS-VAL-012  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| absolute and file:// resolve without new W004 | ✓ PASS |

---

### TC-TRS-VAL-013 — TC-TRS-VAL-013

**Verifies:** REQ-TRS-VAL-013  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| --fetch-remote fetches and verifies | ✓ PASS |

---

### TC-TRS-VAL-014 — TC-TRS-VAL-014

**Verifies:** REQ-TRS-VAL-014  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| drift checks scoped to active TestCases | ✓ PASS |

---

### TC-TRS-VAL-015 — TC-TRS-VAL-015

**Verifies:** REQ-TRS-VAL-015  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| --deny I010 gates | ✓ PASS |

---

### TC-TRS-VAL-016 — TC-TRS-VAL-016

**Verifies:** REQ-TRS-VAL-016  
**Result:** ✓ PASS (6 passed, 0 failed)

| Scenario | Result |
|---|---|
| list --has-wcet --json includes wcet | ✓ PASS |
| SIL requirement with wcet, no measuring test produces W029 | ✓ PASS |
| W029 is gateable with --deny | ✓ PASS |

---

### TC-TRS-VAR-001 — TC-TRS-VAR-001

**Verifies:** REQ-TRS-VAR-001  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| matrix on flat model falls back without error | ✓ PASS |
| unresolved appliesWhen is E209 when dormant | ✓ PASS |
| feature model without Configuration emits no W015 | ✓ PASS |

---

### TC-TRS-VAR-002 — TC-TRS-VAR-002

**Verifies:** REQ-TRS-VAR-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| refs of deselecting config excludes conditioned TestCase | ✓ PASS |

---

### TC-TRS-VAR-003 — TC-TRS-VAR-003

**Verifies:** REQ-TRS-VAR-003  
**Result:** ✓ PASS (14 passed, 0 failed)

| Scenario | Result |
|---|---|
| bare QName remains back-compatible | ✓ PASS |

---

### TC-TRS-VAR-004 — TC-TRS-VAR-004

**Verifies:** REQ-TRS-VAR-004  
**Result:** ✓ PASS (10 passed, 0 failed)

| Scenario | Result |
|---|---|
| text matrix renders configurations and cells | ✓ PASS |

---

### TC-TRS-VAR-005 — TC-TRS-VAR-005

**Verifies:** REQ-TRS-VAR-005  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| W015 is gateable with --deny | ✓ PASS |
| dormant model emits no W015 | ✓ PASS |

---

### TC-TRS-VAR-006 — TC-TRS-VAR-006

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

### TC-TRS-XREF-001 — TC-TRS-XREF-001

**Verifies:** REQ-TRS-XREF-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| absolute supertype reference resolves correctly | ✓ PASS |
| absolute reference to non-existent element produces an error | ✓ PASS |

---

### TC-TRS-XREF-002 — TC-TRS-XREF-002

**Verifies:** REQ-TRS-XREF-002  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| sibling reference resolves within the same package | ✓ PASS |

---

### TC-TRS-XREF-003 — TC-TRS-XREF-003

**Verifies:** REQ-TRS-XREF-003  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| dangling reference produces an error without crashing | ✓ PASS |

---

### TC-TRS-XREF-004 — TC-TRS-XREF-004

**Verifies:** REQ-TRS-XREF-004  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| two-element supertype cycle is detected without crashing | ✓ PASS |

---

### TC-TRS-XREF-005 — TC-TRS-XREF-005

**Verifies:** REQ-TRS-XREF-005  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| verifies: resolves by stable id: regardless of file path | ✓ PASS |

---

### TC-TRS-XREF-006 — TC-TRS-XREF-006

**Verifies:** REQ-TRS-XREF-006  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| an unresolved reference not starting with the root name gets no hint | ✓ PASS |

---
