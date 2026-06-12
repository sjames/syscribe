---
id: TC-TRS-OUT-009
type: TestCase
testLevel: L3
status: draft
name: "Verify executed-evidence glyphs/annotations in matrix and trace, plus --linked-only and graceful degradation."
verifies:
  - REQ-TRS-OUT-009
---

Verify that, with a committed `.syscribe/results.json` sidecar, `matrix` distinguishes passing-covered (`✓`) from covered-not-passing (`▣`) cells and `trace` annotates verifying TestCases with `[pass]`/`[fail]`/`[unknown]`; that `--linked-only` reverts both to the plain linked view; and that a model with no sidecar is unchanged.

```gherkin
Feature: Executed-evidence in matrix and trace (W010 results)

  Scenario: matrix distinguishes passing from covered-not-passing
    Given a model with a feature model, a Configuration, two requirements each with an active TestCase, and an ingested results sidecar where one TestCase passed and one failed
    When matrix is invoked
    Then the passing requirement's covered cell shows ✓
    And the failing requirement's covered cell shows ▣
    And the legend mentions "covered, not passing"

  Scenario: trace annotates verifying TestCases with the ingested verdict
    Given the same model with a results sidecar
    When trace is invoked on the passing requirement
    Then its verifying TestCase is annotated [pass]
    And trace on the failing requirement annotates its TestCase [fail]

  Scenario: --linked-only reverts matrix to the plain covered glyph
    Given the same model with a results sidecar
    When matrix is invoked with --linked-only
    Then no ▣ glyph appears and covered cells show ✓

  Scenario: --linked-only reverts trace to no verdict annotation
    Given the same model with a results sidecar
    When trace --linked-only is invoked on the failing requirement
    Then no [fail] or [pass] annotation appears

  Scenario: a model with no sidecar degrades gracefully
    Given a model with the same shape but no .syscribe/ directory
    When matrix and trace are invoked
    Then no ▣ glyph and no [pass]/[fail]/[unknown] annotation appear
```
