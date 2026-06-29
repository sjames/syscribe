---
type: TestCase
id: TC-TRS-HTML-006
name: "Client-side search index and script are emitted"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/export_html.rs
verifies:
  - REQ-TRS-HTML-006
tags:
  - html-export
---

```gherkin
Feature: Offline search

  Scenario: a search index and script are written
    Given a generated site over the fixture model
    Then a search index file is present and is valid JSON containing the model elements
    And a bundled search script is present
    And an element's entry carries its qualified name, id, and page URL
```
