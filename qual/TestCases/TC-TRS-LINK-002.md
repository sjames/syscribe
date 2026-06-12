---
id: TC-TRS-LINK-002
type: TestCase
testLevel: L3
status: draft
name: "Verify SVG element shapes are wrapped in a hyperlink to the hosted URL."
verifies:
  - REQ-TRS-LINK-002
---

Verify that, with `[links]` configured, the requirement-trace SVG wraps each element shape in `<a xlink:href=... href=... target="_blank">` to the hosted URL, and that with no `[links]` table the SVG has no hosted hyperlink wrappers.

```gherkin
Feature: clickable SVG element shapes

  Scenario: configured links wrap shapes in an SVG hyperlink
    Given a model with [links] configured
    When a requirement-trace SVG is rendered
    Then each shape is wrapped in <a ... href="<hosted url>" target="_blank">
    And the anchor carries both xlink:href and href

  Scenario: no [links] table leaves shapes unwrapped by hosted links
    Given a model with no [links] configured
    When a requirement-trace SVG is rendered
    Then the SVG contains no hosted-URL <a> wrappers
```
