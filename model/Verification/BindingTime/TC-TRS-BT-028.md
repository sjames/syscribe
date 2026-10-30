---
id: TC-TRS-BT-028
type: TestCase
testLevel: L3
status: active
name: "Verify params export --schema emits a valid JSON Schema"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-028.sh
verifies:
  - REQ-TRS-BT-028
---

```gherkin
Feature: Values-file schema
  Scenario: round trip
    Given a schema and a values file generated from one Configuration
    When a standard JSON Schema validator checks the file
    Then it validates
```
