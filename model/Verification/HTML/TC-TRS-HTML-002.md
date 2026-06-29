---
type: TestCase
id: TC-TRS-HTML-002
name: "Element pages render frontmatter, documentation, and metadata"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/export_html.rs
verifies:
  - REQ-TRS-HTML-002
tags:
  - html-export
---

```gherkin
Feature: Element pages

  Scenario: an element page shows its identity and rendered doc
    Given a generated site over the fixture model
    When the page for REQ-FX-001 is read
    Then it contains the element's name and stable id
    And it contains the element's documentation rendered as HTML
```
