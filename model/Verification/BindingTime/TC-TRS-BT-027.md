---
id: TC-TRS-BT-027
type: TestCase
testLevel: L3
status: draft
name: "Verify consumedBy resolves and is shown"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-027.sh
verifies:
  - REQ-TRS-BT-027
---

```gherkin
Feature: Parameter consumer
  Scenario: unresolvable
    Given a consumedBy that resolves to nothing
    When validate is run
    Then it is reported like an unresolvable implementedBy

  Scenario: trace
    Given a resolvable consumedBy
    When trace is run
    Then the consumer is listed for the parameter
```
