---
id: TC-TRS-SYSMLV2-008
type: TestCase
testLevel: L3
status: draft
name: "Verify @SyscribeDomain/@SyscribeIntegrity/@SyscribeShortName/@SyscribeImplementedBy lift onto a SysMLv2 part def/part, existing validation fires unchanged, and a no-annotation part is unaffected."
verifies:
  - REQ-TRS-SYSMLV2-008
---

```gherkin
Feature: fixed @Syscribe* field annotations lift onto a part def/part
  Scenario: all four annotations lift onto a part def
    Given a SysMLv2 part def carrying @SyscribeDomain, @SyscribeIntegrity (asil), @SyscribeShortName, and @SyscribeImplementedBy
    When the model is exported
    Then the synthesized element shows domain, asilLevel, shortName, and implementedBy matching the annotation values

  Scenario: @SyscribeIntegrity with both asil and sil raises the existing mutual-exclusion warning
    Given a SysMLv2 part def carrying @SyscribeIntegrity { asil = 'D'; sil = 2; }
    When the model is validated
    Then the existing W006 asilLevel/silLevel mutual-exclusion warning is raised exactly once

  Scenario: a part def with no annotation is unaffected
    Given a SysMLv2 part def with none of the four annotations
    When the model is exported
    Then the synthesized element carries no domain, asilLevel, silLevel, plLevel, shortName, or implementedBy field
```
