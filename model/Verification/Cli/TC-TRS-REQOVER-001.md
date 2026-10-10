---
type: TestCase
id: TC-TRS-REQOVER-001
name: "the overview lists unverified and unlinked requirements, capped and sorted"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-server/tests/req_explorer.rs
verifies:
  - REQ-TRS-REQOVER-001
tags:
  - server
---

```gherkin
Feature: explorer overview landing

  Scenario: lists
    Given requirements with and without verifying tests and links
    Then unverifiedList and unlinkedList carry id, qname, name and status, sorted by id, and respect the cap

  Scenario: page
    Then the page has the overview container
```
