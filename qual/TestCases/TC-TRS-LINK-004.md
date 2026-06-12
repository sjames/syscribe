---
id: TC-TRS-LINK-004
type: TestCase
testLevel: L3
status: draft
name: "Verify Markdown report and export render element references as hosted links."
verifies:
  - REQ-TRS-LINK-004
---

Verify that, with `[links]` configured, the validation report renders element references as `[<label>](<hosted url>)` and the JSON export carries the hosted `url`, and that with no `[links]` table the report contains no hosted links.

```gherkin
Feature: hosted Markdown links in report and export

  Scenario: configured links render report element references as Markdown links
    Given a model with [links] configured
    When the validation report is produced
    Then a satisfying element is rendered as [<qname>](<hosted url>)

  Scenario: no [links] table leaves the report unlinked
    Given a model with no [links] configured
    When the validation report is produced
    Then the report contains no hosted Markdown links
```
