---
id: TC-TRS-BT-025
type: TestCase
testLevel: L3
status: active
name: "Verify export fails on unresolved required parameters unless --allow-open"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-025.sh
verifies:
  - REQ-TRS-BT-025
---

```gherkin
Feature: Strict export
  Scenario: strict
    Given a required parameter with no value, default or fallback
    When params export is run
    Then it exits non-zero naming every unresolved parameter

  Scenario: allow-open
    Given the same model
    When params export --allow-open is run
    Then a marked placeholder is emitted for each
```
