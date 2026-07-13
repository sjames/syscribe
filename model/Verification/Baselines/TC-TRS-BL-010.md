---
type: TestCase
id: TC-TRS-BL-010
name: "Configured [baselines] directories redirect element and manifest output"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/baseline.rs
tags:
  - baseline
  - config
verifies:
  - REQ-TRS-BL-010
---

Verifies that `[baselines]` in `.syscribe.toml` redirects where `baseline create` writes the
element and the manifest, that defaults are unchanged when unset, and that an escaping
`element_dir` is rejected.

```gherkin
Feature: Configurable baseline output locations

  Scenario: Configured directories redirect output
    Given .syscribe.toml with [baselines] element_dir = "Releases" and manifest_dir = "evidence"
    When `baseline create --tag REL-2026-06` is run
    Then the Baseline element is written under model/Releases/
    And the manifest is written under <git-root>/evidence/
    And the element still validates and verifies (the manifest is self-recorded)

  Scenario: Defaults apply when unset
    Given no [baselines] table
    Then the element is written under model/Baselines/ and the manifest under <git-root>/baselines/

  Scenario: An element_dir escaping the model root is rejected
    Given [baselines] element_dir = "../outside"
    When create is run
    Then it errors and writes nothing
```
