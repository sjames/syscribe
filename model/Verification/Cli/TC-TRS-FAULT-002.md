---
type: TestCase
id: TC-TRS-FAULT-002
name: "The injection campaign reports which fault sets the invariants survive"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/fault_campaign.rs
verifies:
  - REQ-TRS-FAULT-002
tags:
  - fault-injection
---

```gherkin
Feature: Fault-injection campaign

  Scenario: single faults
    Then every applicable fault is run against the invariants, survivors and defeats are reported, and a defeat gives exit 2 unless --report-only

  Scenario: cut sets
    Then the minimal cut sets of a fault tree up to the order are run as fault sets, and cut sets with an event that has no fault model are reported as not injectable

  Scenario: honesty
    Then invariants that fail without faults stop the campaign, a model without an @invariant scenario is a usage error, and FMEA rows without a fault model are listed as not exercised
```
