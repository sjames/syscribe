---
id: TC-TRS-HPLE-001
type: TestCase
testLevel: L3
status: draft
name: "Verify subConfigurations resolves a named Configuration and gates on its internal validity (dangling, wrong-type, SAT-invalid)."
verifies:
  - REQ-TRS-HPLE-001
---

```gherkin
Feature: subConfigurations resolution and peer-validity gate
  Scenario: a valid peer Configuration consolidates cleanly
    Given a Configuration whose subConfigurations names a real, internally-valid peer Configuration
    When the model is validated
    Then no subConfigurations error is raised

  Scenario: a dangling subConfigurations name is rejected
    Given a Configuration whose subConfigurations names an id that resolves to nothing
    When the model is validated
    Then a dangling-reference error is raised

  Scenario: a subConfigurations name resolving to a non-Configuration is rejected
    Given a Configuration whose subConfigurations names a real element that is not a Configuration
    When the model is validated
    Then a wrong-type error is raised

  Scenario: a subConfigurations name resolving to a SAT-invalid Configuration is rejected
    Given a Configuration whose subConfigurations names a peer Configuration that does not satisfy its own feature model
    When the model is validated
    Then a not-internally-valid error is raised
```
