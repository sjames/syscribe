---
type: TestCase
id: TC-TRS-HTML-001
name: "export-html writes a multi-file static site to the output directory"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/export_html.rs
verifies:
  - REQ-TRS-HTML-001
tags:
  - html-export
---

```gherkin
Feature: Static site generation

  Scenario: the site is generated into the output directory
    Given a fixture model
    When syscribe export-html --out <dir> is run
    Then the command exits 0
    And <dir>/index.html exists
    And an element page exists under <dir>/elements/ for each model element
```
