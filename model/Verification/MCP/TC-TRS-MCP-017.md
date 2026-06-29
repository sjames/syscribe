---
type: TestCase
id: TC-TRS-MCP-017
name: "Elements are readable as resources with reference completion"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_authoring.rs
verifies:
  - REQ-TRS-MCP-017
tags:
  - mcp
---

```gherkin
Feature: Element resources and completion

  Scenario: an element is readable via its resource URI
    Given an initialized mcp server over a fixture model
    When a resources/read is issued for syscribe://element/Requirements::REQ-FX-001
    Then the structured detail of REQ-FX-001 is returned

  Scenario: a resource template is advertised
    When resource templates are listed
    Then a syscribe://element/ template is present

  Scenario: element references are completable
    When completion is requested for an element-reference argument with a prefix
    Then candidate qnames or ids matching the prefix are returned
```
