---
id: TC-TRS-META-001
type: TestCase
testLevel: L3
status: draft
name: "Verify stereotypes as MetadataDef applications: valid apply (bare + tagged), E317 unresolved, E318 appliesTo mismatch, W045 undeclared tag key, show «Name», list --metadata."
verifies:
  - REQ-TRS-META-001
---

```gherkin
Feature: stereotypes via MetadataDef applications (metadata:)
  Scenario: a valid application validates clean
    Given a MetadataDef Critical (appliesTo PartDef) and a PartDef applying it (bare and with a tagged value)
    When validate runs
    Then no E317/E318/W045 is raised for those elements

  Scenario: an unresolved application raises E317
    Given a PartDef whose metadata: references a non-existent MetadataDef
    When validate runs
    Then E317 is raised naming that element

  Scenario: an inapplicable stereotype raises E318
    Given a PartDef applying a MetadataDef whose appliesTo excludes PartDef
    When validate runs
    Then E318 is raised naming that element

  Scenario: an undeclared tagged-value key warns W045
    Given a PartDef applying Critical with a key not in Critical's features
    When validate runs
    Then W045 is raised naming that element

  Scenario: show and list surface the applied stereotype
    When show is run on the stereotyped PartDef
    Then «Critical» (with its tagged value) is displayed
    And list PartDef --metadata Stereotypes::Critical includes it
```
