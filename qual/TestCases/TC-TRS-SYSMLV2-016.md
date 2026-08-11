---
id: TC-TRS-SYSMLV2-016
type: TestCase
testLevel: L3
status: draft
name: "Verify a package-relative typedBy: reference to a documented target across SysMLv2 packages suppresses W600, an equally-undocumented cross-package target still raises it, and the total W600 count matches exactly the elements expected to still fire."
verifies:
  - REQ-TRS-SYSMLV2-016
---

```gherkin
Feature: cross-package typedBy: W600 suppression
  Scenario: a package-relative typedBy: reference to a documented target across packages suppresses W600
    Given a Part usage typed by a documented PartDef declared in a different SysMLv2 package
    When the model is validated
    Then W600 fires exactly twice, for the undocumented PartDef and the Part usage typed by it
    And no other element raises W600
```
