---
id: TC-W009-RS-001
type: TestCase
title: "rust testFunction resolves in source"
status: draft
testLevel: L3
verifies:
  - REQ-W009-CLEAN-001
sourceFile: rust_tests.rs
testFunctions:
  - function: "mutex::tests::acquire_returns_ok"
    scenario: "rust resolves"
---

```gherkin
Feature: rust matcher
  Scenario: rust resolves
    Given the source defines the test
    Then no W009 is emitted
```
