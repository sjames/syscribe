---
type: TestCase
id: TC-TRS-MCP-015
name: "check_ref and next_id pre-write guards behave correctly"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_authoring.rs
verifies:
  - REQ-TRS-MCP-015
tags:
  - mcp
---

```gherkin
Feature: Pre-write guard tools

  Scenario: check_ref reports a resolving reference
    Given an initialized mcp server over a fixture model
    When check_ref is called with an existing element reference
    Then it reports resolved true with the element's qname, id, and type

  Scenario: check_ref reports an unresolvable reference
    When check_ref is called with a nonexistent reference
    Then it reports resolved false without erroring

  Scenario: next_id returns the next free stable id for a prefix
    When next_id is called for a prefix already used in the model
    Then it returns an id with that prefix not already taken
```
