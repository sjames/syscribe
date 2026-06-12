---
id: TC-TRS-MG-007
type: TestCase
testLevel: L3
status: draft
name: "Verify the MoE-weighted trade study: MoE rows x configuration columns; objective scores 1.0; sub-threshold scores 0 and fails the config; ranked rollup; --config; --json; n/a cells."
verifies:
  - REQ-TRS-MG-007
---

```gherkin
Feature: MoE-weighted trade study comparing configurations
  Scenario: the grid lists MoEs against configurations
    Given a model with MoE elements and several Configurations
    When trade-study is run
    Then every MoE is a row, every Configuration is a column, and each cell shows value, score and weighted contribution

  Scenario: a value beyond the objective scores 1.0
    Given a maximize MoE whose projected value exceeds its objective under a configuration
    When trade-study is run
    Then that cell scores 1.0

  Scenario: a sub-threshold value fails the configuration
    Given a MoE whose projected value is worse than its threshold under a configuration
    When trade-study is run
    Then that cell scores 0, is flagged a threshold violation, and the configuration is marked failing in the rollup

  Scenario: the rollup ranks configurations and marks the winner
    Given configurations with differing weighted totals and none failing
    When trade-study is run
    Then the footer ranks them by weighted total and marks the top one as the winner

  Scenario: --config restricts the columns
    When trade-study --config A --config B is run
    Then only configurations A and B are columns

  Scenario: an unevaluable cell is n/a and dropped from weight normalisation
    Given a MoE whose host has no numeric result under a configuration
    When trade-study is run
    Then that cell is reported n/a and excluded from that column's weight normalisation

  Scenario: the trade study emits JSON
    When trade-study --json is run
    Then a JSON grid and per-configuration rollup are produced
```
