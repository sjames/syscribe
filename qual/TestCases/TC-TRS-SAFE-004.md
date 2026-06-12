---
id: TC-TRS-SAFE-004
type: TestCase
testLevel: L3
status: draft
name: Verify that CybersecurityGoal, SecurityControl, and VulnerabilityReport validation rules E815-E824, E827-E832, W802-W804, and W807 are enforced
verifies:
  - REQ-TRS-SAFE-004
---

Verify that the tool detects all CybersecurityGoal, SecurityControl, and VulnerabilityReport field-level and cross-reference validation errors and warnings.

```gherkin
Feature: CybersecurityGoal, SecurityControl, and VulnerabilityReport validation rules

  Scenario: E815 — missing required field on CybersecurityGoal triggers error
    Given a CybersecurityGoal element missing the id field
    When the tool validates the model
    Then at least one E815 finding is emitted

  Scenario: E816 — CybersecurityGoal id not matching CSG-* pattern triggers error
    Given a CybersecurityGoal with id that does not match the CSG-* pattern
    When the tool validates the model
    Then at least one E816 finding is emitted

  Scenario: E817 — invalid securityProperty value triggers error
    Given a valid CybersecurityGoal with securityProperty set to an invalid value
    When the tool validates the model
    Then at least one E817 finding is emitted

  Scenario: E818 — invalid calLevel value triggers error
    Given a valid CybersecurityGoal with calLevel set to an invalid value
    When the tool validates the model
    Then at least one E818 finding is emitted

  Scenario: E819 — missing required field on SecurityControl triggers error
    Given a SecurityControl element missing the id field
    When the tool validates the model
    Then at least one E819 finding is emitted

  Scenario: E820 — SecurityControl id not matching SC-* pattern triggers error
    Given a SecurityControl with id that does not match the SC-* pattern
    When the tool validates the model
    Then at least one E820 finding is emitted

  Scenario: E821 — invalid controlType value triggers error
    Given a valid SecurityControl with controlType set to an invalid value
    When the tool validates the model
    Then at least one E821 finding is emitted

  Scenario: E822 — missing required field on VulnerabilityReport triggers error
    Given a VulnerabilityReport element missing the id field
    When the tool validates the model
    Then at least one E822 finding is emitted

  Scenario: E823 — VulnerabilityReport id not matching VR-* pattern triggers error
    Given a VulnerabilityReport with id that does not match the VR-* pattern
    When the tool validates the model
    Then at least one E823 finding is emitted

  Scenario: E824 — cvssScore out of range triggers error
    Given a valid VulnerabilityReport with cvssScore set to a value outside 0.0-10.0
    When the tool validates the model
    Then at least one E824 finding is emitted

  Scenario: E827 — unresolvable threatScenarios reference on CybersecurityGoal triggers error
    Given a CybersecurityGoal with a threatScenarios entry that does not resolve to any element
    When the tool validates the model
    Then at least one E827 finding is emitted

  Scenario: E828 — unresolvable implementsGoals reference on SecurityControl triggers error
    Given a SecurityControl with an implementsGoals entry that does not resolve to any element
    When the tool validates the model
    Then at least one E828 finding is emitted

  Scenario: E829 — unresolvable mitigatedBy reference on VulnerabilityReport triggers error
    Given a VulnerabilityReport with a mitigatedBy entry that does not resolve to any element
    When the tool validates the model
    Then at least one E829 finding is emitted

  Scenario: E830 — unresolvable affectedElements reference on VulnerabilityReport triggers error
    Given a VulnerabilityReport with an affectedElements entry that does not resolve to any element
    When the tool validates the model
    Then at least one E830 finding is emitted

  Scenario: E831 — derivedFromSecurityGoal that does not resolve to a CybersecurityGoal triggers error
    Given an element with derivedFromSecurityGoal pointing to a non-existent element
    When the tool validates the model
    Then at least one E831 finding is emitted

  Scenario: E832 — derivedFromSafetyGoal that does not resolve to a SafetyGoal triggers error
    Given an element with derivedFromSafetyGoal pointing to a non-existent element
    When the tool validates the model
    Then at least one E832 finding is emitted

  Scenario: W802 — CybersecurityGoal not implemented by any SecurityControl triggers warning
    Given a valid CybersecurityGoal with no SecurityControl referencing it
    When the tool validates the model
    Then at least one W802 finding is emitted

  Scenario: W803 — VulnerabilityReport with status open triggers warning
    Given a valid VulnerabilityReport with status set to open
    When the tool validates the model
    Then at least one W803 finding is emitted

  Scenario: W804 — CybersecurityGoal not referenced by any Requirement triggers warning
    Given a valid CybersecurityGoal with no Requirement pointing to it via derivedFromSecurityGoal
    When the tool validates the model
    Then at least one W804 finding is emitted

  Scenario: W807 — security Requirement with no verificationMethod triggers warning
    Given a Requirement with derivedFromSecurityGoal set but no verificationMethod field
    When the tool validates the model
    Then at least one W807 finding is emitted
```
