---
id: TC-TRS-META-002
type: TestCase
testLevel: L3
status: draft
name: "Verify diagrams render applied MetadataDef stereotypes as «Name» banners: a stereotyped element shows «Critical» in addition to its type-keyword banner; an element with no application shows no spurious banner."
verifies:
  - REQ-TRS-META-002
---

```gherkin
Feature: applied stereotypes render as «Name» banners in element diagrams
  Scenario: a stereotyped element shows the «Name» banner
    Given a MetadataDef Critical (annotates PartDef) applied to a PartDef via metadata:
    When the element diagram for that PartDef is rendered
    Then the SVG contains a «Critical» banner using the stereotype styling
    And the type-keyword «part def» banner is still present

  Scenario: an element without an application shows no stereotype banner
    Given a PartDef that applies no MetadataDef
    When its element diagram is rendered
    Then the SVG contains no «Critical» banner
```
