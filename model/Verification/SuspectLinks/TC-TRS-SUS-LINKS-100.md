---
type: TestCase
id: TC-TRS-SUS-LINKS-100
name: "End-to-end suspect-link lifecycle: baseline, change, detect, clear"
status: active
testLevel: L3
sourceFile: repo:crates/syscribe/tests/suspect_links.rs
tags:
  - traceability
  - suspect-links
  - integration
verifies:
  - REQ-TRS-SUS-LINKS-000
---

Integration test exercising the full lifecycle the feature promises: a reviewed link is
baselined, a later change to its target makes it suspect, the reviewer sees it and clears
it, and the model returns to clean.

```gherkin
Feature: Suspect-link lifecycle

  Scenario: A reviewed link goes suspect when its target changes, then is cleared
    Given a model with a TestCase T that verifies requirement R
    And the reviewer runs "suspect accept T R" to baseline the link
    When validate is run
    Then no W090 is emitted

    When R's normative text is edited
    And validate is run
    Then W090 is emitted for the T-to-R link
    And "suspect list" shows the T-to-R link as suspect

    When the reviewer runs "suspect accept T R"
    And validate is run
    Then no W090 is emitted
    And "suspect list" shows no suspect links
```
