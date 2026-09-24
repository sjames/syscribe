---
id: TC-TRS-PKG-001
type: TestCase
testLevel: L3
status: draft
name: "Verify show and export-html list a package's direct members generated from the directory."
verifies:
  - REQ-TRS-PKG-001
---

```gherkin
Feature: Generated package membership
  Scenario: show lists direct members from the directory
    Given a package whose _index.md prose mentions only one of its members
    When show is run on the package
    Then a Members section lists every direct child with id, type, name and status
    And grandchildren are not listed

  Scenario: --no-related keeps the member list
  Scenario: an empty package shows an explicit empty state
  Scenario: export-html package page lists members with links
```
