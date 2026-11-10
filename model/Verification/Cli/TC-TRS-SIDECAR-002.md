---
type: TestCase
id: TC-TRS-SIDECAR-002
name: "a corrupt mutation sidecar is refused with exit 1 and reported by the coverage commands"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/exec_coverage.rs
verifies:
  - REQ-TRS-SIDECAR-001
tags:
  - cli
---

```gherkin
Feature: an unreadable mutation sidecar

  Scenario: mutate --record
    Then it exits 1 saying nothing was recorded and to fix or remove the file, and leaves the file untouched

  Scenario: the coverage commands
    Then requirement-coverage reports the unreadable scores in its errors
```
