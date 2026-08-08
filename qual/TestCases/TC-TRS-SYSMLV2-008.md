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

  Scenario: a @SyscribeDomain-lifted domain genuinely drives the existing domain-compatibility check
    Given a SysMLv2 part def carrying @SyscribeDomain { value = 'software'; } and satisfying a native Requirement with reqDomain: hardware
    When the model is validated
    Then the existing E313 domain-mismatch error is raised

  Scenario: a @SyscribeImplementedBy-lifted path genuinely drives the existing disk-check
    Given a SysMLv2 part def carrying @SyscribeImplementedBy { path = '...'; } naming a path that does not exist on disk
    When the model is validated
    Then the existing W023 warning is raised

  Scenario: a part def with no annotation is unaffected
    Given a SysMLv2 part def with none of the four annotations
    When the model is exported
    Then the synthesized element carries no domain, asilLevel, silLevel, plLevel, shortName, or implementedBy field
```
