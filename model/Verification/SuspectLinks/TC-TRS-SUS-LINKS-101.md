---
type: TestCase
id: TC-TRS-SUS-LINKS-101
name: "Feature is purely additive: a model with no baselines emits no new findings"
status: active
testLevel: L3
sourceFile: repo:crates/syscribe/tests/suspect_links.rs
tags:
  - traceability
  - suspect-links
  - integration
verifies:
  - REQ-TRS-SUS-LINKS-000
---

Integration test asserting the opt-in guarantee at the model level: introducing the
suspect-link mechanism must not change validation output for a model that has not
baselined any links.

```gherkin
Feature: Opt-in additivity

  Scenario: An unbaselined model has identical findings before and after the feature
    Given an existing model with trace links but no traceBaselines fields anywhere
    When validate is run
    Then the set of findings contains no W090
    And the finding set is unchanged from the pre-feature baseline

  Scenario: Baselining is required to activate detection
    Given a model with a trace link from S to T
    And no baseline is recorded
    When T's projection changes
    And validate is run
    Then no W090 is emitted
    And the link only becomes checkable after "suspect accept S T"
```
