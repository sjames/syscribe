---
type: TestCase
id: TC-TRS-MCP-049
name: "The MCP server picks up external model edits automatically, and only those"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_watch.rs
verifies:
  - REQ-TRS-MCP-048
tags:
  - mcp
  - reload
---

```gherkin
Feature: File-watch auto-reload of the MCP store

  Scenario: an external edit is visible without calling reload
    Given an initialized mcp server on a copy of the fixture model
    When REQ-FX-001's name is changed on disk by another process
    Then get_element for REQ-FX-001 returns the new name within the deadline
    And a logging message {"event":"reload","source":"watch","count":N} was sent
    And a notifications/resources/list_changed notification was sent

  Scenario: a new file is resolvable without calling reload
    When a new Requirement file is created on disk
    Then get_element resolves it within the deadline

  Scenario: a --read-only server still watches
    Given a server started with --read-only
    When REQ-FX-001 is edited on disk
    Then get_element returns the new name

  Scenario: --no-watch keeps the loaded model until reload
    Given a server started with --no-watch
    When REQ-FX-001 is edited on disk
    Then get_element keeps returning the old name and no watch reload is logged
    And after the reload tool is called the new name is returned

  Scenario: the server's own committed write causes no second reload
    When update_element commits a new name for REQ-FX-001
    Then no watch reload is logged
    And a following external edit of REQ-FX-003 is picked up with exactly one watch reload

  Scenario: a half-saved file keeps the old store, then recovers
    When REQ-FX-001 is overwritten with unparseable frontmatter
    Then a reload_deferred warning is logged and get_element still returns the old name
    When the file is fixed
    Then get_element returns the fixed name

  Scenario: the watcher does not keep the process alive
    When the client closes the server's stdin
    Then the server process exits

  Scenario: the initialize instructions describe auto-reload
    Then the initialize result's instructions say the server reloads automatically
```
