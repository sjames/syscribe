---
type: TestCase
id: TC-TRS-HTML-007
name: "Custom CSS replaces the default and output is fully offline"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/export_html.rs
verifies:
  - REQ-TRS-HTML-007
tags:
  - html-export
---

```gherkin
Feature: Styling and offline guarantee

  Scenario: a default stylesheet is written when none is supplied
    Given a generated site over the fixture model with no --css
    Then <dir>/style.css exists and every page links style.css

  Scenario: --css replaces the default stylesheet
    Given a custom CSS file with a recognisable marker rule
    When syscribe export-html --css <file> is run
    Then <dir>/style.css contains the custom marker

  Scenario: the generated site has no network references
    Given a generated site over the fixture model
    Then no generated file contains an http:// or https:// reference
```
