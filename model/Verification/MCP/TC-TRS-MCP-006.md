---
type: TestCase
id: TC-TRS-MCP-006
name: "update_element patches frontmatter while preserving unknown keys and body"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_write.rs
verifies:
  - REQ-TRS-MCP-006
tags:
  - mcp
  - write
---

Verifies that `update_element` changes only the requested frontmatter keys and the body when
asked, leaving every other frontmatter key — including unrecognised ones — and the untouched
content intact.

```gherkin
Feature: Non-destructive element update

  Scenario: a single frontmatter key is patched
    Given a fixture element carrying a custom unrecognised frontmatter key and a body
    When update_element sets status to a new value with dry_run=false
    Then the element's status is the new value
    And the custom unrecognised key is still present with its original value
    And the Markdown body is byte-for-byte unchanged

  Scenario: the body is patched without disturbing frontmatter
    When update_element replaces the body with dry_run=false
    Then the body is the new text
    And all frontmatter keys retain their original values
```
