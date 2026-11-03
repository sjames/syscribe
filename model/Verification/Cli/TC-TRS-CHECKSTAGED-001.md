---
type: TestCase
id: TC-TRS-CHECKSTAGED-001
name: "check-staged judges the staged content and the hook enforces it"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/check_staged.rs
verifies:
  - REQ-TRS-CHECKSTAGED-001
tags:
  - cli
---

```gherkin
Feature: check-staged

  Scenario: a staged test case without Gherkin
    Then the check fails and names scaffold-gherkin --fix

  Scenario: unrelated files
    Then the check exits 0 quickly without loading the model

  Scenario: partial staging
    Then it is the staged content, not the working tree, that is judged

  Scenario: dependants, deletions and config
    Then findings on direct dependants are reported, a deletion reports what it breaks, and a config change checks everything

  Scenario: gating
    Then warnings fail only under --deny, --max-warnings or --warnings-as-errors

  Scenario: the hook
    Then a bad commit is blocked, a good one passes, a missing tool is reported and a foreign hook is never overwritten
```
