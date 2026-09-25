---
id: TC-TRS-QUAL-001
type: TestCase
testLevel: L3
status: draft
name: "Verify the qualification model validates with no errors and no W047, and the TVR version comes from the binary"
verifies:
  - REQ-TRS-QUAL-001
---

Self-check of the qualification model (`qual/`) with the binary under qualification.

```gherkin
Feature: qualification model self-consistency (GH #173)

  Scenario: the qualification model validates clean of W047
    Given the qualification model under qual/
    When the user runs `syscribe -m qual validate`
    Then the command exits 0
    And no W047 (unrecognised frontmatter field) finding is reported

  Scenario: the TVR version is not maintained in model data
    Given the qualification package qual/_index.md
    Then it carries no top-level version: key
    And run_qual.sh passes the binary's --version to the TVR generator
```
