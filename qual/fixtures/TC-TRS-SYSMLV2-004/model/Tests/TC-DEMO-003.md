---
type: TestCase
id: TC-DEMO-003
name: "Verifies a hand-authored native PartDef (rejection case)"
status: active
testLevel: L2
verifies:
  - Arch::SomePart
---

```gherkin
Feature: verify a hand-authored non-Requirement element
  Scenario: hand-authored PartDef is rejected
    Given a hand-authored native PartDef with no SysMLv2 involvement
    When the model is validated
    Then the existing wrong-type finding is raised
```
