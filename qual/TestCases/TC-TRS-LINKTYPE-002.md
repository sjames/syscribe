---
id: TC-TRS-LINKTYPE-002
type: TestCase
testLevel: L3
status: draft
name: "Verify links: entries resolve against declared link types, with E630/E631/E632 for undeclared types, malformed shapes and dangling targets."
verifies:
  - REQ-TRS-LINKTYPE-002
---

```gherkin
Feature: User-defined link types (TC-TRS-LINKTYPE-002)

  Scenario: a valid links: entry validates cleanly
    Given a declared link type used with id and qname targets
    When the model is validated
    Then no E63x and no W047 is raised for the element

  Scenario: an undeclared link type raises E630 naming the declared types
    Given a links: key that is not declared
    When the model is validated
    Then E630 is raised and its message lists the declared types

  Scenario: a malformed links: shape raises E631
    Given links: as a list, and a key whose value is a mapping
    When the model is validated
    Then E631 is raised for each

  Scenario: a dangling target raises E632
    Given a links: target that does not resolve
    When the model is validated
    Then E632 is raised

  Scenario: links: with no [linkTypes] table raises E630 with a declaration hint
    Given a model with links: but no [linkTypes] table
    When the model is validated
    Then E630 is raised stating no link types are declared
```
