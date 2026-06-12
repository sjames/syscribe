---
id: TC-TRS-CLI-008
type: TestCase
testLevel: L3
status: draft
name: "Verify the clap top-level router: unknown command rejected (non-zero, no model needed); known commands, man-page help, and version preserved."
verifies:
  - REQ-TRS-CLI-008
---

```gherkin
Feature: clap-based top-level command router
  Scenario: unknown commands are rejected, known behaviour preserved
    Given the syscribe binary
    When run as `syscribe bogus-command` from a directory with no model
    Then it exits non-zero and writes an error to stderr (no "model path" error)
    And `syscribe -m <fixture> list PartDef` still lists an element and exits 0
    And `syscribe validate --help` still prints a SYNOPSIS man-page and exits 0
    And `syscribe --version` still prints "syscribe <semver>" and exits 0
    And the explicit `report` command runs the default validation report
```
