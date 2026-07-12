---
type: TestCase
id: TC-TRS-SUS-LINKS-003
name: "Baselines apply across all trace-link kinds"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/suspect_links.rs
tags:
  - traceability
  - suspect-links
verifies:
  - REQ-TRS-SUS-LINKS-003
---

Verifies that the suspect mechanism operates for every trace-link kind, not just the
requirement-centric ones, and that a single baseline map keyed by target serves links of
different kinds.

```gherkin
Feature: Coverage of all trace-link kinds

  Scenario Outline: A baselined link of each kind is detected as suspect on change
    Given a source with a <kind> link to target T that is baselined
    When T's projection changes
    Then the <kind> link to T is reported suspect

    Examples:
      | kind          |
      | verifies      |
      | derivedFrom   |
      | satisfies     |
      | satisfiedBy   |
      | refines       |
      | implementedBy |
      | supertype     |
      | subsets       |
      | redefines     |
      | breakdownAdr  |

  Scenario: One baseline entry serves a target referenced by two link kinds
    Given a source that references target T via both refines and satisfies
    And T is baselined once
    When T's projection changes
    Then the single baseline entry drives detection for both links

  Scenario: Targets resolve id-first then by qualified name
    Given a baselined link whose key is a stable id
    And another whose key is a qualified name
    Then both resolve to their target elements
```
