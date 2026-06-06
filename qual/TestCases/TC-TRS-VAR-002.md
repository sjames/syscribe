---
id: TC-TRS-VAR-002
type: TestCase
testLevel: L3
status: draft
title: "Verify TestCase-to-Configuration membership is derived from appliesWhen."
verifies:
  - REQ-TRS-VAR-002
---

Verify that a `TestCase` runs in a `Configuration` iff its `appliesWhen:` is satisfied by that configuration's selections, that a `TestCase` with no `appliesWhen:` is configuration-agnostic, and that the relationship surfaces in `links` and `refs`.

```gherkin
Feature: TestCase variant membership via appliesWhen

  Scenario: links shows the TestCase appliesWhen condition
    Given a TestCase with appliesWhen: Features::Wdt
    When the tool prints links for that TestCase
    Then the output references the Features::Wdt FeatureDef

  Scenario: refs of a selecting configuration lists the in-condition TestCase
    Given a Configuration that selects Features::Wdt
    When the tool prints refs for that Configuration
    Then the output lists the TestCase whose appliesWhen is Features::Wdt

  Scenario: refs of a deselecting configuration excludes the conditioned TestCase
    Given a Configuration that deselects Features::Wdt
    When the tool prints refs for that Configuration
    Then the output does not list the Features::Wdt-conditioned TestCase
    And it still lists the configuration-agnostic TestCase
```
