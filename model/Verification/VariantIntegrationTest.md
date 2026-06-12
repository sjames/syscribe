---
type: TestCase
id: TC-UAV-VAR-001
name: "Each product configuration passes its variant integration smoke test"
status: active
testLevel: L4
verifies:
  - REQ-UAV-VAR-000
tags:
  - product-line
  - integration
---

Per-product integration test executed for every `Configuration` in the product
line. The projected variant is built and exercised end-to-end against its active
requirements.

Run: `cargo xtask hil -- variant-smoke --all-configs`

```gherkin
Feature: Product-line variant integration

  Scenario Outline: A configured product meets its active requirements
    Given the UAV is built per configuration <config>
    When the variant integration smoke test is executed
    Then all requirements active in <config> pass

    Examples:
      | config                |
      | CONF-UAV-SURVEY-001   |
      | CONF-UAV-MAPPING-001  |
      | CONF-UAV-DELIVERY-001 |
```
