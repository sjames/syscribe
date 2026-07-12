---
type: TestCase
id: TC-TRS-SUS-LINKS-001
name: "traceBaselines parses as a target-to-hash map and is optional"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/suspect_links.rs
tags:
  - traceability
  - suspect-links
verifies:
  - REQ-TRS-SUS-LINKS-001
---

Verifies that the `traceBaselines` frontmatter field is accepted as an optional mapping
from target identifier to an algorithm-prefixed hash string, keyed independently of link
kind, and serialized in deterministic order.

```gherkin
Feature: traceBaselines schema field

  Scenario: A source element parses a per-target baseline map
    Given a TestCase with verifies: [REQ-SCHED-BITMAP-001]
    And the TestCase has traceBaselines mapping REQ-SCHED-BITMAP-001 to "blake3:9f2a3c1d"
    When the model is loaded
    Then the element parses without error
    And the baseline for target REQ-SCHED-BITMAP-001 is "blake3:9f2a3c1d"

  Scenario: A multi-valued link carries one entry per target
    Given a TestCase with verifies: [REQ-A-001, REQ-B-001]
    And traceBaselines has entries for both REQ-A-001 and REQ-B-001
    When the model is loaded
    Then each target resolves to its own stored hash

  Scenario: The field is optional
    Given an element with trace links but no traceBaselines field
    When the model is loaded
    Then the element parses without error
    And it has no baselined links

  Scenario: Serialization is deterministic
    Given a source with baselines for targets Z, A, and M
    When the element is re-serialized
    Then the traceBaselines keys appear in sorted order
```
