---
id: TC-TRS-LINK-002
type: TestCase
testLevel: L3
status: retired
name: "Verify SVG element shapes are wrapped in a hyperlink to the hosted URL."
verifies:
  - REQ-TRS-LINK-002
---

**Retired** with the CLI diagram toolkit (`ADR-SYS-VIS-001`, `REQ-TRS-VIS-013`): this case
exercised `[links]` hyperlinks through `diagram req`, which no longer exists. The SVG
hyperlink behaviour will be re-verified against `syscribe diagram export --format svg`
(`REQ-TRS-VIS-009`/`REQ-TRS-VIS-010`) when it lands.

Verified that, with `[links]` configured, the requirement-trace SVG wrapped each element shape in `<a xlink:href=... href=... target="_blank">` to the hosted URL, and that with no `[links]` table the SVG has no hosted hyperlink wrappers.

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
