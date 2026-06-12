---
id: TC-W009-TRIG-RS-001
type: TestCase
name: "Rust testFunction was renamed in the source"
status: active
testLevel: L3
verifies:
  - REQ-W009-TRIG-001
sourceFile: tests.rs
testFunctions:
  - function: "mutex::tests::present_fn"
    scenario: "present resolves"
  - function: "mutex::tests::renamed_fn"
    scenario: "renamed no longer resolves"
---

```gherkin
Feature: W009 rust trigger
  Scenario: present resolves
    Given present_fn exists
    Then no W009 for it
  Scenario: renamed no longer resolves
    Given renamed_fn was removed
    Then a W009 finding is emitted
```
