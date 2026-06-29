---
type: TestCase
id: TC-TRS-HTML-005
name: "Validation, coverage, and traceability report pages are generated"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/export_html.rs
verifies:
  - REQ-TRS-HTML-005
tags:
  - html-export
---

```gherkin
Feature: Report pages

  Scenario: the report pages exist and carry their content
    Given a generated site over the fixture model
    Then reports/validation.html exists and lists validation findings
    And reports/coverage.html exists and reports verification coverage
    And reports/traceability.html exists and relates requirements to test cases
```
