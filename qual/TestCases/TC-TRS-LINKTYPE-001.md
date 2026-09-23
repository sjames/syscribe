---
id: TC-TRS-LINKTYPE-001
type: TestCase
testLevel: L3
status: draft
name: "Verify [linkTypes] declarations are parsed, malformed entries raise W630 and are ignored, and an unconfigured model is unaffected."
verifies:
  - REQ-TRS-LINKTYPE-001
---

```gherkin
Feature: User-defined link types (TC-TRS-LINKTYPE-001)

  Scenario: a well-formed [linkTypes] table validates cleanly
    Given a .syscribe.toml declaring valid link types (camelCase and snake_case keys)
    When the model is validated
    Then no W630 and no E63x finding is raised

  Scenario: each structural defect raises W630 and the entry is ignored
    Given a table with a bad name, a built-in collision, an inverse collision, a bad cardinality, a lower bound without sourceTypes, an unknown element type, an unsupported base, relax without extends, a non-relaxable code and a duplicate inverse
    When the model is validated
    Then W630 names every defective entry
    And a links: use of an ignored entry raises E630

  Scenario: an unknown key raises W630 but the entry stays usable
    Given an entry with an unknown key
    When an element uses it
    Then W630 is raised for the key and no E630 for the use

  Scenario: a model without [linkTypes] is unaffected
    Given a model with no [linkTypes] table and no links:
    When the model is validated
    Then no W63x/E63x finding is raised
```
