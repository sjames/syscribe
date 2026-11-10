---
type: TestCase
id: TC-TRS-MOEMEAS-001
name: "trade-study scores recorded measurements and knocks out recorded failures"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/trade_study_measured.rs
verifies:
  - REQ-TRS-MOEMEAS-001
tags:
  - cli
  - magicgrid
---

```gherkin
Feature: ranking Configurations on recorded evidence

  Scenario: the design favours a variant whose recorded evidence fails
    Then with mg_moe_source measurement the other variant wins, the failing one is FAIL with its reasons listed, and predicted is shown beside measured

  Scenario: the prediction still scores by default
    Then a recorded failure still knocks the variant out and the measured value is shown beside the predicted one

  Scenario: nothing recorded
    Then the cells are n/a and no variant is the winner

  Scenario: aggregates, units and limits
    Then mean, max, p95 and last combine values given in different units, a wrong dimension is an error, and a fail verdict or broken limit is a violation

  Scenario: records from a file outside the model
    Then a directory or a named file ranks without ingesting, replaces the sidecar for the Configurations it covers, treats a missing file as no evidence, names the file and its hash, ignores and counts unmatched records, and refuses a malformed argument or file

  Scenario: the new fields are validated
    Then a bad source, aggregate, missing measurement name or unknown unit is MG033
```
