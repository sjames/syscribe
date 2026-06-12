---
id: TC-TRS-DISC-005
type: TestCase
testLevel: L3
status: draft
name: "Verify `why-active <el> --config <C>`: active/inactive/always-active verdict; errors on missing/unresolved --config."
verifies:
  - REQ-TRS-DISC-005
---

```gherkin
Feature: why-active — element activation verdict under a configuration
  Scenario: active under a selecting config
    Given a product-line model
    When the tool runs `why-active <gated el> --config <config that selects its feature>`
    Then output names the element and prints "Verdict: active"
  Scenario: inactive under a non-selecting config
    When the tool runs `why-active <same gated el> --config <config that does not select its feature>`
    Then output prints "Verdict: inactive"
  Scenario: always active for an ungated element
    When the tool runs `why-active <always-active el> --config <any config>`
    Then output prints "Verdict: always active"
  Scenario: --config required and must resolve
    When --config is omitted, or names an unknown configuration
    Then the tool exits non-zero
```
