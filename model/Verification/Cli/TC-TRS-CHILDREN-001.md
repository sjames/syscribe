---
type: TestCase
id: TC-TRS-CHILDREN-001
name: "children lists direct children with counters, pages and a configuration tree"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/children.rs
verifies:
  - REQ-TRS-CHILDREN-001
tags:
  - cli
---

```gherkin
Feature: explorer tree data

  Scenario: containment
    Then root and package children carry counters, packages come first, and a virtual package expands

  Scenario: pages and configurations
    Then pages cover the children once and the configuration tree follows derivedFrom and subConfigurations

  Scenario: surfaces
    Then errors exit 1 or 2, nothing is written, and the CLI, MCP tool and LSP request return the same page
```
