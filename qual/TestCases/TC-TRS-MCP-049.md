---
id: TC-TRS-MCP-049
type: TestCase
testLevel: L3
status: draft
name: "Verify the MCP server reloads its model automatically after an on-disk edit, and does not under --no-watch."
verifies:
  - REQ-TRS-MCP-048
---

```gherkin
Feature: MCP file-watch auto-reload (TC-TRS-MCP-049)

  Background:
    Given a copy of a one-requirement model (REQ-WATCH-001 named "Original name")

  Scenario: an external edit is visible without calling reload
    Given `syscribe mcp` serving the copy over stdio
    When REQ-WATCH-001's name is changed on disk to "Edited on disk"
    Then get_element returns "Edited on disk" without a reload call
    And a logging message {"event":"reload","source":"watch"} was sent
    And a notifications/resources/list_changed notification was sent
    And the server exits once its stdin is closed

  Scenario: --no-watch keeps the loaded model
    Given `syscribe mcp --no-watch` serving the copy over stdio
    When REQ-WATCH-001's name is changed on disk
    Then get_element keeps returning "Original name" and no watch reload is logged
    And the server exits once its stdin is closed
```
