---
id: TC-TST-XRF-001
type: TestCase
name: Test TC for id-based cross-reference
status: draft
testLevel: L3
verifies:
  - REQ-TST-XRF-001
---

Test case that references a requirement by stable REQ-* id.

```gherkin
Feature: Id-based cross-reference resolution

  Scenario: TestCase verifies Requirement by id
    Given a TestCase with verifies pointing to REQ-TST-XRF-001
    When the tool resolves cross-references
    Then the reference is resolved successfully without error
```
