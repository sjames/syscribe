---
id: TC-TRS-ID-008
type: TestCase
testLevel: L3
status: draft
name: "Verify Configuration ids need no numeric suffix; the numeric form still validates and malformed ids are still E200."
verifies:
  - REQ-TRS-ID-008
---

```gherkin
Feature: Configuration id grammar
  Scenario: descriptive and numeric ids
    Given Configurations CONF-GTI-US-M3, CONF-GTI-IN-M2 and CONF-GTI-US-001
    When the tool runs `validate`
    Then no E200 or W042 is reported
    And `validate --config CONF-GTI-US-M3` runs on that variant
  Scenario: malformed ids
    Given Configurations with ids CONF, CONF-gti-US, CONF-G and a 13-character segment
    When the tool runs `validate`
    Then each is reported as E200
```
