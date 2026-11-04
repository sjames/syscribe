---
type: TestCase
id: TC-TRS-WRITEGUARD-001
name: "Stale writes are refused, envelopes are stable and exit codes are fixed"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/provenance_log.rs
verifies:
  - REQ-TRS-WRITEGUARD-001
tags:
  - cli
---

```gherkin
Feature: Write contract

  Scenario: expect-hash
    Then a stale hash is refused with nothing written and a current one is accepted

  Scenario: JSON envelope
    Then set reports wrote, wouldWrite, commit and the diff as one object

  Scenario: exit codes
    Then 0 success, 1 usage or refusal, 2 a gate, and impact with no element exits 2
```
