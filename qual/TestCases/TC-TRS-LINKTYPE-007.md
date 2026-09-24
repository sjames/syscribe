---
id: TC-TRS-LINKTYPE-007
type: TestCase
testLevel: L3
status: draft
name: "Verify follow traverses custom and built-in links forward, inverse and reversed, one hop or transitively, with depth bounds, cycle termination and text/json/dot formats."
verifies:
  - REQ-TRS-LINKTYPE-007
---

```gherkin
Feature: User-defined link types (TC-TRS-LINKTYPE-007)

  Scenario: one hop forward
  Scenario: transitive forward
  Scenario: depth bounds the traversal
  Scenario: inverse name and --reverse traverse backwards
  Scenario: transitive traversal terminates on cycles
  Scenario: built-in link and reverse-index names
  Scenario: json and dot formats
  Scenario: unknown link or element exits non-zero
```
