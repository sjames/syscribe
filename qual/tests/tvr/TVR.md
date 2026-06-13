# Tool Validation Report

**Tool:** syscribe CLI validator  
**Version:** syscribe 0.26.25  
**Standard:** ISO 26262:2018 Part 8 §11 (TCL2), IEC 61508:2010 Part 3 Annex D  
**Date:** 2026-06-13  
**TRS:** `qual/Requirements/`  **Test cases:** `qual/TestCases/`

---

## 1. Summary

| Metric | Value |
|---|---|
| Total test cases | 2 |
| Passed | 2 |
| Failed | 0 |
| Overall verdict | **PASS** |

---

## 2. Results

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
