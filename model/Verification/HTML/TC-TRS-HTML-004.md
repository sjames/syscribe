---
type: TestCase
id: TC-TRS-HTML-004
name: "Diagrams are embedded and render offline"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/export_html.rs
verifies:
  - REQ-TRS-HTML-004
tags:
  - html-export
---

```gherkin
Feature: Embedded diagrams

  Scenario: a SysML diagram is inlined as SVG
    Given a generated site over a fixture containing a BDD Diagram
    When the diagram's page is read
    Then it contains an inline <svg> element

  Scenario: a Mermaid diagram is embedded and rendered by a bundled script
    Given a generated site over a fixture containing a Mermaid Diagram
    When the diagram's page is read
    Then it contains a Mermaid block
    And the site bundles mermaid.min.js and the page references it (no CDN)
```
