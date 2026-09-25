---
id: TC-TRS-PROJ-008
type: TestCase
testLevel: L3
status: draft
name: "Verify the --config lens applies to trace, why, who-verifies, refs and links."
verifies:
  - REQ-TRS-PROJ-001
---

Issue #139: the single-element query commands accepted `--config` but ignored it. Under the lens
they must read only the elements active in the configuration, and a start element that exists in
the 150% model but is inactive in the configuration is a usage error (exit `1`, message naming the
element and the configuration) rather than a silent whole-model answer or a fuzzy substitute.

```gherkin
Feature: The configuration lens on single-element query commands

  Scenario: inactive satisfiers and verifiers disappear under the lens
    Given a requirement satisfied by an always-active part and a Wdt-gated part
    And verified by an always-active TestCase and a Wdt-gated TestCase
    When trace, who-verifies, refs and links run with --config naming the no-Wdt variant
    Then the Wdt-gated part and TestCase are absent from every output
    And with --config naming the Wdt variant they are present
    And without --config they are present (the whole model)

  Scenario: why reads the projected view
    Given a part active in every variant
    When why runs with --config naming the no-Wdt variant
    Then it exits 0 and the Wdt-gated TestCase is not listed

  Scenario: an inactive start element is a usage error
    Given a requirement and a part gated on Wdt
    When trace, why, who-verifies, refs or links run on them with --config naming the no-Wdt variant
    Then each exits 1 with no stdout and stderr says the element is not active in the configuration

  Scenario: an unresolvable --config is a usage error
    When trace runs with --config naming no Configuration
    Then it exits 1
```
