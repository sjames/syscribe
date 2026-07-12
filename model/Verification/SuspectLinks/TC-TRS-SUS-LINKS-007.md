---
type: TestCase
id: TC-TRS-SUS-LINKS-007
name: "No eager propagation; suspicion advances one hop only when an intermediate is edited"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/suspect_links.rs
tags:
  - traceability
  - suspect-links
verifies:
  - REQ-TRS-SUS-LINKS-007
---

Verifies that suspect status is not flooded across the trace graph: only direct links
flip, and an upstream link becomes suspect only after the intermediate element's own
projection changes.

```gherkin
Feature: Implicit one-hop propagation

  Background:
    Given a chain C -> A -> B where every link is baselined
    And all baselines currently match

  Scenario: A target change flags only the direct link
    When B's projection changes
    And validate is run
    Then the A-to-B link is reported suspect
    And the C-to-A link is NOT reported suspect

  Scenario: Suspicion advances only after the intermediate is edited
    Given B has changed and the A-to-B link was accepted
    When A's own projection is edited
    And validate is run
    Then the C-to-A link becomes suspect

  Scenario: No transitive closure is computed
    When a leaf target deep in a chain changes
    Then only its direct dependents are flagged, not the whole ancestry
```
