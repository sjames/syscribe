---
type: TestCase
id: TC-BCOV-SRC-001
name: "exercise ActSrc"
status: active
testLevel: L3
sourceFile: src/actsrc/run.rs
verifies:
  - REQ-BCOV-001
---
```gherkin
Feature: src
  Scenario: runs
    Then it works
```
