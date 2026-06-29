---
type: TestCase
id: TC-FXPAR-001
name: "Unit test of the decomposed parent (not an integration test)"
status: draft
testLevel: L2
verifies:
  - REQ-FXPARENT-001
---

```gherkin
Feature: Parent unit check
  Scenario: a unit-level check of the parent
    Given the fixture model
    Then REQ-FXPARENT-001 has a unit (L2) test but no integration (L3-L5) test
```
