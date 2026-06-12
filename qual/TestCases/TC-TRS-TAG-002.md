---
id: TC-TRS-TAG-002
type: TestCase
testLevel: L3
status: draft
name: "Verify list --tag multi-tag AND filtering: repeated --tag narrows to elements carrying all tags."
verifies:
  - REQ-TRS-TAG-002
---

```gherkin
Feature: list --tag multi-tag AND filtering
  Scenario: repeated --tag narrows to elements carrying every tag
    Given TestCases tagged [integration,safety], [integration], and [safety]
    When list TestCase --tag integration runs
    Then both the integration-only and the integration+safety TestCases are listed
    When list TestCase --tag integration --tag safety runs
    Then only the TestCase carrying both tags is listed
    And the single-tag and safety-only cases are excluded
    When list TestCase runs with no --tag
    Then all three TestCases are listed (filter inactive)
```
