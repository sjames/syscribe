---
type: TestCase
id: TC-DEMO-001
name: "Verifies a SysMLv2-mapped element by qname"
status: active
testLevel: L2
verifies:
  - SysML2::Demo::Widget
---

```gherkin
Feature: verify a SysMLv2 element
  Scenario: qname resolves cleanly
    Given a SysMLv2-mapped Widget part def
    When the model is validated
    Then no dangling-reference or wrong-type finding is raised
```
