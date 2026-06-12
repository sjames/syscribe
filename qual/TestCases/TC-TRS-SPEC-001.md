---
id: TC-TRS-SPEC-001
type: TestCase
testLevel: L3
status: draft
name: "Verify the discoverable syscribe spec documents the safety/security types and analysis fields."
verifies:
  - REQ-TRS-SPEC-001
---

Verify that `syscribe spec types`, `syscribe spec fields`, and `syscribe spec safety` expose the safety/security element types and analysis fields, so an author can discover them.

```gherkin
Feature: discoverable spec reference completeness

  Scenario: spec types lists the safety/security types incl. FMEAEntry
    When the tool runs `spec types`
    Then FMEAEntry and every other safety/security type appears

  Scenario: spec fields documents the safety analysis fields
    When the tool runs `spec fields`
    Then each safety analysis field (severity..recommendedAction) appears

  Scenario: spec fields documents the security analysis fields
    When the tool runs `spec fields`
    Then each security analysis field (damageSeverity..mitigatedBy) appears

  Scenario: spec safety documents cveId, safeState and ftti
    When the tool runs `spec safety`
    Then cveId, safeState and ftti appear
```
