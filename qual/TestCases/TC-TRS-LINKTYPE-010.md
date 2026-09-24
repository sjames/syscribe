---
id: TC-TRS-LINKTYPE-010
type: TestCase
testLevel: L3
status: draft
name: "Verify the MCP server exposes read-only link_types and follow tools."
verifies:
  - REQ-TRS-LINKTYPE-010
---

```gherkin
Feature: User-defined link types (TC-TRS-LINKTYPE-010)

  Scenario: tools/list advertises link_types and follow
  Scenario: link_types returns the declared vocabulary
  Scenario: follow returns traversal results
```
