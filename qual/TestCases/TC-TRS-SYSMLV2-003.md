---
id: TC-TRS-SYSMLV2-003
type: TestCase
testLevel: L3
status: draft
name: "Verify a SysMLv2 element's satisfy/verify resolves against a native Requirement by both quoted-id and qualified-name form."
verifies:
  - REQ-TRS-SYSMLV2-003
---

```gherkin
Feature: SysMLv2 satisfy/verify targets a native Requirement
  Scenario: satisfy by quoted REQ-* id resolves cleanly
    Given a SysMLv2 element with satisfy '<REQ-id>'; targeting a real native Requirement
    When the model is validated
    Then no dangling-reference finding is raised and the requirement's satisfying-element warning is suppressed

  Scenario: satisfy by Syscribe qualified name resolves cleanly
    Given a SysMLv2 element with satisfy <Package>::'<REQ-id>'; targeting a different real native Requirement
    When the model is validated
    Then no dangling-reference finding is raised for that target

  Scenario: verify targets a native Requirement
    Given a SysMLv2 requirement usage with verify '<REQ-id>'; targeting a third real native Requirement
    When the model is validated
    Then no dangling-reference finding is raised for that target
```
