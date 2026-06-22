---
id: TC-TRS-ID-007
type: TestCase
testLevel: L3
status: draft
name: "Verify configurable additional stable-ID prefixes via [ids.prefixes]: additive, per-type, id-resolvable, digit-capped, W046 on malformed config."
verifies:
  - REQ-TRS-ID-007
---

```gherkin
Feature: configurable additional stable-ID prefixes
  Scenario: baseline — an unconfigured custom prefix is rejected
    Given a model with no [ids.prefixes] config
    And a Requirement with id STK-SCHED-001
    When validate runs
    Then the STK id raises E006

  Scenario: configured prefix is accepted, additively, and resolves
    Given [ids.prefixes] Requirement = ["STK"]
    When validate runs
    Then a Requirement with id STK-SCHED-001 is clean
    And a Requirement with the built-in id REQ-SCHED-001 is still clean
    And a TestCase verifying STK-SCHED-001 resolves (no E102)
    And a Requirement with id STK-SCHED-000000001 raises E023 (digit cap applies)
    And a TestCase carrying id STK-TC-001 raises E006 (prefix is per-type)

  Scenario: malformed config raises W046 and skips only the bad entries
    Given [ids.prefixes] with an unknown type key and a malformed prefix
    And a well-formed sibling prefix GOOD on Requirement
    When validate runs
    Then the unknown type key raises W046
    And the malformed prefix raises W046
    And a Requirement with id GOOD-SCHED-001 is clean
```
