---
type: TestCase
id: TC-TRS-COMPLY-001
name: "compliance maps work products to process areas with present/approved/missing"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/compliance.rs
verifies:
  - REQ-TRS-COMPLY-001
tags:
  - cli
---

```gherkin
Feature: compliance report

  Scenario: built-in mapping
    Then iso26262 reports a missing FMEA, a partial safety goal and a complete HARA for a small model

  Scenario: configured mapping and lens
    Then a [standards] table replaces the items, --config removes gated elements and --fail-on-missing gates

  Scenario: errors
    Then an unknown standard and a malformed table exit 1
```
