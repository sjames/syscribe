---
type: TestCase
id: TC-TRS-SIDECAR-001
name: "the mutation and campaign sidecars are bounded, safe to read and atomic"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/sidecar_safety.rs
verifies:
  - REQ-TRS-SIDECAR-001
tags:
  - cli
---

```gherkin
Feature: sidecar files that tools append to

  Scenario: bounded per machine and subject
    Then recording more than the bound keeps the newest per subject, another subject keeps its own, and re-recording a hash replaces it

  Scenario: a missing file and an unreadable one
    Then no file is no records, and a damaged file is an error that names it, is never overwritten and is left for a person to fix

  Scenario: atomic writes
    Then both sidecars leave no temporary file and stay valid JSON
```
