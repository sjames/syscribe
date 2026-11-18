---
id: TC-TRS-MINVER-002
type: TestCase
testLevel: L3
status: active
name: "Verify syscribe-server refuses a model that needs a newer syscribe."
verifies:
  - REQ-TRS-MINVER-001
sourceFile: repo:crates/syscribe-server/tests/min_version.rs
testFunctions:
  - the_server_refuses_a_model_that_needs_a_newer_syscribe
---

```gherkin
Feature: Minimum syscribe version in the server (TC-TRS-MINVER-002)

  Scenario: The server refuses the model
    Given a model declaring a minVersion newer than the running syscribe-server
    When syscribe-server starts on it
    Then it exits 1 naming both versions
```
