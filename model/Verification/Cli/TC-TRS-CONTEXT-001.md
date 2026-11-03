---
type: TestCase
id: TC-TRS-CONTEXT-001
name: "context bundles an element's neighbours, verification and findings within a bound"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/context_bundle.rs
verifies:
  - REQ-TRS-CONTEXT-001
tags:
  - cli
---

```gherkin
Feature: context briefing bundle

  Scenario: a requirement with its verification
    Then the bundle lists its neighbours, the verifying test with verdict and environment, and its findings

  Scenario: a planning item
    Then the bundle lists parent, children, achieves, evidence, claim and open questions

  Scenario: the size bound
    Then output never exceeds max-bytes at any bound in either format, JSON stays valid, and the omissions are named

  Scenario: errors and surfaces
    Then an unknown reference exits 1, a bad option exits 2, and the MCP tool returns the same data
```
