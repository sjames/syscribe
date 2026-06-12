---
id: TC-TRS-LINK-001
type: TestCase
testLevel: L3
status: draft
name: "Verify hosted source URLs resolve from the [links] config for file-backed elements."
verifies:
  - REQ-TRS-LINK-001
---

Verify that, with a `[links]` table in `.syscribe.toml`, the tool resolves a hosted source URL for each file-backed element (observed through `export`), and that with no `[links]` table no element resolves to a URL.

```gherkin
Feature: hosted source URL resolution

  Scenario: base_url resolves to model-root-relative path
    Given a model with [links] base_url configured
    When the model is exported
    Then each file-backed element carries a "url" pointing under the base_url
    And the FlightController element URL ends with UAV/Avionics/FlightController.md

  Scenario: no [links] table yields no URLs
    Given a model with no [links] configured
    When the model is exported
    Then no element carries a "url"
```
