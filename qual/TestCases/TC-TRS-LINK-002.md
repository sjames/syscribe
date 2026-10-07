---
id: TC-TRS-LINK-002
type: TestCase
testLevel: L3
status: active
name: "Verify SVG element shapes are wrapped in a hyperlink to the hosted URL, and generated Mermaid carries click directives, only when [links] is configured."
verifies:
  - REQ-TRS-LINK-002
sourceFile: repo:qual/tests/tc/TC-TRS-LINK-002.sh
---

Black-box CLI check run by the qualification runner (`bash qual/tests/run_qual.sh
TC-TRS-LINK-002`) against the `TC-TRS-LINK-001` fixture roots: `linked` (a `[links]` table in
`.syscribe.toml`) and `none` (no table). Both carry `Diagrams::Pinned`, a fully pinned BDD
manifest naming `UAV::Avionics::FlightController` and `Requirements::SafeLanding`, exported with
`syscribe diagram export Diagrams::Pinned --format svg|mermaid` (`REQ-TRS-VIS-009`/`-010`).

Verifies that, with `[links]` configured, the exported SVG declares `xmlns:xlink` and wraps each
element's shape group in `<a xlink:href="<url>" href="<url>" target="_blank" rel="noopener">`
with the element's hosted URL, that the generated Mermaid gains a `click <id> href "<url>" _blank`
line per linked node, and that with no `[links]` table the SVG has no `<a>` wrappers and the
Mermaid no `click` lines while both are still produced.

```gherkin
Feature: clickable SVG element shapes (TC-TRS-LINK-002)

  Scenario: configured links wrap shapes in an SVG hyperlink
    Given a model with [links] configured and a fully pinned Diagram
    When the diagram is exported with diagram export --format svg
    Then the SVG declares xmlns:xlink
    And each element's shape group is wrapped in <a xlink:href="<hosted url>" href="<hosted url>" target="_blank" rel="noopener">
    And the anchor directly encloses the <g id sysml:ref> of that shape
    And there is exactly one anchor per linked shape

  Scenario: configured links add Mermaid click directives
    Given the same model
    When the diagram is exported with diagram export --format mermaid
    Then a click <id> href "<hosted url>" _blank line is emitted for the linked node
    And every node carries a %% ref: annotation

  Scenario: no [links] table leaves shapes unwrapped by hosted links
    Given a model with no [links] configured
    When the diagram is exported as svg and as mermaid
    Then the SVG is still drawn and contains no hosted-URL <a> wrappers
    And the Mermaid contains no click lines
```
