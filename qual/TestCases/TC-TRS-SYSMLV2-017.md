---
id: TC-TRS-SYSMLV2-017
type: TestCase
testLevel: L3
status: draft
name: "Verify a package-relative typedBy: reference across SysMLv2 packages suppresses W007 on the referenced def, is a real connectivity-visible TypedBy edge, and a genuinely unused def still raises W007."
verifies:
  - REQ-TRS-SYSMLV2-017
---

```gherkin
Feature: cross-package typedBy: W007 usage tracking and TypedBy graph edge
  Scenario: a package-relative typedBy: reference to a def in another package suppresses W007 on that def
    Given a Part usage typed by a PartDef declared in a different SysMLv2 package, referenced only that way
    When the model is validated
    Then W007 does not fire for the referenced PartDef

  Scenario: the same cross-package reference is a real, connectivity-visible TypedBy edge
    Given the same Part usage and its cross-package typedBy: target
    When connectivity is queried rooted at the Part usage
    Then the output shows a typedBy edge to the referenced PartDef

  Scenario: a genuinely unused PartDef in the same model still raises W007
    Given a PartDef that nothing anywhere references as supertype or typedBy
    When the model is validated
    Then W007 fires for that PartDef
```
