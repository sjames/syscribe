---
id: TC-TRS-CLI-009
type: TestCase
testLevel: L3
status: draft
name: "Verify invalid option values and unknown options are usage errors (exit 1, empty stdout, named option), and valid options still work."
verifies:
  - REQ-TRS-CLI-009
---

Issue #133: enumerated/numeric option values and unknown options are rejected instead of
silently falling back to defaults.

```gherkin
Feature: CLI argument hygiene

  Scenario: invalid enumerated values are rejected and the valid values listed
    Given a small model
    When impact --direction sideways, impact/n2/behavioral-coverage/sbom/build-config --format xml,
      validate --results <file> --format bogus, or diagram render --view bogus is run
    Then each exits non-zero with nothing on stdout
    And stderr names the offending value and lists the valid values

  Scenario: non-integer counts are rejected
    When n2 --depth abc, impact --depth -1, behavioral-coverage --depth x, digest --limit x,
      topics --top x, clusters --k x or verification-depth --min-levels x is run
    Then each exits 1 with nothing on stdout and stderr names the option

  Scenario: unknown options are rejected on the checked commands
    When validate, list, show, trace, impact, links, export, find or ls is given --bogus
    Then each exits 1 with nothing on stdout and stderr names --bogus
    And a value-taking option with no value (impact X --depth) is rejected the same way
    And the check happens even when the model directory does not exist

  Scenario: valid options still work
    When impact, n2, behavioral-coverage, sbom, build-config, validate, list, show, trace, export,
      find and ls are run with their documented options
    Then each exits 0
```
