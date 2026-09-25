---
id: TC-TRS-CLI-007
type: TestCase
testLevel: L3
status: draft
name: "Verify version reporting via --version, -V, and the version subcommand (exit 0, no model dir)."
verifies:
  - REQ-TRS-CLI-007
---

```gherkin
Feature: the tool reports its own version
  Scenario: version after a model flag (issue #133)
    Given a model directory
    When the user runs `syscribe -m <root> version`, `syscribe --model <root> version` or `syscribe --model=<root> version`
    Then the output is exactly "syscribe <semver>" and the exit code is 0
  Scenario: all three spellings print the version and exit 0
    Given the syscribe binary and no model directory
    When the user runs `syscribe --version`, `syscribe -V`, and `syscribe version`
    Then each prints "syscribe <semver>" to stdout
    And each matches the syscribe crate package version
    And each exits 0
    And it works from a directory with no model or .syscribe.toml
```
