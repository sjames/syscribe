---
id: TC-TRS-MG-013
type: TestCase
testLevel: L3
status: draft
name: "Verify magicgrid --audit: clean model PASS (exit 0) + readiness; a MagicGrid error lists the code and FAILs (exit 2); plain magicgrid has no verdict; --json."
verifies:
  - REQ-TRS-MG-013
---

```gherkin
Feature: magicgrid --audit rolls up MagicGrid findings, readiness, and a verdict
  Scenario: a clean MagicGrid model passes
    Given a clean MagicGrid model
    When magicgrid --audit is run
    Then it reports Verdict PASS and exits zero

  Scenario: the audit reports readiness (SoI / MoE / grid)
    Given a clean MagicGrid model with a unique system of interest
    When magicgrid --audit is run
    Then the readiness summary names the system of interest

  Scenario: a MagicGrid error lists the code and fails
    Given a non-draft UseCaseDef with no actor
    When magicgrid --audit is run
    Then MG013 is listed and Verdict FAIL is reported with a non-zero exit

  Scenario: plain magicgrid has no verdict
    Given any MagicGrid model
    When magicgrid is run without --audit
    Then the grid is printed and no Verdict line appears

  Scenario: the audit emits JSON
    When magicgrid --audit --json is run
    Then a JSON object with a verdict field is produced
```
