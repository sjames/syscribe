---
type: TestCase
id: TC-TRS-MCP-009
name: "Write tools reject path-traversal qnames and never write outside the model root"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_write.rs
verifies:
  - REQ-TRS-MCP-009
tags:
  - mcp
  - write
  - security
---

Verifies that a write tool refuses a qualified name that would escape the model root and that
no file is created outside it.

```gherkin
Feature: Write-path confinement

  Scenario: create_element refuses a traversal qname
    Given an initialized mcp server over a fixture model
    When create_element is called with a qname containing a parent-directory segment and dry_run=false
    Then the call reports written=false and a path-confinement error
    And no .md file is created above the model root

  Scenario: move_element refuses a traversal destination
    When move_element is called with a destination qname that would resolve outside the model root and dry_run=false
    Then the call reports written=false
    And the source element remains at its original path

  Scenario: create_element refuses a write that escapes via a symlinked directory
    Given a fixture model containing a directory symlink that points outside the model root
    When create_element targets a qname under that symlinked directory and dry_run=false
    Then the call reports written=false
    And no file is created at the symlink's target outside the model root
```
