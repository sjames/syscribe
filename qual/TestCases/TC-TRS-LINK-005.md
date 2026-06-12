---
id: TC-TRS-LINK-005
type: TestCase
testLevel: L3
status: draft
name: "Verify the live web UI detail panel shows a per-element source-link icon to the hosted model element."
verifies:
  - REQ-TRS-LINK-005
---

Verify that, in the live web server, the element detail panel renders a small
"view source" icon linking to the element's hosted URL in a new tab when
`[links]` is configured, and renders no such icon when `[links]` is absent.
Exercised by an in-process Axum integration test (`tower::ServiceExt::oneshot`)
against the same router `main` serves.

```gherkin
Feature: live detail-panel source-link icon

  Scenario: detail panel shows a source-link icon when [links] is configured
    Given a model with a [links] base_url configured
    When the detail panel for a file-backed element is requested
    Then the response contains an anchor with target="_blank" and rel="noopener"
    And the anchor href is the element's resolved hosted URL

  Scenario: no source-link icon when [links] is absent
    Given a model with no [links] configured
    When the detail panel for the same element is requested
    Then the response contains no source-link icon
```
