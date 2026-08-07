---
id: TC-TRS-HPLE-003
type: TestCase
testLevel: L3
status: draft
name: "Verify a cross-tier parameterBindings entry is rejected when the owning tier doesn't select the feature, or when a nearer tier already bound it."
verifies:
  - REQ-TRS-HPLE-003
---

```gherkin
Feature: Cross-tier parameterBindings legality
  Scenario: a genuinely open, selected cross-tier parameter validates cleanly
    Given a consolidating Configuration binding a parameter the owning peer Configuration selects and leaves open
    When the model is validated
    Then no cross-tier binding error is raised

  Scenario: binding a parameter of a feature the owning tier does not select is rejected
    Given a consolidating Configuration binding a parameter whose owning peer Configuration does not select that feature
    When the model is validated
    Then a not-selected-by-owner error is raised

  Scenario: double-binding a parameter a nearer tier already closed is rejected
    Given a peer Configuration that already binds a parameter in its own parameterBindings
    And a consolidating Configuration that binds the same parameter again
    When the model is validated
    Then an already-bound-by-a-nearer-tier error is raised naming the peer Configuration
```
