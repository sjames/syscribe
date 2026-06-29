---
type: TestCase
id: TC-TRS-MCP-005
name: "create_element creates a new element and refuses to overwrite an existing qname"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_write.rs
verifies:
  - REQ-TRS-MCP-005
tags:
  - mcp
  - write
---

Verifies that `create_element` creates a new element file, auto-allocates a stable id for
id-identified types, and refuses to overwrite an element whose qualified name already exists.

```gherkin
Feature: Guarded element creation

  Scenario: a new element is created on commit
    Given an initialized mcp server over a fixture model
    When create_element is called with a fresh qname, a type, and dry_run=false
    Then a new .md file exists at the path derived from the qname
    And get_element resolves the new element

  Scenario: an id-identified type gets an auto-allocated stable id
    When create_element is called for a Requirement without an explicit id and dry_run=false
    Then the created element carries a stable id matching the requirement id pattern

  Scenario: creating an existing qname is refused
    Given an element that already exists in the fixture
    When create_element is called for that same qname with dry_run=false
    Then the call reports written=false and an already-exists error
    And the existing file is left unchanged
```
