---
type: TestCase
id: TC-TRS-VRES-001
name: "W312/E319 flag requirements whose active verifier fails"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/requirement_failing_verifier.rs
verifies:
  - REQ-TRS-VRES-001
tags:
  - verification
---

```gherkin
Feature: Requirement-level failing verifier

  Scenario: approved requirement
    Given results where an active verifier fails
    Then W312 is raised on the requirement

  Scenario: verified requirement
    Then E319 is raised and W312 is not

  Scenario: passing, draft, unknown or no results
    Then nothing is raised
```
