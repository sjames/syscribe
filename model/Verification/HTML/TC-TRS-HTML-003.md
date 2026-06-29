---
type: TestCase
id: TC-TRS-HTML-003
name: "Navigation tree and resolved cross-reference links are present"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/export_html.rs
verifies:
  - REQ-TRS-HTML-003
tags:
  - html-export
---

```gherkin
Feature: Navigation and cross-links

  Scenario: a reference becomes a hyperlink to the target page
    Given a generated site over a fixture where Derived has supertype Base
    When the page for Parts::Derived is read
    Then it contains a hyperlink to the Parts::Base element page

  Scenario: every page carries a navigation tree
    When any element page is read
    Then it contains a navigation tree linking to element pages
```
