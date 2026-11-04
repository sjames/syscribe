---
type: TestCase
id: TC-TRS-SCAFFOLD-001
name: "Scaffolds validate with zero new findings and mistakes are refused first"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/scaffold_ple.rs
verifies:
  - REQ-TRS-SCAFFOLD-001
tags:
  - cli
---

```gherkin
Feature: scaffolding

  Scenario: the three kinds
    Then a feature, a derived Configuration and a parameter are created or appended, the model validates exactly as before, and the created parameter can be bound

  Scenario: refusals
    Then bad names, ids, stages, ranges, defaults and leaf parents are refused with the reason before any file exists

  Scenario: MCP
    Then the tool previews by default, writes a clean scaffold when asked, refuses bad input, and is hidden under --read-only
```
